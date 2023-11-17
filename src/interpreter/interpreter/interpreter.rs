pub(super) mod type_;
pub(super) mod value;

use std::{collections::HashMap, fmt::Display};

use async_recursion::async_recursion;
use genawaiter::sync::Co; // TODO: replace with rc::Co

use crate::{
    app::graphics::Color,
    interpreter::{
        interpreter::interpreter::{
            type_::Type,
            value::{DisplayValue, ReprValue, Value},
        },
        lang::{BinaryOp, Expr, ExprKind, ShortCircuitOp, Stmt, StmtKind, UnaryOp, VarName},
    },
    source::{Located, Span},
};

#[derive(Clone)]
pub(super) struct InterpreterState<'file> {
    pub(super) env: Vars<'file>,
    pub(super) program_output: String,
}
impl InterpreterState<'_> {
    pub(super) fn new() -> Self {
        Self { env: Vars { scopes: Vec::new() }, program_output: String::new() }
    }
}

#[derive(Clone)]
pub(super) struct Vars<'file> {
    pub(super) scopes: Vec<HashMap<VarName, (Span<'file>, Option<Value>)>>,
}
impl<'file> Vars<'file> {
    fn lookup(&self, vname: &VarName) -> Option<&(Span<'file>, Option<Value>)> {
        for scope in self.scopes.iter().rev() {
            if let Some(result) = scope.get(vname) {
                return Some(result);
            }
        }
        None
    }
    fn lookup_mut(&mut self, vname: &VarName) -> Option<&mut (Span<'file>, Option<Value>)> {
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

    fn define_var(&mut self, vname: VarName, span: Span<'file>, initializer: Option<Value>) {
        self.scopes.last_mut().expect("define var when there are no scopes to define in").insert(vname, (span, initializer));
    }
}

pub(super) struct InterpretYield<'file> {
    pub(super) msg: String,
    pub(super) primary_highlight: Span<'file>,
    pub(super) secondary_highlights: Vec<(Span<'file>, Color)>,
    pub(super) substitutions: Vec<(Span<'file>, String)>,
    pub(super) state: InterpreterState<'file>,
}

pub(crate) struct RuntimeError<'file> {
    pub(crate) span: Span<'file>,
    pub(crate) kind: RuntimeErrorKind,
}
pub(crate) enum RuntimeErrorKind {
    VarUninitialized(VarName),
    VarDoesNotExist(VarName),
    InvalidTypeForShortCircuitOp(ShortCircuitOp, Type),
    InvalidTypesForBinaryOp(BinaryOp, Type, Type),
    InvalidTypeForUnaryOp(UnaryOp, Type),
    ExpectedBool(Type),
}
impl Display for RuntimeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeErrorKind::VarUninitialized(vn) => write!(f, "variable '{}' is uninitialized", vn),
            RuntimeErrorKind::VarDoesNotExist(vn) => write!(f, "variable '{}' does not exist", vn),
            RuntimeErrorKind::InvalidTypeForShortCircuitOp(op, ty) => write!(f, "invalid type '{ty}' to operator logical operator '{op}'"),
            RuntimeErrorKind::InvalidTypesForBinaryOp(op, lty, rty) => write!(f, "invalid types '{lty}' and '{rty}' to operator '{op}'"),
            RuntimeErrorKind::InvalidTypeForUnaryOp(op, ty) => write!(f, "invalid type '{ty}' to unary operator '{op}'"),
            RuntimeErrorKind::ExpectedBool(got_ty) => write!(f, "expected 'bool', got '{got_ty}'"),
        }
    }
}

type ICo<'file> = Co<InterpretYield<'file>>;
pub(super) async fn interpret<'file>(stmts: Vec<Stmt<'file>>, co: ICo<'file>) -> Result<(), RuntimeError<'file>> {
    interpret_statements(&mut InterpreterState::new(), stmts, &co).await
}

