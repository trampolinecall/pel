use std::{collections::HashMap, fmt::Display, future::Future};

use genawaiter::sync::{Co, Gen};
use num_bigint::BigInt;

use crate::{
    interpreter::lang::{BinaryOp, Expr, ExprKind, ShortCircuitOp, Stmt, StmtKind, UnaryOp, VarName},
    source::Span,
    visualizer::widgets::{code_view::CodeView, either::Either, label::Label, responds_to_keyboard::RespondsToKeyboard, vsplit::VSplit, Widget},
};

#[derive(Default, Clone)]
struct Vars {
    scopes: Vec<HashMap<VarName, Option<Value>>>,
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
pub(crate) struct Interpreter<'file, F: Future<Output = Result<(), RuntimeError<'file>>>> {
    state: InterpreterState<'file>,

    interpret_generator: Gen<(Stmt<'file>, Vars), (), F>,
}
enum InterpreterState<'file> {
    NotStarted,
    AboutToExecute(Stmt<'file>, Vars),
    Finished(Result<(), RuntimeError<'file>>),
}

pub(crate) fn new_interpreter(stmts: Vec<Stmt>) -> Interpreter<impl Future<Output = Result<(), RuntimeError>>> {
    let gen = Gen::new(move |co| interpret(stmts, co));
    Interpreter { state: InterpreterState::NotStarted, interpret_generator: gen }
}
impl<'file, F: Future<Output = Result<(), RuntimeError<'file>>>> Interpreter<'file, F> {
    pub(crate) fn view(&self) -> impl Widget<Interpreter<'file, F>> {
        let inside = match &self.state {
            InterpreterState::NotStarted => Either::new_left(Label::new("interpreter not started".to_string())),
            InterpreterState::AboutToExecute(cur_stmt, _) => Either::new_right(VSplit::new(CodeView::new(cur_stmt.span), Label::new("interpreter running".to_string()))), // TODO
            InterpreterState::Finished(Ok(())) => Either::new_left(Label::new("interpreter finished successfully".to_string())),                                          // TODO
            InterpreterState::Finished(Err(err)) => Either::new_left(Label::new(format!("interpreter errored: {}", err))),                                                // TODO
        };

        RespondsToKeyboard::<Self, _, _>::new(sfml::window::Key::Space, |interpreter: &mut _| interpreter.step(), inside)
    }

    fn step(&mut self) {
        match self.state {
            InterpreterState::NotStarted | InterpreterState::AboutToExecute(_, _) => match self.interpret_generator.resume() {
                genawaiter::GeneratorState::Yielded(step) => self.state = InterpreterState::AboutToExecute(step.0, step.1),
                genawaiter::GeneratorState::Complete(res) => self.state = InterpreterState::Finished(res),
            },

            InterpreterState::Finished(_) => {}
        }
    }
}

#[derive(Clone)]
enum Value {
    Int(BigInt),
    Float(f64),
    String(String),
    Bool(bool),
}
pub(crate) enum Type {
    Int,
    Float,
    String,
    Bool,
}

impl Value {
    fn type_(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
            Value::String(_) => Type::String,
            Value::Bool(_) => Type::Bool,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => {
                write!(f, "{i}")?;
            }
            Value::Float(fl) => {
                write!(f, "{fl}")?;
            }
            Value::String(s) => {
                write!(f, "{s}")?;
            }
            Value::Bool(b) => {
                write!(f, "{b}")?;
            }
        }

        Ok(())
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int")?,
            Type::Float => write!(f, "float")?,
            Type::String => write!(f, "string")?,
            Type::Bool => write!(f, "bool")?,
        }

        Ok(())
    }
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

