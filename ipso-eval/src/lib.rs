mod test;

use ipso_core::{Builtin, Expr, ModulePath, ModuleUsage, Pattern, StringPart};
use ipso_rope::Rope;
use ipso_syntax::{Binop, ModuleName};
use paste::paste;
use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, Write},
    ops::Index,
    rc::Rc,
};
use typed_arena::Arena;

macro_rules! function1 {
    ($name:ident, $self:expr, $body:expr) => {{
        paste! {
            fn [<$name _code_0>]<'heap>(
                eval: &mut Interpreter<'_, '_, 'heap>,
                env: &'heap [Value<'heap>],
                arg: Value<'heap>,
            ) -> Value<'heap> {
                // This clippy lint is `allow`ed because requiring `$body` to be a function
                // seems to be the only way to make sure it's scope-checked.
                #[allow(clippy::redundant_closure_call)]
                $body(eval, env, arg)
            }

            let closure = $self.alloc(Object::StaticClosure {
                env: &[],
                body: StaticClosureBody([<$name _code_0>]),
            });
            closure
        }
    }};
}

macro_rules! function2 {
    ($name:ident, $self:expr, $body:expr) => {{
        function1!(
            $name,
            $self,
            (|eval: &mut Interpreter<'_, '_, 'heap>,
              env: &'heap [Value<'heap>],
              arg: Value<'heap>| {
                paste! {
                    fn [<$name _code_1>]<'heap>(
                        eval: &mut Interpreter<'_, '_, 'heap>,
                        env: &'heap [Value<'heap>],
                        arg: Value<'heap>,
                    ) -> Value<'heap> {
                        $body(eval, env, arg)
                    }
                    let env = eval.alloc_values({
                        let mut env = Vec::from(env);
                        env.push(arg);
                        env
                    });
                    eval.alloc(Object::StaticClosure {
                        env,
                        body: StaticClosureBody([<$name _code_1>]),
                    })
                }
            })
        )
    }};
}

macro_rules! function3 {
    ($name:ident, $self:expr, $body:expr) => {{
        function2!(
            $name,
            $self,
            (|eval: &mut Interpreter<'_, '_, 'heap>, env: &'heap [Value<'heap>], arg| {
                paste! {
                    fn [<$name _code_2>]<'heap>(
                        eval: &mut Interpreter<'_, '_, 'heap>,
                        env: &'heap [Value<'heap>],
                        arg: Value<'heap>,
                    ) -> Value<'heap> {
                        $body(eval, env, arg)
                    }
                    let env = eval.alloc_values({
                        let mut env = Vec::from(env);
                        env.push(arg);
                        env
                    });
                    eval.alloc(Object::StaticClosure {
                        env,
                        body: StaticClosureBody([<$name _code_2>]),
                    })
                }
            })
        )
    }};
}

#[derive(Clone)]
pub struct StaticClosureBody<'heap>(
    fn(&mut Interpreter<'_, '_, 'heap>, &'heap [Value<'heap>], Value<'heap>) -> Value<'heap>,
);

impl<'heap> Debug for StaticClosureBody<'heap> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StaticClosure()")
    }
}

#[derive(Clone)]
pub struct IOBody<'heap>(
    fn(&mut Interpreter<'_, '_, 'heap>, &'heap [Value<'heap>]) -> Value<'heap>,
);

impl<'heap> Debug for IOBody<'heap> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IO()")
    }
}

/// Equivalent to `Cow<[Value<'heap>]>`, but `push` is faster when going from
/// borrowed to owned.
#[derive(Debug)]
pub enum Env<'heap> {
    Borrowed(&'heap [Value<'heap>]),
    Owned(Vec<Value<'heap>>),
}

impl<'heap> Default for Env<'heap> {
    fn default() -> Self {
        Env::Owned(Default::default())
    }
}

impl<'heap> Env<'heap> {
    fn push(&mut self, value: Value<'heap>) {
        match self {
            Env::Borrowed(vs) => {
                let mut new_vs = Vec::with_capacity(vs.len() + 1);
                new_vs.extend_from_slice(vs);
                new_vs.push(value);
                *self = Env::Owned(new_vs);
            }
            Env::Owned(vs) => vs.push(value),
        }
    }

    fn len(&self) -> usize {
        match self {
            Env::Borrowed(vs) => vs.len(),
            Env::Owned(vs) => vs.len(),
        }
    }

    pub fn new() -> Self {
        Env::Owned(Vec::new())
    }
}

