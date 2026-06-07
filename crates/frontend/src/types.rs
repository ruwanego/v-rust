use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Bool,
    I64,
    String,
}

impl Type {
    #[must_use]
    pub fn is_integer(self) -> bool {
        matches!(self, Type::I64)
    }

    #[must_use]
    pub fn is_bool(self) -> bool {
        matches!(self, Type::Bool)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Type::Void => "void",
            Type::Bool => "bool",
            Type::I64 => "i64",
            Type::String => "String",
        };
        formatter.write_str(name)
    }
}
