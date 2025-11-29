use std::{fmt::Display, hash::Hash, iter::zip};

use crate::{FunctionType, Symbol, TypeSymbol};


#[derive(Debug, Clone, Eq)]
pub struct StructType {
    pub name: Symbol,
    pub fields: Vec<(Symbol, TypeSymbol)>,
    // Methods are assumed to start with "self"
    pub methods: Vec<(Symbol, FunctionType)>,
    pub statics: Vec<(Symbol, FunctionType)>,
}

impl PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        for (p1, p2) in zip(&self.fields, &other.fields) {
            if p1.1 != p2.1 || p1.0 != p2.0 {
                return false;
            }
        }

        for (p1, p2) in zip(&self.methods, &other.methods) {
            if p1.1 != p2.1 || p1.0 != p2.0 {
                return false;
            }
        }

        for (p1, p2) in zip(&self.statics, &other.statics) {
            if p1.1 != p2.1 || p1.0 != p2.0 {
                return false;
            }
        }

        true
    }
}

impl Hash for StructType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // NOTE: It should be enough to assume that a scope may only contain a type once, hence this hash is enough!
        self.name.hash(state);
    }
}

impl Display for StructType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "struct {} {{", self.name)?;
        for field in &self.fields {
            write!(f, "{}: {},", field.0, field.1)?;
        }

        for function in &self.methods {
            write!(f, "{}: {},", function.0, function.1)?;
        }
        for function in &self.statics {
            write!(f, "{}: {},", function.0, function.1)?;
        }
        Ok(())
    }
}