impl<'heap> From<&'heap [Value<'heap>]> for Env<'heap> {
    fn from(value: &'heap [Value<'heap>]) -> Self {
        Env::Borrowed(value)
    }
}

impl<'heap> Index<usize> for Env<'heap> {
    type Output = Value<'heap>;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Env::Borrowed(vs) => &vs[index],
            Env::Owned(vs) => &vs[index],
        }
    }
}

#[derive(Debug)]
pub enum Object<'heap> {
    String(&'heap str),
    Bytes(&'heap [u8]),
    Variant(usize, Value<'heap>),
    Array(&'heap [Value<'heap>]),
    Record(&'heap [Value<'heap>]),
    Closure {
        env: &'heap [Value<'heap>],
        arg: bool,
        body: Rc<Expr>,
    },
    StaticClosure {
        env: &'heap [Value<'heap>],
        body: StaticClosureBody<'heap>,
    },
    IO {
        env: &'heap [Value<'heap>],
        body: IOBody<'heap>,
    },
}

impl<'heap> Object<'heap> {
    pub fn unpack_array(&'heap self) -> &'heap [Value<'heap>] {
        match self {
            Object::Array(vals) => vals,
            val => panic!("expected array, got {:?}", val),
        }
    }

    pub fn perform_io<'io, 'ctx>(
        &'heap self,
        interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
    ) -> Value<'heap> {
        match self {
            Object::IO { env, body } => body.0(interpreter, env),
            val => panic!("expected io, got {:?}", val),
        }
    }

    pub fn unpack_string(&'heap self) -> &'heap str {
        match self {
            Object::String(str) => str,
            val => panic!("expected string, got {:?}", val),
        }
    }

    pub fn unpack_bytes(&'heap self) -> &'heap [u8] {
        match self {
            Object::Bytes(bs) => bs,
            val => panic!("expected bytes, got {:?}", val),
        }
    }

    pub fn unpack_variant(&'heap self) -> (&'heap usize, &'heap Value<'heap>) {
        match self {
            Object::Variant(tag, rest) => (tag, rest),
            val => panic!("expected variant, got {:?}", val),
        }
    }

    fn unpack_record(&'heap self) -> &'heap [Value] {
        match self {
            Object::Record(fields) => fields,
            val => panic!("expected record,got {:?}", val),
        }
    }

    pub fn apply<'io, 'ctx>(
        &'heap self,
        interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
        arg: Value<'heap>,
    ) -> Value<'heap> {
        match self {
            Object::Closure {
                env,
                arg: use_arg,
                body,
            } => {
                let mut env = Env::from(*env);
                if *use_arg {
                    env.push(arg);
                }
                interpreter.eval(&mut env, body)
            }
            Object::StaticClosure { env, body } => body.0(interpreter, env, arg),
            a => panic!("expected closure, got {:?}", a),
        }
    }

    pub fn render(&self) -> String {
        match self {
            Object::Closure {
                env: _,
                arg: _,
                body: _,
            } => String::from("<closure>"),
            Object::StaticClosure { env: _, body: _ } => String::from("<static builtin>"),
            Object::IO { env: _, body: _ } => String::from("<io>"),
            Object::String(s) => format!("{:?}", s),
            Object::Bytes(bs) => format!("{:?}", bs),
            Object::Array(items) => {
                let mut s = String::new();
                s.push_str("[ ");
                let mut items_iter = items.iter();
                match items_iter.next() {
                    None => {}
                    Some(item) => {
                        s.push_str(item.render().as_str());
                        for item in items_iter {
                            s.push_str(", ");
                            s.push_str(item.render().as_str());
                        }
                    }
                }
                s.push_str(" ]");
                s
            }
            Object::Record(items) => {
                let mut s = String::new();
                s.push_str("{ ");
                let mut items_iter = items.iter();
                match items_iter.next() {
                    None => {}
                    Some(item) => {
                        s.push_str(item.render().as_str());
                        for item in items_iter {
                            s.push_str(", ");
                            s.push_str(item.render().as_str());
                        }
                    }
                }
                s.push_str(" }");
                s
            }
            Object::Variant(tag, value) => {
                let mut s = String::new();
                s.push_str("ctor(");
                s.push_str(format!("{:?}", tag).as_str());
                s.push_str(", ");
                s.push_str(value.render().as_str());
                s.push(')');
                s
            }
        }
    }
}

impl<'heap> PartialEq for Object<'heap> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::Closure { env, arg, body } => match other {
                Object::Closure {
                    env: env2,
                    arg: arg2,
                    body: body2,
                } => env == env2 && arg == arg2 && body == body2,
                _ => false,
            },
            Object::StaticClosure { env, body } => match other {
                Object::StaticClosure {
                    env: env2,
                    body: body2,
                } => env == env2 && (body.0 as usize) == (body2.0 as usize),
                _ => false,
            },
            Object::IO { env, body } => match other {
                Object::IO {
                    env: env2,
                    body: body2,
                } => env == env2 && (body.0 as usize) == (body2.0 as usize),
                _ => false,
            },
            Object::String(s) => match other {
                Object::String(s2) => s == s2,
                _ => false,
            },
            Object::Bytes(bs) => match other {
                Object::Bytes(bs2) => bs == bs2,
                _ => false,
            },
            Object::Array(items) => match other {
                Object::Array(items2) => items == items2,
                _ => false,
            },
            Object::Record(items) => match other {
                Object::Record(items2) => items == items2,
                _ => false,
            },
            Object::Variant(tag, value) => match other {
                Object::Variant(tag2, value2) => tag == tag2 && value == value2,
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Value<'heap> {
    True,
    False,
    Int(u32),
    Char(char),
    Unit,

    Stdout,
    Stdin,

    Object(&'heap Object<'heap>),
}

impl<'heap> Value<'heap> {
    pub fn apply<'io, 'ctx>(
        &self,
        interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
        arg: Value<'heap>,
    ) -> Value<'heap> {
        self.unpack_object().apply(interpreter, arg)
    }

    pub fn perform_io<'io, 'ctx>(
        &self,
        interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
    ) -> Value<'heap> {
        self.unpack_object().perform_io(interpreter)
    }

