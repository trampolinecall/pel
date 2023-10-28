use std::{collections::HashMap, fmt::Display};

use num_bigint::BigInt;

use crate::{
    error::ErrorReportedPromise,
    lang::{Expr, VarName, Stmt},
};

#[derive(Clone)]
enum Value {
    ConstInt(BigInt),
    ConstBool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::ConstInt(i) => {
                write!(f, "{i}")?;
            }
            Value::ConstBool(b) => {
                write!(f, "{b}")?;
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct InterpreterState {
    vars: HashMap<VarName, Value>,

    cur_instr: usize,
}

/*
pub(crate) fn interpret(program: &Program) -> Result<(), ErrorReportedPromise> {
    let mut state = InterpreterState::default();

    fn interpret_value(state: &mut InterpreterState, v: &Expr) -> Result<Value, ()> {
        match v {
            Expr::GetVar(vname) => {
                state.vars.get(vname).cloned().ok_or_else(|| {
                    eprintln!("error: variable '{vname}' does not exist"); // TODO: diagnostic system
                })
            }
            Expr::ConstInt(i) => Ok(Value::ConstInt(i.clone())),
            Expr::ConstBool(b) => Ok(Value::ConstBool(*b)),
        }
    }

    loop {
        if state.cur_instr >= program.instructions.len() {
            break;
        }

        let instr = &program.instructions[state.cur_instr];
        match instr {
            Statement::MakeVar(vname, initializer) => {
                let initializer = interpret_value(&mut state, initializer)?;
                state.vars.insert(vname.clone(), initializer);
            }
            Statement::AssignVar(var, v) => {
                let v = interpret_value(&mut state, v)?;
                match state.vars.get_mut(var) {
                    Some(v_place) => {
                        *v_place = v;
                    }
                    None => {
                        eprintln!("error: variable '{var}' does not exist");
                        return Err(());
                    }
                }
            }
            Statement::Print(v) => {
                let v = interpret_value(&mut state, v)?;
                println!("{v}");
            }
            Statement::Block(_) => todo!(),
        }
    }

    Ok(())
}
*/
