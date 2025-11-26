use std::{fmt::Display, hash::Hash};

use crate::{FunctionType, StructType, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeSymbolType {
    Int,
    Float,
    Bool,
    String,
    Symbol(Symbol),
    List(Box<TypeSymbol>),
    Map(Box<TypeSymbol>, Box<TypeSymbol>),
    Option(Box<TypeSymbol>),
    Result(Box<TypeSymbol>, Box<TypeSymbol>),
    Struct(StructType),
    Function(FunctionType),
    SelfType,
}

impl Display for TypeSymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::String => write!(f, "string"),
            Self::Bool => write!(f, "bool"),
            Self::Symbol(s) => write!(f, "{}", s),
            Self::List(s) => write!(f, "[{}]", s),
            Self::Map(k, v) => write!(f, "{{{} -> {}}}", k, v),
            Self::Option(v) => write!(f, "{}?", v),
            Self::Result(v, e) => write!(f, "{}!{}", v, e),
            Self::Struct(s) => write!(f, "{}", s),
            Self::Function(v) => write!(f, "{}", v),
            Self::SelfType => write!(f, "self"),
        }
    }
}

// let a: weak int = 10;
// TypeSymbol {
// is_weak: true,
// type_of: TypeSymbolType::Symbol(int),
// }

/// The symbol that represents any existing type
#[derive(Debug, Clone, Hash, Eq)]
pub struct TypeSymbol {
    pub is_weak: bool,
    pub type_of: TypeSymbolType,
    pub resolved: bool,
    pub inferred: bool,
}

impl TypeSymbol {
    pub fn strong(type_of: TypeSymbolType) -> Self {
        Self {
            is_weak: false,
            type_of,
            resolved: false,
            inferred: false,
        }
    }

    pub fn weak(type_of: TypeSymbolType) -> Self {
        Self {
            is_weak: true,
            type_of,
            resolved: false,
            inferred: false,
        }
    }
    pub fn make_weak(mut self) -> Self {
        self.is_weak = true;
        self
    }

    pub fn mark_as_unresolved(&mut self) {
        self.resolved = false;
    }

    pub fn mark_as_resolved(&mut self) {
        self.resolved = true;
    }
}

impl PartialEq for TypeSymbol {
    fn eq(&self, other: &Self) -> bool {
        self.is_weak == other.is_weak && self.type_of == other.type_of
    }
}

impl Display for TypeSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_weak {
            write!(f, "weak ")?;
        }

        write!(f, "{}", self.type_of)?;

        Ok(())
    }
}
