use crate::syntax;

#[derive(Debug, PartialEq, Eq)]
pub enum Pattern {
    Name,
    Record { names: usize, rest: bool },
    Variant { name: String },
    Wildcard,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Branch {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StringPart {
    String(String),
    Expr(Expr),
}

#[derive(Debug, PartialEq, Eq)]
pub struct EVar(usize);

pub struct Evidence(Vec<(Constraint, Option<Expr>)>);

pub enum Constraint {
    HasField {
        field: String,
        actual: syntax::Type<usize>,
    },
}

impl Evidence {
    pub fn new() -> Self {
        Evidence(Vec::new())
    }
    pub fn fresh_evar(&mut self, constraint: Constraint) -> EVar {
        let ix = self.0.len();
        self.0.push((constraint, None));
        EVar(ix)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Var(usize),
    Name(String),

    App(Box<Expr>, Box<Expr>),
    Lam {
        arg: Pattern,
        body: Box<Expr>,
    },

    True,
    False,
    IfThenElse(Box<Expr>, Box<Expr>, Box<Expr>),

    Int(u32),

    Binop(syntax::Binop, Box<Expr>, Box<Expr>),

    Char(char),

    String(Vec<StringPart>),

    Array(Vec<Expr>),

    Append(Vec<Expr>, Box<Expr>),
    Record {
        fields: Vec<Expr>,
        rest: Option<Box<Expr>>,
    },
    Project(Box<Expr>, EVar),

    Variant(EVar, Box<Expr>),
    Case(Box<Expr>, Vec<Branch>),
    Unit,
}

impl Expr {
    pub fn mk_app(a: Expr, b: Expr) -> Expr {
        Expr::App(Box::new(a), Box::new(b))
    }

    pub fn mk_lam(arg: Pattern, body: Expr) -> Expr {
        Expr::Lam {
            arg,
            body: Box::new(body),
        }
    }

    pub fn mk_ifthenelse(x: Expr, y: Expr, z: Expr) -> Expr {
        Expr::IfThenElse(Box::new(x), Box::new(y), Box::new(z))
    }

    pub fn mk_record(fields: Vec<Expr>, rest: Option<Expr>) -> Expr {
        Expr::Record {
            fields,
            rest: rest.map(|x| Box::new(x)),
        }
    }

    pub fn mk_project(expr: Expr, offset: EVar) -> Expr {
        Expr::Project(Box::new(expr), offset)
    }

    pub fn mk_variant(tag: EVar, expr: Expr) -> Expr {
        Expr::Variant(tag, Box::new(expr))
    }

    pub fn mk_binop(op: syntax::Binop, a: Expr, b: Expr) -> Expr {
        Expr::Binop(op, Box::new(a), Box::new(b))
    }

    pub fn mk_case(expr: Expr, branches: Vec<Branch>) -> Expr {
        Expr::Case(Box::new(expr), branches)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeSig {
    pub ty_vars: Vec<syntax::Kind>,
    pub body: syntax::Type<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClassMember {
    name: String,
    sig: TypeSig,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Declaration {
    Definition {
        name: String,
        sig: TypeSig,
        body: Expr,
    },
    TypeAlias {
        name: String,
        args: Vec<String>,
        body: syntax::Type<usize>,
    },
    Import {
        module: String,
        name: Option<String>,
    },
    FromImport {
        module: String,
        name: syntax::Names,
    },
    Class {
        ty_vars: Vec<syntax::Kind>,
        supers: Vec<syntax::Type<usize>>,
        name: String,
        args: Vec<usize>,
        members: Vec<ClassMember>,
    },
    Instance {
        ty_vars: Vec<syntax::Kind>,
        assumes: Vec<syntax::Type<usize>>,
        head: syntax::Type<usize>,
        dict: Expr,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub decls: Vec<Declaration>,
}