    pub fn unpack_string(&self) -> &'heap str {
        self.unpack_object().unpack_string()
    }

    pub fn unpack_bytes(&self) -> &'heap [u8] {
        self.unpack_object().unpack_bytes()
    }

    pub fn unpack_array(&self) -> &'heap [Value<'heap>] {
        self.unpack_object().unpack_array()
    }

    pub fn unpack_variant(&self) -> (&'heap usize, &'heap Value<'heap>) {
        self.unpack_object().unpack_variant()
    }

    pub fn unpack_object(&self) -> &'heap Object<'heap> {
        match self {
            Value::Object(obj) => obj,
            val => panic!("expected object, got {:?}", val),
        }
    }

    pub fn unpack_bool(&self) -> bool {
        match self {
            Value::False => false,
            Value::True => true,
            val => panic!("expected bool, got {:?}", val),
        }
    }

    pub fn unpack_int(&self) -> u32 {
        match self {
            Value::Int(n) => *n,
            val => panic!("expected int, got {:?}", val),
        }
    }

    pub fn unpack_char(&self) -> char {
        match self {
            Value::Char(c) => *c,
            val => panic!("expected char, got {:?}", val),
        }
    }

    pub fn unpack_stdout(&self) {
        match self {
            Value::Stdout => (),
            val => panic!("expected stdout, got {:?}", val),
        }
    }

    pub fn unpack_stdin(&self) {
        match self {
            Value::Stdin => (),
            val => panic!("expected stdin, got {:?}", val),
        }
    }

    pub fn unpack_record(&self) -> &'heap [Value<'heap>] {
        self.unpack_object().unpack_record()
    }

    pub fn render(&self) -> String {
        match self {
            Value::True => String::from("true"),
            Value::False => String::from("false"),
            Value::Int(n) => format!("{:?}", n),
            Value::Char(c) => format!("{:?}", c),
            Value::Unit => String::from("()"),
            Value::Stdout => String::from("Stdout"),
            Value::Stdin => String::from("Stdin"),
            Value::Object(o) => o.render(),
        }
    }
}

impl<'heap> PartialEq for Value<'heap> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::True => matches!(other, Value::True),
            Value::False => matches!(other, Value::False),
            Value::Int(n) => match other {
                Value::Int(n2) => n == n2,
                _ => false,
            },
            Value::Char(c) => match other {
                Value::Char(c2) => c == c2,
                _ => false,
            },
            Value::Unit => matches!(other, Value::Unit),
            Value::Stdout => matches!(other, Value::Stdout),
            Value::Stdin => matches!(other, Value::Stdin),
            Value::Object(o1) => match other {
                Value::Object(o2) => o1.eq(o2),
                _ => false,
            },
        }
    }
}

pub struct Module {
    pub module_mapping: HashMap<ModulePath, ModuleUsage>,
    pub bindings: HashMap<String, Rc<Expr>>,
}

