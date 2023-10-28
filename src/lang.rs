use num_bigint::BigInt;

use std::fmt::Display;

#[derive(Eq, PartialEq, Hash, Clone)]
pub(crate) struct VarName(pub(crate) String);
impl Display for VarName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

pub(crate) enum UnaryOp {
    NumericNegate,
    LogicalNegate,
}
pub(crate) enum BinaryOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}
pub(crate) enum ShortCircuitOp {
    Or,
    And,
}
pub(crate) enum Expr {
    Var(VarName),

    Int(BigInt),
    Float(f64),
    String(String),
    Bool(bool),

    Parenthesized(Box<Expr>),

    Call(Box<Expr>, Vec<Expr>),

    ShortCircuitOp(Box<Expr>, ShortCircuitOp, Box<Expr>),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
}

pub(crate) enum Statement {
    Block(Vec<Statement>),
    Expr(Expr),
    Print(Expr),
    Return(Expr),
    MakeVar(VarName, Option<Expr>),
    AssignVar(VarName, Expr),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    While(Expr, Box<Statement>),
}
