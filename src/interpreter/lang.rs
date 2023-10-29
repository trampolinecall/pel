use num_bigint::BigInt;

use std::fmt::Display;

use crate::source::{Span, Located};

#[derive(Eq, PartialEq, Hash, Clone)]
pub(crate) struct VarName(pub(crate) String);
impl Display for VarName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub(crate) enum UnaryOp {
    NumericNegate,
    LogicalNegate,
}
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
pub(crate) enum ShortCircuitOp {
    Or,
    And,
}
#[derive(Clone)]
pub(crate) struct Expr<'file> {
    pub(crate) kind: ExprKind<'file>,
    pub(crate) span: Span<'file>,
}
#[derive(Clone)]
pub(crate) enum ExprKind<'file> {
    Var(VarName),

    Int(BigInt),
    Float(f64),
    String(String),
    Bool(bool),

    Parenthesized(Box<Expr<'file>>),

    Call(Box<Expr<'file>>, Vec<Expr<'file>>),

    ShortCircuitOp(Box<Expr<'file>>, Located<'file, ShortCircuitOp>, Box<Expr<'file>>),
    BinaryOp(Box<Expr<'file>>, Located<'file, BinaryOp>, Box<Expr<'file>>),
    UnaryOp(Located<'file, UnaryOp>, Box<Expr<'file>>),
}

#[derive(Clone)]
pub(crate) struct Stmt<'file> {
    pub(crate) kind: StmtKind<'file>,
    pub(crate) span: Span<'file>,
}
#[derive(Clone)]
pub(crate) enum StmtKind<'file> {
    Block(Vec<Stmt<'file>>),
    Expr(Expr<'file>),
    Print(Expr<'file>),
    Return(Expr<'file>),
    MakeVar(VarName, Option<Expr<'file>>),
    AssignVar(VarName, Expr<'file>),
    If(Expr<'file>, Box<Stmt<'file>>, Option<Box<Stmt<'file>>>),
    While(Expr<'file>, Box<Stmt<'file>>),
}
