use std::fmt::Display;

use num_bigint::BigInt;

use crate::interpreter::interpreter::interpreter::type_::Type;

#[derive(Clone)]
pub(crate) enum Value {
    Int(BigInt),
    Float(f64),
    String(String),
    Bool(bool),
}

impl Value {
    pub(crate) fn type_(&self) -> Type {
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
