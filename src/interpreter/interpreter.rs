use std::{collections::HashMap, fmt::Display, future::Future};

use genawaiter::rc::{Co, Gen};
use num_bigint::BigInt;

use crate::{
    interpreter::lang::{Expr, ExprKind, Stmt, StmtKind, VarName},
    source::Span,
    visualizer::widgets::{label::Label, Widget},
};

pub(crate) struct Interpreter<'file, F: Future<Output = Result<(), RuntimeError<'file>>>> {
    state: InterpreterState<'file>,

    interpret_generator: Gen<(Stmt<'file>, Environments), (), F>,
}
enum InterpreterState<'file> {
    NotStarted,
    AboutToExecute(Stmt<'file>, Environments),
    Finished(Result<(), RuntimeError<'file>>),
}
struct Environments {
    envs: Vec<HashMap<VarName, Value>>,
}

pub(crate) fn new_interpreter(stmts: Vec<Stmt>) -> Interpreter<impl Future<Output = Result<(), RuntimeError>>> {
    let gen = Gen::new(move |co| interpret(stmts, co));
    Interpreter { state: InterpreterState::NotStarted, interpret_generator: gen }
}
impl<'file, F: Future<Output = Result<(), RuntimeError<'file>>>> Interpreter<'file, F> {
    pub(crate) fn view(&self) -> impl Widget<Interpreter<'file, F>> {
        match &self.state {
            InterpreterState::NotStarted => Label::new("interpreter not started".to_string()),                 // TODO
            InterpreterState::AboutToExecute(_, _) => Label::new("interpreter running".to_string()),           // TODO
            InterpreterState::Finished(Ok(())) => Label::new("interpreter finished successfully".to_string()), // TODO
            InterpreterState::Finished(Err(err)) => Label::new(format!("interpreter errored: {}", err)),       // TODO
        }
    }
}

#[derive(Clone)]
enum Value {
    Int(BigInt),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => {
                write!(f, "{i}")?;
            }
            Value::Bool(b) => {
                write!(f, "{b}")?;
            }
        }

        Ok(())
    }
}

pub(crate) enum RuntimeError<'file> {
    VarUninitialized(Span<'file>, VarName),
    VarDoesNotExist(Span<'file>, VarName),
}
impl Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::VarUninitialized(_, vn) => write!(f, "variable '{}' is uninitialized", vn),
            RuntimeError::VarDoesNotExist(_, vn) => write!(f, "variable '{}' does not exist", vn),
        }
    }
}

#[derive(Default)]
struct Environment<'parent> {
    vars: HashMap<VarName, Option<Value>>,
    parent: Option<&'parent mut Environment<'parent>>,
}
impl Environment<'_> {
    fn lookup(&self, vname: &VarName) -> Option<&Option<Value>> {
        if let Some(result) = self.vars.get(vname) {
            Some(result)
        } else {
            match &self.parent {
                Some(parent) => parent.lookup(vname),
                None => None,
            }
        }
    }
    fn lookup_mut(&mut self, vname: &VarName) -> Option<&mut Option<Value>> {
        if let Some(result) = self.vars.get_mut(vname) {
            Some(result)
        } else {
            match &mut self.parent {
                Some(parent) => parent.lookup_mut(vname),
                None => None,
            }
        }
    }

    fn to_environments(&self) -> Environments {
        todo!()
    }
}
async fn interpret<'file>(stmts: Vec<Stmt<'file>>, co: Co<(Stmt<'file>, Environments)>) -> Result<(), RuntimeError<'file>> {
    async fn interpret_value<'file, 'parent>(env: &mut Environment<'parent>, v: &Expr<'file>) -> Result<Value, RuntimeError<'file>> {
        match v.kind {
            ExprKind::Var(_) => todo!(),
            ExprKind::Int(_) => todo!(),
            ExprKind::Float(_) => todo!(),
            ExprKind::String(_) => todo!(),
            ExprKind::Bool(_) => todo!(),
            ExprKind::Parenthesized(_) => todo!(),
            ExprKind::Call(_, _) => todo!(),
            ExprKind::ShortCircuitOp(_, _, _) => todo!(),
            ExprKind::BinaryOp(_, _, _) => todo!(),
            ExprKind::UnaryOp(_, _) => todo!(),
        }
    }

    let mut env = Environment::default();
    for stmt in stmts {
        co.yield_((stmt.clone(), env.to_environments())).await;
        match stmt.kind {
            StmtKind::Block(_) => todo!(),

            StmtKind::Expr(_) => todo!(),

            StmtKind::Print(v) => {
                let v = interpret_value(&mut env, &v).await?;
                println!("{v}");
            }

            StmtKind::Return(_) => todo!(),

            StmtKind::MakeVar(vname, None) => {
                env.vars.insert(vname.clone(), None);
            }

            StmtKind::MakeVar(vname, Some(initializer)) => {
                let initializer = interpret_value(&mut env, &initializer).await?;
                env.vars.insert(vname.clone(), Some(initializer));
            }

            StmtKind::AssignVar(var, v) => {
                let v = interpret_value(&mut env, &v).await?;
                match env.lookup_mut(&var) {
                    Some(v_place) => {
                        *v_place = Some(v);
                    }
                    None => {
                        eprintln!("error: variable '{var}' does not exist");
                        return Err(RuntimeError::VarDoesNotExist(stmt.span, var));
                    }
                }
            }

            StmtKind::If(_, _, _) => todo!(),

            StmtKind::While(_, _) => todo!(),
        }
    }

    Ok(())
}