#[async_recursion]
async fn interpret_statements<'parent, 'parents: 'parent, 'file>(state: &mut InterpreterState<'file>, stmts: Vec<Stmt<'file>>, co: &ICo<'file>) -> Result<(), RuntimeError<'file>> {
    state.env.start_scope();
    for stmt in stmts {
        interpret_statement(state, stmt, co).await?;
    }
    state.env.end_scope();

    Ok(())
}

#[async_recursion]
async fn interpret_statement<'parent, 'parents: 'parent, 'file>(state: &mut InterpreterState<'file>, stmt: Stmt<'file>, co: &ICo<'file>) -> Result<(), RuntimeError<'file>> {
    match stmt.kind {
        StmtKind::Block(stmts) => interpret_statements(state, stmts, co).await,

        StmtKind::Expr(e) => {
            interpret_expr(state, &Vec::new(), e, co).await?;
            Ok(())
        }

        StmtKind::Print(v) => {
            let v_span = v.span;
            let v = interpret_expr(state, &Vec::new(), v, co).await?;
            co.yield_(InterpretYield {
                msg: format!("print value {}", ReprValue(&v)),
                primary_highlight: stmt.span,
                secondary_highlights: Vec::new(),
                state: state.clone(),
                substitutions: vec![(v_span, ReprValue(&v).to_string())],
            })
            .await;
            state.program_output += &DisplayValue(&v).to_string();
            state.program_output += "\n";
            Ok(())
        }

        StmtKind::Return(_) => todo!(),

        StmtKind::MakeVar(vname, None) => {
            co.yield_(InterpretYield {
                msg: format!("make uninitialized variable '{vname}'"),
                primary_highlight: stmt.span,
                secondary_highlights: Vec::new(),
                substitutions: Vec::new(),
                state: state.clone(),
            })
            .await;
            state.env.define_var(vname.clone(), stmt.span, None);
            Ok(())
        }

        StmtKind::MakeVar(vname, Some(initializer)) => {
            let initializer_span = initializer.span;
            let initializer = interpret_expr(state, &Vec::new(), initializer, co).await?;
            co.yield_(InterpretYield {
                msg: format!("make variable '{vname}' with initializer {}", ReprValue(&initializer)),
                primary_highlight: stmt.span,
                secondary_highlights: Vec::new(),
                substitutions: vec![(initializer_span, ReprValue(&initializer).to_string())],
                state: state.clone(),
            })
            .await;
            state.env.define_var(vname.clone(), stmt.span, Some(initializer));
            Ok(())
        }

        StmtKind::AssignVar(var, v) => {
            let v_span = v.span;
            let v = interpret_expr(state, &Vec::new(), v, co).await?;
            co.yield_(InterpretYield {
                msg: format!("assign variable '{var}' with value {}", ReprValue(&v)),
                primary_highlight: stmt.span,
                secondary_highlights: Vec::new(),
                substitutions: vec![(v_span, ReprValue(&v).to_string())],
                state: state.clone(),
            })
            .await;
            match state.env.lookup_mut(&var) {
                Some(v_place) => {
                    v_place.1 = Some(v);
                    Ok(())
                }
                None => Err(RuntimeError { span: stmt.span, kind: RuntimeErrorKind::VarDoesNotExist(var) }),
            }
        }

        StmtKind::If(if_span, cond, t, f) => {
            let cond_span = cond.span;
            let cond = interpret_expr(state, &Vec::new(), cond, co).await?;
            co.yield_(InterpretYield {
                msg: "check condition".to_string(),
                primary_highlight: if_span,
                secondary_highlights: Vec::new(),
                substitutions: vec![(cond_span, ReprValue(&cond).to_string())],
                state: state.clone(),
            })
            .await;
            match cond {
                Value::Bool(true) => interpret_statement(state, *t, co).await,
                Value::Bool(false) => {
                    if let Some(f) = f {
                        interpret_statement(state, *f, co).await?;
                    }
                    Ok(())
                }
                cond => Err(RuntimeError { span: cond_span, kind: RuntimeErrorKind::ExpectedBool(cond.type_()) }),
            }
        }

        StmtKind::While(while_span, cond_ast, body) => loop {
            let cond_value = interpret_expr(state, &Vec::new(), cond_ast.clone(), co).await?;
            co.yield_(InterpretYield {
                msg: "check condition".to_string(),
                primary_highlight: while_span,
                secondary_highlights: Vec::new(),
                substitutions: vec![(cond_ast.span, ReprValue(&cond_value).to_string())],
                state: state.clone(),
            })
            .await;
            match cond_value {
                Value::Bool(true) => {}
                Value::Bool(false) => break Ok(()),
                _ => break Err(RuntimeError { span: cond_ast.span, kind: RuntimeErrorKind::ExpectedBool(cond_value.type_()) }),
            }

            interpret_statement(state, (*body).clone(), co).await?;
        },
    }
}

fn add_substitution<'file>(substitutions: &[(Span<'file>, String)], (sp, thing): (Span<'file>, impl ToString)) -> Vec<(Span<'file>, String)> {
    let mut new_substitutions = substitutions.to_vec();
    new_substitutions.push((sp, thing.to_string()));
    new_substitutions
}
#[async_recursion]
async fn interpret_expr<'file: 'async_recursion, 'parent, 'parents>(
    state: &mut InterpreterState<'file>,
    substitutions: &Vec<(Span<'file>, String)>,
    e: Expr<'file>,
    co: &ICo<'file>,
) -> Result<Value, RuntimeError<'file>> {
    match e.kind {
        ExprKind::Var(vname) => {
            co.yield_(InterpretYield {
                msg: format!("read variable '{vname}'"),
                primary_highlight: e.span,
                secondary_highlights: Vec::new(),
                state: state.clone(),
                substitutions: substitutions.clone(),
            })
            .await;
            match state.env.lookup(&vname) {
                Some((_, Some(v))) => Ok(v.clone()),
                Some((_, None)) => Err(RuntimeError { span: e.span, kind: RuntimeErrorKind::VarUninitialized(vname) }),
                None => Err(RuntimeError { span: e.span, kind: RuntimeErrorKind::VarDoesNotExist(vname) }),
            }
        }
        ExprKind::Int(i) => Ok(Value::Int(i)),
        ExprKind::Float(f) => Ok(Value::Float(f)),
        ExprKind::String(s) => Ok(Value::String(s)),
        ExprKind::Bool(b) => Ok(Value::Bool(b)),
        ExprKind::Parenthesized(e) => Ok(interpret_expr(state, substitutions, *e, co).await?),
        ExprKind::Call(_, _) => {
            todo!()
        }
        ExprKind::ShortCircuitOp(left, Located(_, op), right) => {
            let left_span = left.span;
            let right_span = right.span;
            match op {
                ShortCircuitOp::Or => match interpret_expr(state, substitutions, *left, co).await? {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    left @ Value::Bool(false) => match interpret_expr(state, &[&**substitutions, &add_substitution(substitutions, (left_span, ReprValue(&left)))].concat(), *right, co).await? {
                        a @ Value::Bool(_) => Ok(a),
                        right => Err(RuntimeError { span: right_span, kind: RuntimeErrorKind::InvalidTypeForShortCircuitOp(op, right.type_()) }),
                    },
                    left => Err(RuntimeError { span: left_span, kind: RuntimeErrorKind::InvalidTypeForShortCircuitOp(op, left.type_()) }),
                },
                ShortCircuitOp::And => match interpret_expr(state, substitutions, *left, co).await? {
                    Value::Bool(false) => Ok(Value::Bool(false)),
                    left @ Value::Bool(true) => match interpret_expr(state, &add_substitution(substitutions, (left_span, ReprValue(&left))), *right, co).await? {
                        a @ Value::Bool(_) => Ok(a),
                        right => Err(RuntimeError { span: right_span, kind: RuntimeErrorKind::InvalidTypeForShortCircuitOp(op, right.type_()) }),
                    },
                    left => Err(RuntimeError { span: left_span, kind: RuntimeErrorKind::InvalidTypeForShortCircuitOp(op, left.type_()) }),
                },
            }
        }
        ExprKind::BinaryOp(left, Located(op_span, op), right) => {
            let left_span = left.span;
            let right_span = right.span;

            let left = interpret_expr(state, substitutions, *left, co).await?;
            let subs_with_left = add_substitution(substitutions, (left_span, ReprValue(&left)));
            let right = interpret_expr(state, &subs_with_left, *right, co).await?;
            let subs_with_right = add_substitution(&subs_with_left, (right_span, ReprValue(&right)));

            macro_rules! comparison {
                    ($op:tt) => {
                        match (left, right) {
                            (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool(i1 $op i2)),
                            (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool(f1 $op f2)),
                            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 $op s2)),
                            (Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 $op b2)),
                            (left, right) => Err(RuntimeError { span: op_span, kind: RuntimeErrorKind::InvalidTypesForBinaryOp(op, left.type_(), right.type_()) }),
                        }
                    };
                }

            co.yield_(InterpretYield {
                msg: format!("evaluate operation '{}'", op),
                primary_highlight: op_span,
                secondary_highlights: Vec::new(),
                substitutions: subs_with_right,
                state: state.clone(),
            })
            .await;
            match op {
                BinaryOp::Equal => comparison!(==),
                BinaryOp::NotEqual => comparison!(!=),
                BinaryOp::Greater => comparison!(>),
                BinaryOp::GreaterEqual => comparison!(>=),
                BinaryOp::Less => comparison!(<),
                BinaryOp::LessEqual => comparison!(<=),

                BinaryOp::Add => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 + i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 + f2)),
                    (Value::String(s1), Value::String(s2)) => Ok(Value::String(s1 + &s2)),
                    (left, right) => Err(RuntimeError { span: op_span, kind: RuntimeErrorKind::InvalidTypesForBinaryOp(op, left.type_(), right.type_()) }),
                },
                BinaryOp::Subtract => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 - i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 - f2)),
                    (left, right) => Err(RuntimeError { span: op_span, kind: RuntimeErrorKind::InvalidTypesForBinaryOp(op, left.type_(), right.type_()) }),
                },
                BinaryOp::Multiply => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 * i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 * f2)),
                    (left, right) => Err(RuntimeError { span: op_span, kind: RuntimeErrorKind::InvalidTypesForBinaryOp(op, left.type_(), right.type_()) }),
                },
                BinaryOp::Divide => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 / i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 / f2)),
                    (left, right) => Err(RuntimeError { span: op_span, kind: RuntimeErrorKind::InvalidTypesForBinaryOp(op, left.type_(), right.type_()) }),
                },
                BinaryOp::Modulo => match (left, right) {
                    (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 % i2)),
                    (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 % f2)),
                    (left, right) => Err(RuntimeError { span: op_span, kind: RuntimeErrorKind::InvalidTypesForBinaryOp(op, left.type_(), right.type_()) }),
                },
            }
        }
        ExprKind::UnaryOp(Located(operator_span, operator), operand) => {
            let operand_span = operand.span;
            let operand = interpret_expr(state, substitutions, *operand, co).await?;
            co.yield_(InterpretYield {
                msg: format!("evaluate operation '{}'", operator),
                primary_highlight: operator_span,
                secondary_highlights: Vec::new(),
                substitutions: add_substitution(substitutions, (operand_span, ReprValue(&operand))),
                state: state.clone(),
            })
            .await;
            match operator {
                UnaryOp::NumericNegate => match operand {
                    Value::Int(i) => Ok(Value::Int(-i)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    operand => Err(RuntimeError { span: operator_span, kind: RuntimeErrorKind::InvalidTypeForUnaryOp(operator, operand.type_()) }),
                },
                UnaryOp::LogicalNegate => match operand {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    operand => Err(RuntimeError { span: operator_span, kind: RuntimeErrorKind::InvalidTypeForUnaryOp(operator, operand.type_()) }),
                },
            }
        }
    }
}
