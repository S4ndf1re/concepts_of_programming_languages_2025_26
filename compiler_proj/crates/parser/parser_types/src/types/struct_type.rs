use std::{cell::RefCell, collections::HashMap, fmt::Display, hash::Hash, iter::zip, rc::Rc};

use crate::{BuiltinStruct, Error, FunctionType, InterpreterValue, Scope, Symbol, TypeSymbol};

pub trait Instantiable {
    fn instantiate(
        &self,
        local_scope: Rc<RefCell<Scope>>,
        params: HashMap<Symbol, Box<InterpreterValue>>,
    ) -> Result<InterpreterValue, Error>;
    fn get_required_parameters(&self) -> Result<HashMap<Symbol, TypeSymbol>, Error>;
}

#[derive(Debug, Clone)]
pub struct StructType {
    pub name: Symbol,
    pub fields: Vec<(Symbol, TypeSymbol)>,
    // Methods are assumed to start with "self"
    pub methods: Vec<(Symbol, FunctionType)>,
    pub statics: Vec<(Symbol, FunctionType)>,
    pub prefab: Option<Rc<dyn BuiltinStruct>>,
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

impl Instantiable for StructType {
    fn instantiate(
        &self,
        scope: Rc<RefCell<Scope>>,
        params: HashMap<Symbol, Box<InterpreterValue>>,
    ) -> Result<InterpreterValue, Error> {
        if let Some(prefab) = &self.prefab {
            return prefab.instantiate(scope, params);
        }
        for param in &self.fields {
            if !params.contains_key(&param.0) {
                Err(Error::CantBeEmpty)?
            }
        }

        let struct_value =
            InterpreterValue::Struct(self.name.clone(), scope, params).make_reference_counted()?;

        Ok(struct_value)
    }

    fn get_required_parameters(&self) -> Result<HashMap<Symbol, TypeSymbol>, Error> {
        if let Some(prefab) = &self.prefab {
            return prefab.get_required_parameters();
        }

        Ok(self
            .fields
            .iter()
            .map(|value| (value.0.clone(), value.1.clone()))
            .collect::<HashMap<_, _>>())
    }
}
