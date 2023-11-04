mod type_;
mod value;

use std::{collections::HashMap, fmt::Display};

use async_recursion::async_recursion;
use genawaiter::sync::Co;

use crate::{
    interpreter::{
        interpreter::interpreter::{type_::Type, value::Value},
        lang::{BinaryOp, Expr, ExprKind, ShortCircuitOp, Stmt, StmtKind, UnaryOp, VarName},
    },
    source::{Located, Span},
};

#[derive(Clone)]
pub(super) struct InterpreterState {
    pub(super) env: Vars,
    pub(super) program_output: String,
}
impl InterpreterState {
    pub(super) fn new() -> Self {
        Self { env: Vars { scopes: Vec::new() }, program_output: String::new() }
    }
}

#[derive(Clone)]
pub(super) struct Vars {
    pub(super) scopes: Vec<HashMap<VarName, Option<Value>>>,
}
impl Vars {
    fn lookup(&self, vname: &VarName) -> Option<&Option<Value>> {
        for scope in self.scopes.iter().rev() {
            if let Some(result) = scope.get(vname) {
                return Some(result);
            }
        }
        None
    }
    fn lookup_mut(&mut self, vname: &VarName) -> Option<&mut Option<Value>> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(result) = scope.get_mut(vname) {
                return Some(result);
            }
        }
        None
    }

    fn start_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_var(&mut self, vname: VarName, initializer: Option<Value>) {
        self.scopes.last_mut().expect("define var when there are no scopes to define in").insert(vname, initializer);
    }
}

pub(super) struct InterpretYield<'file> {
    pub(super) msg: String,
    pub(super) highlight: Span<'file>,
    pub(super) state: InterpreterState,
}

pub(crate) enum RuntimeError<'file> {
    VarUninitialized(Span<'file>, VarName),
    VarDoesNotExist(Span<'file>, VarName),
    InvalidTypeForShortCircuitOp(Span<'file>, ShortCircuitOp, Type),
    InvalidTypesForBinaryOp(Span<'file>, BinaryOp, Type, Type),
    InvalidTypeForUnaryOp(Span<'file>, UnaryOp, Type),
    ExpectedBool(Span<'file>, Type),
}
impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::VarUninitialized(_, vn) => write!(f, "variable '{}' is uninitialized", vn),
            RuntimeError::VarDoesNotExist(_, vn) => write!(f, "variable '{}' does not exist", vn),
            RuntimeError::InvalidTypeForShortCircuitOp(_, _, _) => todo!(),
            RuntimeError::InvalidTypesForBinaryOp(_, _, _, _) => todo!(),
            RuntimeError::InvalidTypeForUnaryOp(_, _, _) => todo!(),
            RuntimeError::ExpectedBool(_, _) => todo!(),
        }
    }
}

type ICo<'file> = Co<InterpretYield<'file>>;
pub(super) async fn interpret<'file>(stmts: Vec<Stmt<'file>>, co: ICo<'file>) -> Result<(), RuntimeError<'file>> {
    interpret_statements(&mut InterpreterState::new(), stmts, &co).await
}

#[async_recursion]
async fn interpret_statements<'parent, 'parents: 'parent, 'file>(state: &mut InterpreterState, stmts: Vec<Stmt<'file>>, co: &ICo<'file>) -> Result<(), RuntimeError<'file>> {
    state.env.start_scope();
    for stmt in stmts {
        interpret_statement(state, stmt, co).await?;
    }
    state.env.end_scope();

    Ok(())
}

#[async_recursion]
async fn interpret_statement<'parent, 'parents: 'parent, 'file>(state: &mut InterpreterState, stmt: Stmt<'file>, co: &ICo<'file>) -> Result<(), RuntimeError<'file>> {
    match stmt.kind {
        StmtKind::Block(stmts) => interpret_statements(state, stmts, co).await,

        StmtKind::Expr(e) => {
            interpret_expr(state, e, co).await?;
            Ok(())
        }

        StmtKind::Print(v) => {
            let v = interpret_expr(state, v, co).await?;
            co.yield_(InterpretYield { msg: format!("print value '{v}'"), highlight: stmt.span, state: state.clone() }).await;
            state.program_output += &v.to_string();
            state.program_output += "\n";
            Ok(())
        }

        StmtKind::Return(_) => todo!(),

        StmtKind::MakeVar(vname, None) => {
            co.yield_(InterpretYield { msg: format!("make uninitialized variable '{vname}'"), highlight: stmt.span, state: state.clone() }).await;
            state.env.define_var(vname.clone(), None);
            Ok(())
        }

        StmtKind::MakeVar(vname, Some(initializer)) => {
            let initializer = interpret_expr(state, initializer, co).await?;
            co.yield_(InterpretYield { msg: format!("make variable '{vname}' with initializer {initializer}"), highlight: stmt.span, state: state.clone() }).await;
            state.env.define_var(vname.clone(), Some(initializer));
            Ok(())
        }

        StmtKind::AssignVar(var, v) => {
            let v = interpret_expr(state, v, co).await?;
            co.yield_(InterpretYield { msg: format!("assign variable '{var}' with value {v}"), highlight: stmt.span, state: state.clone() }).await;
            match state.env.lookup_mut(&var) {
                Some(v_place) => {
                    *v_place = Some(v);
                    Ok(())
                }
                None => {
                    eprintln!("error: variable '{var}' does not exist");
                    Err(RuntimeError::VarDoesNotExist(stmt.span, var))
                }
            }
        }

        StmtKind::If(if_span, cond, t, f) => {
            let cond_span = cond.span;
            let cond = interpret_expr(state, cond, co).await?;
            co.yield_(InterpretYield { msg: "check condition".to_string(), highlight: if_span, state: state.clone() }).await;
            match cond {
                Value::Bool(true) => interpret_statement(state, *t, co).await,
                Value::Bool(false) => {
                    if let Some(f) = f {
                        interpret_statement(state, *f, co).await?;
                    }
                    Ok(())
                }
                cond => Err(RuntimeError::ExpectedBool(cond_span, cond.type_())),
            }
        }

        StmtKind::While(while_span, cond, body) => loop {
            let cond_value = interpret_expr(state, cond.clone(), co).await?;
            co.yield_(InterpretYield { msg: "check condition".to_string(), highlight: while_span, state: state.clone() }).await;
            match cond_value {
                Value::Bool(true) => {}
                Value::Bool(false) => break Ok(()),
                _ => break Err(RuntimeError::ExpectedBool(cond.span, cond_value.type_())),
            }

            interpret_statement(state, (*body).clone(), co).await?;
        },
    }
}

