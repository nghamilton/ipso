use crate::core::{EVar, Expr, StringPart};
use crate::syntax::Binop;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Value {
    Closure {
        env: Vec<Value>,
        arg: bool,
        body: Expr,
    },
    True,
    False,
    Int(u32),
    Char(char),
    String(String),
    Array(Vec<Value>),
    Record(Vec<Value>),
}

struct Interpreter {
    context: HashMap<String, Expr>,
    bound_vars: Vec<Value>,
    evidence: Vec<Value>,
}

impl Interpreter {
    pub fn eval(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Var(ix) => self.bound_vars[self.bound_vars.len() - 1 - ix].clone(),
            Expr::Name(name) => {
                let next: Expr = self.context.get(name).unwrap().clone();
                self.eval(&next)
            }

            Expr::App(a, b) => {
                let a = self.eval(a);
                let b = self.eval(b);
                match a {
                    Value::Closure { env, arg, body } => {
                        self.bound_vars = env;
                        if arg {
                            self.bound_vars.push(b);
                        }
                        self.eval(&body)
                    }
                    a => panic!("expected closure, got {:?}", a),
                }
            }
            Expr::Lam { arg, body } => Value::Closure {
                env: self.bound_vars.clone(),
                arg: *arg,
                body: (**body).clone(),
            },

            Expr::True => Value::True,
            Expr::False => Value::False,
            Expr::IfThenElse(cond, t, e) => {
                let cond = self.eval(cond);
                match cond {
                    Value::True => self.eval(t),
                    Value::False => self.eval(e),
                    cond => panic!("expected bool, got {:?}", cond),
                }
            }

            Expr::Int(n) => Value::Int(*n),

            Expr::Binop(op, a, b) => {
                let a = self.eval(a);
                let b = self.eval(b);
                match op {
                    Binop::Add => todo!(),
                    Binop::Multiply => todo!(),
                    Binop::Subtract => todo!(),
                    Binop::Divide => todo!(),
                    Binop::Append => todo!(),
                    Binop::Or => todo!(),
                    Binop::And => todo!(),
                    Binop::Eq => todo!(),
                    Binop::Neq => todo!(),
                    Binop::Gt => todo!(),
                    Binop::Gte => todo!(),
                    Binop::Lt => todo!(),
                    Binop::Lte => todo!(),
                }
            }

            Expr::Char(c) => Value::Char(*c),

            Expr::String(parts) => {
                let mut value = String::new();
                for part in parts {
                    match part {
                        StringPart::Expr(expr) => {
                            let expr = self.eval(expr);
                            match expr {
                                Value::String(s) => value.push_str(s.as_str()),
                                expr => panic!("expected string, got {:?}", expr),
                            }
                        }
                        StringPart::String(s) => value.push_str(s.as_str()),
                    }
                }
                Value::String(value)
            }

            Expr::Array(items) => {
                let items = items.iter().map(|item| self.eval(item)).collect();
                Value::Array(items)
            }

            Expr::Extend(ev, value, rest) => {
                let value = self.eval(value);
                let rest = self.eval(rest);
                match rest {
                    Value::Record(fields) => match &self.evidence[ev.0] {
                        Value::Int(ix) => {
                            // assume: all stacks in fields are non-empty
                            let ix = *ix as usize;
                            let mut record = Vec::with_capacity(fields.len() + 1);
                            record.extend_from_slice(&fields[0..ix]);
                            record.push(value);
                            record.extend_from_slice(&fields[ix..]);

                            debug_assert!(record.len() == fields.len() + 1);

                            Value::Record(record)
                        }
                        evidence => panic!("expected int, got {:?}", evidence),
                    },
                    rest => panic!("expected record, got {:?}", rest),
                }
            }
            Expr::Record(fields) => {
                let mut record: Vec<Value> = Vec::with_capacity(fields.len());
                let fields: Vec<(EVar, Value)> = fields
                    .iter()
                    .map(|(ev, field)| (*ev, self.eval(field)))
                    .collect();
                for (ev, field) in fields.into_iter().rev() {
                    let index = match &self.evidence[ev.0] {
                        Value::Int(ix) => *ix as usize,
                        evidence => panic!("expected int, got {:?}", evidence),
                    };
                    record.insert(index, field);
                }
                Value::Record(record)
            }
            Expr::Project(expr, index) => todo!(),

            Expr::Variant(tag, value) => todo!(),
            Expr::Case(expr, branches) => todo!(),
            Expr::Unit => todo!(),
        }
    }
}