async fn interpret<'file>(stmts: Vec<Stmt<'file>>, co: Co<(Stmt<'file>, Vars)>) -> Result<(), RuntimeError<'file>> {
    use async_recursion::async_recursion;

    #[async_recursion]
    async fn interpret_expr<'file: 'async_recursion, 'parent, 'parents>(env: &mut Vars, e: Expr<'file>) -> Result<Value, RuntimeError<'file>> {
        match e.kind {
            ExprKind::Var(vname) => match env.lookup(&vname) {
                Some(Some(v)) => Ok(v.clone()),
                Some(None) => Err(RuntimeError::VarUninitialized(e.span, vname)),
                None => Err(RuntimeError::VarDoesNotExist(e.span, vname)),
            },
            ExprKind::Int(i) => Ok(Value::Int(i)),
            ExprKind::Float(f) => Ok(Value::Float(f)),
            ExprKind::String(s) => Ok(Value::String(s)),
            ExprKind::Bool(b) => Ok(Value::Bool(b)),
            ExprKind::Parenthesized(e) => Ok(interpret_expr(env, *e).await?),
            ExprKind::Call(_, _) => todo!(),
            ExprKind::ShortCircuitOp(left, op, right) => {
                let left_span = left.span;
                let right_span = right.span;
                match op {
                    ShortCircuitOp::Or => match interpret_expr(env, *left).await? {
                        Value::Bool(true) => Ok(Value::Bool(true)),
                        Value::Bool(false) => match interpret_expr(env, *right).await? {
                            a @ Value::Bool(_) => Ok(a),
                            right => Err(RuntimeError::InvalidTypeForShortCircuitOp(right_span, op, right.type_())),
                        },
                        left => Err(RuntimeError::InvalidTypeForShortCircuitOp(left_span, op, left.type_())),
                    },
                    ShortCircuitOp::And => match interpret_expr(env, *left).await? {
                        Value::Bool(false) => Ok(Value::Bool(false)),
                        Value::Bool(true) => match interpret_expr(env, *right).await? {
                            a @ Value::Bool(_) => Ok(a),
                            right => Err(RuntimeError::InvalidTypeForShortCircuitOp(right_span, op, right.type_())),
                        },
                        left => Err(RuntimeError::InvalidTypeForShortCircuitOp(left_span, op, left.type_())),
                    },
                }
            }
            ExprKind::BinaryOp(left, op, right) => {
                let left = interpret_expr(env, *left).await?;
                let right = interpret_expr(env, *right).await?;

                macro_rules! comparison {
                    ($op:tt) => {
                        match (left, right) {
                            (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool(i1 $op i2)),
                            (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool(f1 $op f2)),
                            (Value::String(s1), Value::String(s2)) => Ok(Value::Bool(s1 $op s2)),
                            (Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 $op b2)),
                            (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(e.span, op, left.type_(), right.type_())),
                        }
                    };
                }

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
                        (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(e.span, op, left.type_(), right.type_())),
                    },
                    BinaryOp::Subtract => match (left, right) {
                        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 - i2)),
                        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 - f2)),
                        (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(e.span, op, left.type_(), right.type_())),
                    },
                    BinaryOp::Multiply => match (left, right) {
                        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 * i2)),
                        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 * f2)),
                        (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(e.span, op, left.type_(), right.type_())),
                    },
                    BinaryOp::Divide => match (left, right) {
                        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 / i2)),
                        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 / f2)),
                        (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(e.span, op, left.type_(), right.type_())),
                    },
                    BinaryOp::Modulo => match (left, right) {
                        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 % i2)),
                        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 % f2)),
                        (left, right) => Err(RuntimeError::InvalidTypesForBinaryOp(e.span, op, left.type_(), right.type_())),
                    },
                }
            }
            ExprKind::UnaryOp(operator, operand) => {
                let operand = interpret_expr(env, *operand).await?;
                match operator {
                    UnaryOp::NumericNegate => match operand {
                        Value::Int(i) => Ok(Value::Int(-i)),
                        Value::Float(f) => Ok(Value::Float(-f)),
                        operand => Err(RuntimeError::InvalidTypeForUnaryOp(e.span, operator, operand.type_())),
                    },
                    UnaryOp::LogicalNegate => match operand {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        operand => Err(RuntimeError::InvalidTypeForUnaryOp(e.span, operator, operand.type_())),
                    },
                }
            }
        }
    }

    #[async_recursion]
    async fn interpret_statement<'parent, 'parents: 'parent, 'file>(stmt: Stmt<'file>, env: &mut Vars, co: &Co<(Stmt<'file>, Vars)>) -> Result<(), RuntimeError<'file>> {
        co.yield_((stmt.clone(), env.clone())).await;
        match stmt.kind {
            StmtKind::Block(stmts) => interpret_statements(stmts, env, co).await,

            StmtKind::Expr(e) => {
                interpret_expr(env, e).await?;
                Ok(())
            }

            StmtKind::Print(v) => {
                let v = interpret_expr(env, v).await?;
                println!("{v}");
                Ok(())
            }

            StmtKind::Return(_) => todo!(),

            StmtKind::MakeVar(vname, None) => {
                env.define_var(vname.clone(), None);
                Ok(())
            }

            StmtKind::MakeVar(vname, Some(initializer)) => {
                let initializer = interpret_expr(env, initializer).await?;
                env.define_var(vname.clone(), Some(initializer));
                Ok(())
            }

            StmtKind::AssignVar(var, v) => {
                let v = interpret_expr(env, v).await?;
                match env.lookup_mut(&var) {
                    Some(v_place) => {
                        *v_place = Some(v);
                        Ok(())
                    }
                    None => {
                        eprintln!("error: variable '{var}' does not exist");
                        env.end_scope();
                        Err(RuntimeError::VarDoesNotExist(stmt.span, var))
                    }
                }
            }

            StmtKind::If(cond, t, f) => {
                let cond_span = cond.span;
                match interpret_expr(env, cond).await? {
                    Value::Bool(true) => interpret_statement(*t, env, co).await,
                    Value::Bool(false) => {
                        if let Some(f) = f {
                            interpret_statement(*f, env, co).await?;
                        }
                        Ok(())
                    }
                    cond => Err(RuntimeError::ExpectedBool(cond_span, cond.type_())),
                }
            }

            StmtKind::While(cond, body) => loop {
                let cond_value = interpret_expr(env, cond.clone()).await?;
                match cond_value {
                    Value::Bool(true) => {}
                    Value::Bool(false) => break Ok(()),
                    _ => break Err(RuntimeError::ExpectedBool(cond.span, cond_value.type_())),
                }

                interpret_statement((*body).clone(), env, co).await?;
            },
        }
    }

    #[async_recursion]
    async fn interpret_statements<'parent, 'parents: 'parent, 'file>(stmts: Vec<Stmt<'file>>, env: &mut Vars, co: &Co<(Stmt<'file>, Vars)>) -> Result<(), RuntimeError<'file>> {
        env.start_scope();
        for stmt in stmts {
            interpret_statement(stmt, env, co).await?;
        }
        env.end_scope();

        Ok(())
    }

    interpret_statements(stmts, &mut Vars::default(), &co).await
}
