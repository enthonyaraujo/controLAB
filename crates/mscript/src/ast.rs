use crate::Span;

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    LeftDiv,
    Power,
    DotMul,
    DotDiv,
    DotLeftDiv,
    DotPower,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: Kind,
    pub span: Span,
    pub depth: usize,
}

#[derive(Clone, Debug)]
pub enum Kind {
    Number(f64),
    Imaginary(f64),
    Text(String),
    Variable(String),
    All,
    End,
    Not(Box<Expr>),
    Unary(bool, Box<Expr>),
    Binary(Operator, Box<Expr>, Box<Expr>),
    Range(Box<Expr>, Option<Box<Expr>>, Box<Expr>),
    Transpose(bool, Box<Expr>),
    Matrix(Vec<Vec<Expr>>),
    Apply(String, Vec<Expr>),
}

pub struct Statement {
    pub name: Option<String>,
    pub targets: Option<Vec<Option<String>>>,
    pub indices: Option<Vec<Expr>>,
    pub expr: Expr,
    pub silent: bool,
}