pub struct Interpreter<'io, 'ctx, 'heap> {
    stdin: &'io mut dyn BufRead,
    stdout: &'io mut dyn Write,
    bytes: &'heap Arena<u8>,
    values: &'heap Arena<Value<'heap>>,
    objects: &'heap Arena<Object<'heap>>,
    context: &'ctx HashMap<String, Rc<Expr>>,
    module_context: HashMap<ModulePath, Module>,
    module_unmapping: Vec<HashMap<ModuleName, ModulePath>>,
}

impl<'io, 'ctx, 'heap> Interpreter<'io, 'ctx, 'heap> {
    pub fn new(
        stdin: &'io mut dyn BufRead,
        stdout: &'io mut dyn Write,
        context: &'ctx HashMap<String, Rc<Expr>>,
        module_context: HashMap<ModulePath, Module>,
        bytes: &'heap Arena<u8>,
        values: &'heap Arena<Value<'heap>>,
        objects: &'heap Arena<Object<'heap>>,
    ) -> Self {
        Interpreter {
            stdin,
            stdout,
            context,
            module_context,
            module_unmapping: Vec::with_capacity(1),
            bytes,
            values,
            objects,
        }
    }

    pub fn alloc(&self, obj: Object<'heap>) -> Value<'heap> {
        Value::Object(self.objects.alloc(obj))
    }

    pub fn alloc_str(&self, s: &str) -> &'heap str {
        self.bytes.alloc_str(s)
    }

    pub fn alloc_bytes<I: IntoIterator<Item = u8>>(&self, s: I) -> &'heap [u8] {
        self.bytes.alloc_extend(s)
    }

    pub fn alloc_values<I: IntoIterator<Item = Value<'heap>>>(
        &self,
        vals: I,
    ) -> &'heap [Value<'heap>]
