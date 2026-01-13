use std::{collections::HashMap, fmt::Display, hash::Hash, iter::zip};

use crate::{Instantiable, InterpreterValue, Symbol, TypeSymbol};

#[derive(Debug, Clone)]
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

impl Instantiable for ComponentType {
    fn instantiate(
        &self,
        local_scope: std::rc::Rc<std::cell::RefCell<super::Scope>>,
        params: HashMap<Symbol, Box<super::InterpreterValue>>,
    ) -> Result<super::InterpreterValue, crate::Error> {
        let is_placeholder = params.is_empty() || self.fields.is_empty();

        let struct_value = if !is_placeholder {
            InterpreterValue::Component(self.name.clone(), local_scope, params)
                .make_reference_counted()
        } else {
            InterpreterValue::ComponentPlaceholder(self.name.clone()).make_reference_counted()
        }?;

        Ok(struct_value)
    }

    fn get_required_parameters(&self) -> std::collections::HashMap<Symbol, TypeSymbol> {
        self.fields
            .iter()
            .map(|value| (value.0.clone(), value.1.clone()))
            .collect::<HashMap<_, _>>()
    }
}
