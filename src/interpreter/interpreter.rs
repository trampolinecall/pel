use std::{collections::HashMap, fmt::Display, future::Future};

use genawaiter::sync::{Co, Gen};
use num_bigint::BigInt;

use crate::{
    interpreter::lang::{BinaryOp, Expr, ExprKind, ShortCircuitOp, Stmt, StmtKind, UnaryOp, VarName},
    source::{Located, Span},
    visualizer::{
        graphics::Fonts,
        widgets::{code_view::code_view, either::Either, empty::Empty, expand::Expand, flex, label::Label, responds_to_keyboard::RespondsToKeyboard, Widget},
    },
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
struct InterpretYield<'file> {
    msg: String,
    highlight: Span<'file>,
    program_output: String,
    env: Vars,
}
pub(crate) struct Interpreter<'file, F: Future<Output = Result<(), RuntimeError<'file>>>> {
    state: InterpreterState<'file>,

    interpret_generator: Gen<InterpretYield<'file>, (), F>,
}
enum InterpreterState<'file> {
    NotStarted,
    AboutToExecute(InterpretYield<'file>),
    Finished { result: Result<(), RuntimeError<'file>> },
}

pub(crate) fn new_interpreter(stmts: Vec<Stmt>) -> Interpreter<impl Future<Output = Result<(), RuntimeError>>> {
    let gen = Gen::new(move |co| interpret(stmts, co));
    Interpreter { state: InterpreterState::NotStarted, interpret_generator: gen }
}
impl<'file, F: Future<Output = Result<(), RuntimeError<'file>>> + 'file> Interpreter<'file, F> {
    pub(crate) fn view(&self) -> impl Widget<Interpreter<'file, F>> {
        let (code_view, msg) = match &self.state {
            InterpreterState::NotStarted => (Either::new_left(Empty), Label::new("interpreter not started".to_string(), Fonts::text_font, 15)),
            InterpreterState::AboutToExecute(InterpretYield { msg, highlight, env, program_output }) => {
                // TODO: hashmap does not preserve order that variables are created
                // TODO: var and value side by side in table aligned
                let env_view = flex::homogeneous::Flex::new(
                    flex::Direction::Vertical,
                    env.scopes
                        .iter()
                        .flat_map(|env_scope| {
                            env_scope.iter().map(|(var_name, value)| {
                                (
                                    flex::ItemSettings::Fixed,
                                    Label::new(
                                        match value {
                                            // TODO: min height?
                                            Some(value) => format!("{var_name}: {value}"),
                                            None => format!("{var_name}: <uninitialized>"),
                                        },
                                        Fonts::text_font,
                                        15,
                                    ),
                                )
                            })
                        })
                        .collect(),
                );

                (
                    Either::new_right(flex! {
                        horizontal
                        code_view: ItemSettings::Flex(0.8), code_view(*highlight, Fonts::text_font, 15, Fonts::monospace_font, 15),
                        program_output: ItemSettings::Flex(0.3), Label::new(program_output.clone(), Fonts::monospace_font, 15), // TODO: scrolling, min size, fixed size?
                        env_view: ItemSettings::Flex(0.2), env_view,
                    }),
                    Label::new(format!("running\n{msg}"), Fonts::text_font, 15),
                )
            }
            InterpreterState::Finished { result: Ok(()) } => (Either::new_left(Empty), Label::new("interpreter finished successfully".to_string(), Fonts::text_font, 15)),
            InterpreterState::Finished { result: Err(err) } => (Either::new_left(Empty), Label::new(format!("interpreter had error: {err}"), Fonts::text_font, 15)),
        };

        RespondsToKeyboard::<Self, _, _>::new(
            sfml::window::Key::Space,
            |interpreter: &mut _| interpreter.step(),
            flex! {
                horizontal
                code_view: ItemSettings::Flex(0.8), code_view,
                msg: ItemSettings::Flex(0.2), msg,
            },
        )
    }

    fn step(&mut self) {
        match self.state {
            InterpreterState::NotStarted | InterpreterState::AboutToExecute { .. } => match self.interpret_generator.resume() {
                genawaiter::GeneratorState::Yielded(step) => self.state = InterpreterState::AboutToExecute(step),
                genawaiter::GeneratorState::Complete(res) => self.state = InterpreterState::Finished { result: res },
            },

            InterpreterState::Finished { result: _, .. } => {}
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

// TODO: print and repr
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

type ICo<'file> = Co<InterpretYield<'file>>;
async fn interpret<'file>(stmts: Vec<Stmt<'file>>, co: ICo<'file>) -> Result<(), RuntimeError<'file>> {
    use async_recursion::async_recursion;

    #[async_recursion]
    async fn interpret_expr<'file: 'async_recursion, 'parent, 'parents>(env: &mut Vars, program_output: &mut String, e: Expr<'file>, co: &ICo<'file>) -> Result<Value, RuntimeError<'file>> {
        match e.kind {
            ExprKind::Var(vname) => {
                co.yield_(InterpretYield { msg: format!("read variable '{vname}'"), highlight: e.span, program_output: program_output.clone(), env: env.clone() }).await;
                match env.lookup(&vname) {
                    Some(Some(v)) => Ok(v.clone()),
                    Some(None) => Err(RuntimeError::VarUninitialized(e.span, vname)),
                    None => Err(RuntimeError::VarDoesNotExist(e.span, vname)),
                }
            }
            ExprKind::Int(i) => Ok(Value::Int(i)),
            ExprKind::Float(f) => Ok(Value::Float(f)),
            ExprKind::String(s) => Ok(Value::String(s)),
            ExprKind::Bool(b) => Ok(Value::Bool(b)),
            ExprKind::Parenthesized(e) => Ok(interpret_expr(env, program_output, *e, co).await?),
            ExprKind::Call(_, _) => {
                todo!()
            }
            ExprKind::ShortCircuitOp(left, Located(_, op), right) => {
                let left_span = left.span;
                let right_span = right.span;
                match op {
                    ShortCircuitOp::Or => match interpret_expr(env, program_output, *left, co).await? {
                        Value::Bool(true) => Ok(Value::Bool(true)),
                        Value::Bool(false) => match interpret_expr(env, program_output, *right, co).await? {
                            a @ Value::Bool(_) => Ok(a),
                            right => Err(RuntimeError::InvalidTypeForShortCircuitOp(right_span, op, right.type_())),
                        },
                        left => Err(RuntimeError::InvalidTypeForShortCircuitOp(left_span, op, left.type_())),
                    },
                    ShortCircuitOp::And => match interpret_expr(env, program_output, *left, co).await? {
                        Value::Bool(false) => Ok(Value::Bool(false)),
                        Value::Bool(true) => match interpret_expr(env, program_output, *right, co).await? {
                            a @ Value::Bool(_) => Ok(a),
                            right => Err(RuntimeError::InvalidTypeForShortCircuitOp(right_span, op, right.type_())),
                        },
                        left => Err(RuntimeError::InvalidTypeForShortCircuitOp(left_span, op, left.type_())),
                    },
                }
            }
            ExprKind::BinaryOp(left, Located(op_span, op), right) => {
                let left = interpret_expr(env, program_output, *left, co).await?;
                let right = interpret_expr(env, program_output, *right, co).await?;

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

                co.yield_(InterpretYield { msg: "evaluate operation".to_string(), highlight: op_span, program_output: program_output.clone(), env: env.clone() }).await; // TODO: put operator in quotes in message
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
                let operand = interpret_expr(env, program_output, *operand, co).await?;
                co.yield_(InterpretYield { msg: "evaluate operation".to_string(), highlight: operator_span, program_output: program_output.clone(), env: env.clone() }).await;
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

    #[async_recursion]
    async fn interpret_statement<'parent, 'parents: 'parent, 'file>(env: &mut Vars, program_output: &mut String, stmt: Stmt<'file>, co: &ICo<'file>) -> Result<(), RuntimeError<'file>> {
        match stmt.kind {
            StmtKind::Block(stmts) => interpret_statements(env, program_output, stmts, co).await,

            StmtKind::Expr(e) => {
                interpret_expr(env, program_output, e, co).await?;
                Ok(())
            }

            StmtKind::Print(v) => {
                let v = interpret_expr(env, program_output, v, co).await?;
                co.yield_(InterpretYield { msg: format!("print value '{v}'"), highlight: stmt.span, program_output: program_output.clone(), env: env.clone() }).await;
                *program_output += &v.to_string();
                *program_output += "\n";
                Ok(())
            }

            StmtKind::Return(_) => todo!(),

            StmtKind::MakeVar(vname, None) => {
                co.yield_(InterpretYield { msg: format!("make uninitialized variable '{vname}'"), highlight: stmt.span, program_output: program_output.clone(), env: env.clone() }).await;
                env.define_var(vname.clone(), None);
                Ok(())
            }

            StmtKind::MakeVar(vname, Some(initializer)) => {
                let initializer = interpret_expr(env, program_output, initializer, co).await?;
                co.yield_(InterpretYield { msg: format!("make variable '{vname}' with initializer {initializer}"), highlight: stmt.span, program_output: program_output.clone(), env: env.clone() })
                    .await;
                env.define_var(vname.clone(), Some(initializer));
                Ok(())
            }

            StmtKind::AssignVar(var, v) => {
                let v = interpret_expr(env, program_output, v, co).await?;
                co.yield_(InterpretYield { msg: format!("assign variable '{var}' with value {v}"), highlight: stmt.span, program_output: program_output.clone(), env: env.clone() }).await;
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

            StmtKind::If(if_span, cond, t, f) => {
                let cond_span = cond.span;
                let cond = interpret_expr(env, program_output, cond, co).await?;
                co.yield_(InterpretYield { msg: "check condition".to_string(), highlight: if_span, program_output: program_output.clone(), env: env.clone() }).await;
                match cond {
                    Value::Bool(true) => interpret_statement(env, program_output, *t, co).await,
                    Value::Bool(false) => {
                        if let Some(f) = f {
                            interpret_statement(env, program_output, *f, co).await?;
                        }
                        Ok(())
                    }
                    cond => Err(RuntimeError::ExpectedBool(cond_span, cond.type_())),
                }
            }

            StmtKind::While(while_span, cond, body) => loop {
                let cond_value = interpret_expr(env, program_output, cond.clone(), co).await?;
                co.yield_(InterpretYield { msg: "check condition".to_string(), highlight: while_span, program_output: program_output.clone(), env: env.clone() }).await;
                match cond_value {
                    Value::Bool(true) => {}
                    Value::Bool(false) => break Ok(()),
                    _ => break Err(RuntimeError::ExpectedBool(cond.span, cond_value.type_())),
                }

                interpret_statement(env, program_output, (*body).clone(), co).await?;
            },
        }
    }

    #[async_recursion]
    async fn interpret_statements<'parent, 'parents: 'parent, 'file>(env: &mut Vars, program_output: &mut String, stmts: Vec<Stmt<'file>>, co: &ICo<'file>) -> Result<(), RuntimeError<'file>> {
        env.start_scope();
        for stmt in stmts {
            interpret_statement(env, program_output, stmt, co).await?;
        }
        env.end_scope();

        Ok(())
    }

    interpret_statements(&mut Vars::default(), &mut String::new(), stmts, &co).await
}