#[async_recursion]
async fn interpret_expr<'file: 'async_recursion, 'parent, 'parents>(state: &mut InterpreterState, e: Expr<'file>, co: &ICo<'file>) -> Result<Value, RuntimeError<'file>> {
    match e.kind {
        ExprKind::Var(vname) => {
            co.yield_(InterpretYield { msg: format!("read variable '{vname}'"), highlight: e.span, state: state.clone() }).await;
            match state.env.lookup(&vname) {
                Some(Some(v)) => Ok(v.clone()),
                Some(None) => Err(RuntimeError::VarUninitialized(e.span, vname)),
                None => Err(RuntimeError::VarDoesNotExist(e.span, vname)),
            }
        }
        ExprKind::Int(i) => Ok(Value::Int(i)),
        ExprKind::Float(f) => Ok(Value::Float(f)),
        ExprKind::String(s) => Ok(Value::String(s)),
        ExprKind::Bool(b) => Ok(Value::Bool(b)),
        ExprKind::Parenthesized(e) => Ok(interpret_expr(state, *e, co).await?),
        ExprKind::Call(_, _) => {
            todo!()
        }
        ExprKind::ShortCircuitOp(left, Located(_, op), right) => {
            let left_span = left.span;
            let right_span = right.span;
            match op {
                ShortCircuitOp::Or => match interpret_expr(state, *left, co).await? {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    Value::Bool(false) => match interpret_expr(state, *right, co).await? {
                        a @ Value::Bool(_) => Ok(a),
                        right => Err(RuntimeError::InvalidTypeForShortCircuitOp(right_span, op, right.type_())),
                    },
                    left => Err(RuntimeError::InvalidTypeForShortCircuitOp(left_span, op, left.type_())),
                },
                ShortCircuitOp::And => match interpret_expr(state, *left, co).await? {
                    Value::Bool(false) => Ok(Value::Bool(false)),
                    Value::Bool(true) => match interpret_expr(state, *right, co).await? {
                        a @ Value::Bool(_) => Ok(a),
                        right => Err(RuntimeError::InvalidTypeForShortCircuitOp(right_span, op, right.type_())),
                    },
                    left => Err(RuntimeError::InvalidTypeForShortCircuitOp(left_span, op, left.type_())),
                },
            }
        }
        ExprKind::BinaryOp(left, Located(op_span, op), right) => {
            let left = interpret_expr(state, *left, co).await?;
            let right = interpret_expr(state, *right, co).await?;

            macro_rules! comparison {
                    ($op:tt) => {
                        match (left, right) {
                            (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool(i1 $op i2)),
                            (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool(f1 $op f2)),
                            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 $op s2)),
                            (Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 $op b2)),
                            (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(op_span, op, left.type_(), right.type_())),
                        }
                    };
                }

            co.yield_(InterpretYield { msg: "evaluate operation".to_string(), highlight: op_span, state: state.clone() }).await; // TODO: put operator in quotes in message
            match op {
                BinaryOp::Equal => {
                    comparison!(==)
                }
                BinaryOp::NotEqual => {
                    comparison!(!=)
                }
                BinaryOp::Greater => {
                    comparison!(>)
                }
                BinaryOp::GreaterEqual => {
                    comparison!(>=)
                }
                BinaryOp::Less => {
                    comparison!(<)
                }
                BinaryOp::LessEqual => {
                    comparison!(<=)
                }
                BinaryOp::Add => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 + i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 + f2)),
                    (Value::String(s1), Value::String(s2)) => Ok(Value::String(s1 + &s2)),
                    (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(op_span, op, left.type_(), right.type_())),
                },
                BinaryOp::Subtract => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 - i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 - f2)),
                    (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(op_span, op, left.type_(), right.type_())),
                },
                BinaryOp::Multiply => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 * i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 * f2)),
                    (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(op_span, op, left.type_(), right.type_())),
                },
                BinaryOp::Divide => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 / i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 / f2)),
                    (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(op_span, op, left.type_(), right.type_())),
                },
                BinaryOp::Modulo => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 % i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 % f2)),
                    (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(op_span, op, left.type_(), right.type_())),
                },
            }
        }
        ExprKind::UnaryOp(Located(operator_span, operator), operand) => {
            let operand = interpret_expr(state, *operand, co).await?;
            co.yield_(InterpretYield { msg: "evaluate operation".to_string(), highlight: operator_span, state: state.clone() }).await;
            match operator {
                UnaryOp::NumericNegate => match operand {
                    Value::Int(i) => Ok(Value::Int(-i)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    operand => Err(RuntimeError::InvalidTypeForUnaryOp(operator_span, operator, operand.type_())),
                },
                UnaryOp::LogicalNegate => match operand {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    operand => Err(RuntimeError::InvalidTypeForUnaryOp(operator_span, operator, operand.type_())),
                },
            }
        }
    }
}
