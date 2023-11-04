use std::fmt::Display;

pub(crate) enum Type {
    Int,
    Float,
    String,
    Bool,
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