where {
        self.values.alloc_extend(vals)
    }

    pub fn eval_builtin(&self, name: &Builtin) -> Value<'heap> {
        match name {
            Builtin::PureIO => {
                function1!(
                    pure_io,
                    self,
                    |interpreter: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        fn pure_io_1<'io, 'ctx, 'heap>(
                            _: &mut Interpreter<'io, 'ctx, 'heap>,
                            env: &'heap [Value<'heap>],
                        ) -> Value<'heap> {
                            env[0]
                        }
                        let env = interpreter.alloc_values({
                            let mut env = Vec::from(env);
                            env.push(arg);
                            env
                        });
                        let closure = Object::IO {
                            env,
                            body: IOBody(pure_io_1),
                        };
                        interpreter.alloc(closure)
                    }
                )
            }
            Builtin::MapIO => {
                function2!(
                    map_io,
                    self,
                    |interpreter: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        fn map_io_2<'io, 'ctx, 'heap>(
                            interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
                            env: &'heap [Value<'heap>],
                        ) -> Value<'heap> {
                            let f = env[0];
                            let io_a = env[1];
                            let a = io_a.perform_io(interpreter); // type: a
                            f.apply(interpreter, a) // type: b
                        }
                        let env = interpreter.alloc_values({
                            let mut env = Vec::from(env);
                            env.push(arg);
                            env
                        });
                        let closure = Object::IO {
                            env,
                            body: IOBody(map_io_2),
                        };
                        interpreter.alloc(closure)
                    }
                )
            }
            Builtin::BindIO => {
                function2!(
                    bind_io,
                    self,
                    |interpreter: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        fn bind_io_2<'io, 'ctx, 'heap>(
                            interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
                            env: &'heap [Value<'heap>],
                        ) -> Value<'heap> {
                            let io_a = env[0];
                            let f = env[1];
                            let a = io_a.perform_io(interpreter); // type: a
                            let io_b = f.apply(interpreter, a); // type: IO b
                            io_b.perform_io(interpreter) // type: b
                        }
                        let env = interpreter.alloc_values({
                            let mut new_env = Vec::with_capacity(env.len() + 1);
                            new_env.extend_from_slice(env);
                            new_env.push(arg);
                            new_env
                        });
                        let closure = Object::IO {
                            env,
                            body: IOBody(bind_io_2),
                        };
                        interpreter.alloc(closure)
                    }
                )
            }
            Builtin::Trace => {
                function2!(
                    trace,
                    self,
                    |interpreter: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let _ = writeln!(interpreter.stdout, "trace: {}", env[0].render()).unwrap();
                        arg
                    }
                )
            }
            Builtin::ToUtf8 => {
                function1!(
                    to_utf8,
                    self,
                    |interpreter: &mut Interpreter<'_, '_, 'heap>,
                     _: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = arg.unpack_string();
                        interpreter.alloc(Object::Bytes(a.as_bytes()))
                    }
                )
            }
            Builtin::Stdout => Value::Stdout,
            Builtin::Stdin => Value::Stdin,
            Builtin::WriteStdout => {
                function2!(
                    write_stdout,
                    self,
                    |interpreter: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        fn write_stdout_2<'io, 'ctx, 'heap>(
                            interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
                            env: &'heap [Value<'heap>],
                        ) -> Value<'heap> {
                            // env[0] : Stdout
                            // env[1] : Bytes
                            let () = env[0].unpack_stdout();
                            let bs = env[1].unpack_bytes();
                            let _ = interpreter.stdout.write_all(bs).unwrap();
                            Value::Unit
                        }

                        let env = interpreter.alloc_values({
                            let mut env = Vec::from(env);
                            env.push(arg);
                            env
                        });
                        interpreter.alloc(Object::IO {
                            env,
                            body: IOBody(write_stdout_2),
                        })
                    }
                )
            }
            Builtin::FlushStdout => {
                fn flush_stdout<'io, 'ctx, 'heap>(
                    interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
                    env: &'heap [Value<'heap>],
                ) -> Value<'heap> {
                    // env[0] : Stdout
                    env[0].unpack_stdout();
                    interpreter.stdout.flush().unwrap();
                    Value::Unit
                }
                function1!(
                    flush_stdout,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let env = eval.alloc_values({
                            let mut env = Vec::from(env);
                            env.push(arg);
                            env
                        });
                        eval.alloc(Object::IO {
                            env,
                            body: IOBody(flush_stdout),
                        })
                    }
                )
            }
            Builtin::ReadLineStdin => {
                function1!(
                    readline_stdin,
                    self,
                    |interpreter: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        fn read_line_stdin_1<'io, 'ctx, 'heap>(
                            interpreter: &mut Interpreter<'io, 'ctx, 'heap>,
                            env: &'heap [Value<'heap>],
                        ) -> Value<'heap> {
                            // env[0] : Stdin
                            let () = env[0].unpack_stdin();
                            let mut str = String::new();
                            let _ = interpreter.stdin.read_line(&mut str).unwrap();
                            let str = interpreter.alloc_str(&str);
                            interpreter.alloc(Object::String(str))
                        }
                        let env = interpreter.alloc_values({
                            let mut env = Vec::from(env);
                            env.push(arg);
                            env
                        });
                        let closure = interpreter.alloc(Object::IO {
                            env,
                            body: IOBody(read_line_stdin_1),
                        });
                        closure
                    }
                )
            }
            Builtin::EqString => {
                function2!(
                    eq_string,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = env[0].unpack_string();
                        let b = arg.unpack_string();
                        if a == b {
                            Value::True
                        } else {
                            Value::False
                        }
                    }
                )
            }
            Builtin::EqInt => {
                function2!(
                    eq_int,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = env[0].unpack_int();
                        let b = arg.unpack_int();
                        if a == b {
                            Value::True
                        } else {
                            Value::False
                        }
                    }
                )
            }
            Builtin::LtInt => {
                function2!(
                    lt_int,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = env[0].unpack_int();
                        let b = arg.unpack_int();
                        if a < b {
                            Value::True
                        } else {
                            Value::False
                        }
                    }
                )
            }
            Builtin::ShowInt => {
                function1!(
                    show_int,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     _env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = arg.unpack_int();
                        let str = eval.alloc_str(&format!("{}", a));
                        eval.alloc(Object::String(str))
                    }
                )
            }
            Builtin::Subtract => {
                function2!(
                    subtract,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = env[0].unpack_int();
                        let b = arg.unpack_int();
                        Value::Int(a - b)
                    }
                )
            }
            Builtin::Add => {
                function2!(
                    add,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = env[0].unpack_int();
                        let b = arg.unpack_int();
                        Value::Int(a + b)
                    }
                )
            }
            Builtin::Multiply => {
                function2!(
                    multiply,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let a = env[0].unpack_int();
                        let b = arg.unpack_int();
                        Value::Int(a * b)
                    }
                )
            }
            Builtin::EqArray => {
                function3!(
                    eq_array,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let f = env[0];
                        let a = env[1].unpack_array();
                        let b = arg.unpack_array();

                        let mut acc = Value::True;
                        if a.len() == b.len() {
                            for (a, b) in a.iter().zip(b.iter()) {
                                let res = f.apply(eval, *a).apply(eval, *b).unpack_bool();
                                if !res {
                                    acc = Value::False;
                                    break;
                                }
                            }
                        } else {
                            acc = Value::False;
                        }
                        acc
                    }
                )
            }
            Builtin::LtArray => {
                function3!(
                    lt_array,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let lt = env[0];
                        let a = env[1].unpack_array();
                        let b = arg.unpack_array();

                        let mut ix = 0;
                        let a_len = a.len();
                        let b_len = b.len();
                        loop {
                            // the prefix of a matches the prefix of b
                            if ix < a_len {
                                if ix < b_len {
                                    let a_val = a[ix];
                                    let b_val = b[ix];
                                    if lt.apply(eval, a_val).apply(eval, b_val).unpack_bool() {
                                        ix += 1;
                                    } else {
                                        return Value::False;
                                    }
                                } else {
                                    // a is longer than b
                                    return Value::False;
                                }
                            } else if ix < b_len {
                                // a is shorter than b
                                return Value::True;
                            } else {
                                // a is the same length as b
                                return Value::False;
                            }
                        }
                    }
                )
            }
            Builtin::FoldlArray => {
                function3!(
                    foldl_array,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let f = env[0];
                        let z = env[1];
                        let arr = arg.unpack_array();

                        let mut acc = z;
                        for el in arr {
                            acc = f.apply(eval, acc).apply(eval, *el);
                        }
                        acc
                    }
                )
            }
            Builtin::GenerateArray => {
                function2!(
                    generate_array,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let len = env[0].unpack_int();
                        let f = arg;

                        let mut array = Vec::with_capacity(len as usize);
                        for ix in 0..len {
                            let ix = Value::Int(ix);
                            array.push(f.apply(eval, ix));
                        }

                        let array = eval.alloc_values(array);
                        eval.alloc(Object::Array(array))
                    }
                )
            }
            Builtin::LengthArray => {
                function1!(
                    length_array,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     _env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let arr = arg.unpack_array();

                        Value::Int(arr.len() as u32)
                    }
                )
            }
            Builtin::IndexArray => {
                function2!(
                    index_array,
                    self,
                    |_eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let ix = env[0].unpack_int() as usize;
                        let arr = arg.unpack_array();

                        arr[ix]
                    }
                )
            }
            Builtin::SliceArray => {
                function3!(
                    slice_array,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let start = env[0].unpack_int() as usize;
                        let len = env[1].unpack_int() as usize;
                        let arr = arg.unpack_array();

                        eval.alloc(Object::Array(&arr[start..start + len]))
                    }
                )
            }
            Builtin::FilterString => {
                function2!(
                    filter_string,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let predicate = env[0];
                        let string = arg.unpack_string();
                        let new_string: String = string
                            .chars()
                            .filter(|c| {
                                let c_val = Value::Char(*c);
                                predicate.apply(eval, c_val).unpack_bool()
                            })
                            .collect();
                        let str = eval.alloc_str(&new_string);
                        eval.alloc(Object::String(str))
                    }
                )
            }
            Builtin::EqChar => {
                function2!(
                    eq_char,
                    self,
                    |_: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let c1 = env[0].unpack_char();
                        let c2 = arg.unpack_char();
                        if c1 == c2 {
                            Value::True
                        } else {
                            Value::False
                        }
                    }
                )
            }
            Builtin::SplitString => {
                function2!(
                    split_string,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let c = env[0].unpack_char();
                        let s = arg.unpack_string();
                        let a =
                            eval.alloc_values(s.split(c).map(|s| eval.alloc(Object::String(s))));
                        eval.alloc(Object::Array(a))
                    }
                )
            }
            Builtin::FoldlString => {
                function3!(
                    foldl_string,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let f = env[0];
                        let mut acc = env[1];
                        let s = arg.unpack_string();
                        for c in s.chars() {
                            let c_value = Value::Char(c);
                            acc = f.apply(eval, acc).apply(eval, c_value);
                        }
                        acc
                    }
                )
            }
            Builtin::SnocArray => {
                function2!(
                    snoc_array,
                    self,
                    |eval: &mut Interpreter<'_, '_, 'heap>,
                     env: &'heap [Value<'heap>],
                     arg: Value<'heap>| {
                        let array = env[0].unpack_array();
                        let new_array = eval.alloc_values({
                            let mut new_array = Vec::from(array);
                            new_array.push(arg);
                            new_array
                        });
                        eval.alloc(Object::Array(new_array))
                    }
                )
            }
        }
    }

    pub fn eval_from_module(
        &mut self,
        env: &mut Env<'heap>,
        path: &ModulePath,
        binding: &str,
    ) -> Value<'heap> {
        let (expr, next_module_mapping) = match self.module_context.get(path) {
            None => panic!("no module found at {:?}", path),
            Some(module) => match module.bindings.get(binding) {
                None => panic!("{:?} not found in {:?}", binding, path),
                Some(expr) => (expr.clone(), module.module_mapping.clone()),
            },
        };
        self.module_unmapping.push(
            next_module_mapping
                .into_iter()
                .filter_map(|(module_path, module_usage)| {
                    let m_module_name = match module_usage {
                        ModuleUsage::All => module_path.get_module_name().cloned(),
                        ModuleUsage::Items(_) => module_path.get_module_name().cloned(),
                        ModuleUsage::Named(name) => Some(ModuleName(vec![name])),
                    };
                    m_module_name.map(|module_name| (module_name, module_path))
                })
                .collect(),
        );
        let res = self.eval(env, &expr);
        self.module_unmapping.pop();
        res
    }

    pub fn eval(&mut self, env: &mut Env<'heap>, expr: &Expr) -> Value<'heap> {
        let out = match expr {
            Expr::Var(ix) => env[env.len() - 1 - ix],
            Expr::EVar(n) => panic!("found EVar({:?})", n),
            Expr::Placeholder(n) => panic!("found Placeholder({:?})", n),
            Expr::Name(name) => {
                let body = match self.context.get(name) {
                    None => panic!("{:?} not in scope", name),
                    Some(body) => body,
                };
                self.eval(env, body)
            }
            Expr::Module(name, item) => {
                let path: ModulePath = self
                    .module_unmapping
                    .last()
                    .unwrap()
                    .get(name)
                    .unwrap()
                    .clone();
                self.eval_from_module(env, &path, item)
            }
            Expr::Builtin(name) => self.eval_builtin(name),

            Expr::App(a, b) => {
                let a = self.eval(env, a);
                let b = self.eval(env, b);
                a.apply(self, b)
            }
            Expr::Lam { arg, body } => {
                let env = match env {
                    Env::Borrowed(env) => env,
                    Env::Owned(env) => self.alloc_values(env.iter().copied()),
                };
                self.alloc(Object::Closure {
                    env,
                    arg: *arg,
                    body: body.clone(),
                })
            }

            Expr::Let { value, rest } => {
                let value = self.eval(env, value);
                env.push(value);
                self.eval(env, rest)
            }

            Expr::True => Value::True,
            Expr::False => Value::False,
            Expr::IfThenElse(cond, t, e) => {
                let cond = self.eval(env, cond);
                match cond {
                    Value::True => self.eval(env, t),
                    Value::False => self.eval(env, e),
                    cond => panic!("expected bool, got {:?}", cond),
                }
            }

            Expr::Int(n) => Value::Int(*n),

            Expr::Binop(op, a, b) => {
                let a = self.eval(env, a);
                let b = self.eval(env, b);
                match op {
                    Binop::Add => {
                        let a = a.unpack_int();
                        let b = b.unpack_int();
                        Value::Int(a + b)
                    }
                    Binop::Multiply => todo!("eval multiply {:?} {:?}", a, b),
                    Binop::Subtract => todo!("eval subtract {:?} {:?}", a, b),
                    Binop::Divide => todo!("eval divide {:?} {:?}", a, b),
                    Binop::Append => todo!("eval append {:?} {:?}", a, b),
                    Binop::Or => todo!("eval or {:?} {:?}", a, b),
                    Binop::And => todo!("eval and {:?} {:?}", a, b),
                    Binop::Eq => todo!("eval eq {:?} {:?}", a, b),
                    Binop::Neq => todo!("eval neq {:?} {:?}", a, b),
                    Binop::Gt => todo!("eval gt {:?} {:?}", a, b),
                    Binop::Gte => todo!("eval gte {:?} {:?}", a, b),
                    Binop::Lt => todo!("eval lt {:?} {:?}", a, b),
                    Binop::Lte => todo!("eval lte {:?} {:?}", a, b),
                }
            }

            Expr::Char(c) => Value::Char(*c),

            Expr::String(parts) => {
                let mut value = String::new();

                for part in parts {
                    match part {
                        StringPart::Expr(expr) => {
                            let s = self.eval(env, expr).unpack_string();
                            value.push_str(s);
                        }
                        StringPart::String(s) => value.push_str(s.as_str()),
                    }
                }
                let str = self.alloc_str(&value);
                self.alloc(Object::String(str))
            }

            Expr::Array(items) => {
                let items: Vec<Value> = items.iter().map(|item| self.eval(env, item)).collect();
                let items = self.alloc_values(items);
                self.alloc(Object::Array(items))
            }

            Expr::Extend(ev, value, rest) => {
                let ix = self.eval(env, ev).unpack_int();
                let value = self.eval(env, value);
                let rest = self.eval(env, rest);
                match rest.unpack_object() {
                    Object::Record(fields) => {
                        // assume: all stacks in fields are non-empty
                        let ix = ix as usize;
                        let mut record = Vec::with_capacity(fields.len() + 1);
                        record.extend_from_slice(&fields[0..ix]);
                        record.push(value);
                        record.extend_from_slice(&fields[ix..]);

                        debug_assert!(record.len() == fields.len() + 1);

                        let record = self.alloc_values(record);
                        self.alloc(Object::Record(record))
                    }
                    rest => panic!("expected record, got {:?}", rest),
                }
            }
            Expr::Record(fields) => {
                let mut record: Vec<Value<'heap>> = Vec::with_capacity(fields.len());
                let mut fields: Vec<(u32, Value<'heap>)> = fields
                    .iter()
                    .map(|(ev, field)| (self.eval(env, ev).unpack_int(), self.eval(env, field)))
                    .collect();
                fields.sort_by_key(|x| x.0);
                for (_index, field) in fields.into_iter() {
                    record.push(field);
                }

                let record = self.alloc_values(record);
                self.alloc(Object::Record(record))
            }
            Expr::Project(expr, index) => {
                let index = self.eval(env, index).unpack_int();
                let expr = self.eval(env, expr);
                match expr.unpack_object() {
                    Object::Record(fields) => fields[index as usize],
                    expr => panic!("expected record, got {:?}", expr),
                }
            }

            Expr::Variant(tag) => {
                let tag = self.eval(env, tag);
                let env = self.alloc_values(vec![tag]);
                fn code<'heap>(
                    interpreter: &mut Interpreter<'_, '_, 'heap>,
                    env: &'heap [Value<'heap>],
                    arg: Value<'heap>,
                ) -> Value<'heap> {
                    let tag = env[0].unpack_int() as usize;
                    interpreter.alloc(Object::Variant(tag, arg))
                }
                let closure = Object::StaticClosure {
                    env,
                    body: StaticClosureBody(code),
                };
                self.alloc(closure)
            }
            Expr::Embed(tag, rest) => {
                let tag = self.eval(env, tag).unpack_int() as usize;
                let rest = self.eval(env, rest);
                let (&old_tag, arg) = rest.unpack_variant();
                self.alloc(Object::Variant(
                    if tag <= old_tag { old_tag + 1 } else { old_tag },
                    *arg,
                ))
            }
            Expr::Case(expr, branches) => {
                let expr = self.eval(env, expr);
                let mut target: Option<&Expr> = None;

                for branch in branches {
                    match &branch.pattern {
                        Pattern::Name => {
                            env.push(expr);
                            target = Some(&branch.body);
                            break;
                        }
                        Pattern::Record { names, rest } => {
                            let fields = expr.unpack_record();
                            let mut extracted = Vec::with_capacity(names.len());
                            for name in names {
                                let ix = self.eval(env, name).unpack_int() as usize;
                                env.push(fields[ix]);
                                extracted.push(ix);
                            }
                            if *rest {
                                let mut leftover_fields = Rope::from_vec(fields);
                                extracted.sort_unstable();
                                for ix in extracted.iter().rev() {
                                    leftover_fields = leftover_fields.delete(*ix).unwrap();
                                }
                                let leftover_fields =
                                    self.alloc_values(leftover_fields.iter().copied());
                                let leftover_record = self.alloc(Object::Record(leftover_fields));
                                env.push(leftover_record);
                            }
                            target = Some(&branch.body);
                            break;
                        }
                        Pattern::Variant { tag: branch_tag } => {
                            let (tag, value) = expr.unpack_variant();
                            let branch_tag = self.eval(env, branch_tag).unpack_int() as usize;
                            if *tag == branch_tag {
                                env.push(*value);
                                target = Some(&branch.body);
                                break;
                            }
                        }
                        Pattern::Wildcard => {
                            target = Some(&branch.body);
                            break;
                        }
                    }
                }

                match target {
                    Some(target) => self.eval(env, target),
                    None => panic!("incomplete pattern match"),
                }
            }
            Expr::Unit => Value::Unit,
        };
        out
    }
}
