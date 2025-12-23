use std::{fmt::Display, hash::Hash, iter::zip};

use crate::{Symbol, TypeSymbol};

#[derive(Debug, Clone, Eq)]
pub struct ComponentType {
    pub name: Symbol,
    pub fields: Vec<(Symbol, TypeSymbol)>,
}

impl PartialEq for ComponentType {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        for (p1, p2) in zip(&self.fields, &other.fields) {
            if p1.1 != p2.1 || p1.0 != p2.0 {
                return false;
            }
        }

        true
    }
}

impl Hash for ComponentType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // NOTE: It should be enough to assume that a scope may only contain a type once, hence this hash is enough!
        self.name.hash(state);
    }
}

impl Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "struct {} {{", self.name)?;
        for field in &self.fields {
            write!(f, "{}: {},", field.0, field.1)?;
        }

        Ok(())
    }
}
