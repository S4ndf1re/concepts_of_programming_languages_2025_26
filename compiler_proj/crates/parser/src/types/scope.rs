use std::{cell::RefCell, collections::HashMap, ops::Range, rc::Rc};

use crate::{
    Error, FunctionType, InterpreterValue, StructType, Symbol, SystemType, TypeSymbol,
    TypeSymbolType,
};

pub trait ScopeLike {
    fn resolve_value(&self, name: &Symbol) -> Option<InterpreterValue>;
    fn set_value(&mut self, name: &Symbol, value: InterpreterValue) -> Result<(), Error>;
}

#[derive(Debug, Default)]
pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    values: HashMap<Symbol, InterpreterValue>,
    types_for_variable: HashMap<Symbol, TypeSymbol>,
    defined_types: HashMap<Symbol, TypeSymbol>,
    original_locations: HashMap<Symbol, Range<usize>>,
}

impl Scope {
    pub fn new_parented(parent: Rc<RefCell<Scope>>) -> Self {
        Self {
            parent: Some(parent),
            values: HashMap::new(),
            types_for_variable: HashMap::new(),
            defined_types: HashMap::new(),
            original_locations: HashMap::new(),
        }
    }

    pub fn declare_type(
        &mut self,
        name: Symbol,
        mut type_of: TypeSymbol,
        pre_resolve: bool,
        location: Range<usize>,
    ) -> Result<(), Error> {
        if !pre_resolve {
            self.check_variable_type(&mut type_of)?;
        } else {
            let _ = self.check_variable_type(&mut type_of);
        }

        self.defined_types.insert(name.clone(), type_of);
        self.original_locations.insert(name, location);
        Ok(())
    }

    fn check_variable_type_helper(&self, type_of: &mut TypeSymbolType) -> Result<(), Error> {
        match type_of {
            TypeSymbolType::SelfType => Ok(()),
            TypeSymbolType::Any => Ok(()),
            TypeSymbolType::Entity => Ok(()),
            TypeSymbolType::Symbol(s) => {
                if self.resolve_defined_type(s).is_some() {
                    Ok(())
                } else {
                    Err(Error::TypeDoesNotExist(s.clone()))
                }
            }
            TypeSymbolType::List(t) => self.check_variable_type(t.as_mut()),
            TypeSymbolType::Map(k, v) => {
                self.check_variable_type(k.as_mut())?;
                self.check_variable_type(v.as_mut())?;
                Ok(())
            }
            TypeSymbolType::Option(t) => self.check_variable_type(t.as_mut()),
            TypeSymbolType::Result(o, e) => {
                self.check_variable_type(o.as_mut())?;
                self.check_variable_type(e.as_mut())?;
                Ok(())
            }
            TypeSymbolType::Struct(StructType {
                name: _,
                fields,
                methods,
                statics,
            }) => {
                for field in fields {
                    self.check_variable_type(&mut field.1)?;
                }

                for func in methods {
                    for param in &mut func.1.params {
                        self.check_variable_type(&mut param.1)?;
                    }
                    if let Some(ret_type) = &mut func.1.return_type {
                        self.check_variable_type(ret_type.as_mut())?;
                    }
                }

                for func in statics {
                    for param in &mut func.1.params {
                        self.check_variable_type(&mut param.1)?;
                    }
                    if let Some(ret_type) = &mut func.1.return_type {
                        self.check_variable_type(ret_type.as_mut())?;
                    }
                }

                Ok(())
            }
            TypeSymbolType::Function(FunctionType {
                name: _,
                params,
                return_type,
                execution_body: _,
            }) => {
                for param in params {
                    self.check_variable_type(&mut param.1)?;
                }
                if let Some(ret_type) = return_type {
                    self.check_variable_type(ret_type.as_mut())?;
                }
                Ok(())
            }
            TypeSymbolType::System(SystemType {
                name: _,
                params: _,
                queries,
                execution_body: _,
            }) => {
                if let Some(queries) = queries {
                    for query in queries {
                        for dependency in query.type_of.get_dependent_symbols() {
                            if self.resolve_defined_type(dependency).is_none() {
                                Err(Error::TypeDeductionError)?;
                            }
                        }
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Check type recurively, until a symbol or final type is found (i.e. int, float, string, bool, symbol)
    fn check_variable_type(&self, type_of: &mut TypeSymbol) -> Result<(), Error> {
        let res = self.check_variable_type_helper(&mut type_of.type_of);

        if res.is_ok() {
            type_of.mark_as_resolved();
        } else {
            type_of.mark_as_unresolved();
        }

        res
    }

    /// declare a variable, or shadow it if the shadow flag is set to true.
    /// i.e. when shadow: true:
    /// a := 10;
    /// a := "";
    /// is allowed
    pub fn declare_variable(
        &mut self,
        name: Symbol,
        value: InterpreterValue,
        mut type_of: TypeSymbol,
        shadow: bool,
        pre_resolve: bool,
        location: Range<usize>,
    ) -> Result<(), Error> {
        if !shadow && self.types_for_variable.contains_key(&name) {
            return Err(Error::VariableAlreadyDeclared(name));
        }

        if !pre_resolve {
            self.check_variable_type(&mut type_of)?;
        } else {
            let _ = self.check_variable_type(&mut type_of);
        }

        self.types_for_variable.insert(name.clone(), type_of);
        self.values.insert(name.clone(), value);
        self.original_locations.insert(name, location);

        Ok(())
    }

    pub fn declare_function(
        &mut self,
        name: Symbol,
        value: InterpreterValue,
        type_of: TypeSymbol,
        shadow: bool,
        pre_resolve: bool,
        location: Range<usize>,
    ) -> Result<(), Error> {
        self.declare_variable(name, value, type_of, shadow, pre_resolve, location)
    }

    pub fn declare_system(
        &mut self,
        name: Symbol,
        value: InterpreterValue,
        type_of: TypeSymbol,
        shadow: bool,
        pre_resolve: bool,
        location: Range<usize>,
    ) -> Result<(), Error> {
        self.declare_variable(name, value, type_of, shadow, pre_resolve, location)
    }

    /// Resolve type of a variable
    pub fn resolve_type(&self, name: &Symbol) -> Option<TypeSymbol> {
        let mut type_of = self.types_for_variable.get(name).cloned();
        if type_of.is_none()
            && let Some(parent) = &self.parent
        {
            type_of = parent.borrow().resolve_type(name);
        }

        type_of
    }

    /// Resolve a defined type (not for a variable)
    pub fn resolve_defined_type(&self, name: &Symbol) -> Option<TypeSymbol> {
        let mut type_of = self.defined_types.get(name).cloned();
        if type_of.is_none()
            && let Some(parent) = &self.parent
        {
            type_of = parent.borrow().resolve_defined_type(name);
        }

        type_of
    }

    pub fn check_all_types_after_pre_resolve(mut self) -> Result<Self, Error> {
        let mut new_defined_types = HashMap::new();
        let mut new_variable_types = HashMap::new();

        for mut t in self.defined_types.clone() {
            self.check_variable_type(&mut t.1)?;
            new_defined_types.insert(t.0, t.1);
        }

        for mut v in self.types_for_variable.clone() {
            self.check_variable_type(&mut v.1)?;
            new_variable_types.insert(v.0, v.1);
        }

        self.defined_types = new_defined_types;
        self.types_for_variable = new_variable_types;
        Ok(self)
    }
}

impl ScopeLike for Scope {
    /// resolve value of a variable
    fn resolve_value(&self, name: &Symbol) -> Option<InterpreterValue> {
        let mut value = self.values.get(name).cloned();
        if value.is_none()
            && let Some(parent) = &self.parent
        {
            value = parent.borrow().resolve_value(name);
        }

        value
    }

    fn set_value(&mut self, name: &Symbol, value: InterpreterValue) -> Result<(), Error> {
        // TODO: do type checking here
        // NOTE(Jan): use values.get over resolve_value here, since it hast to be checked if THIS scope contains &name, and not any scope hierarchical
        let scoped_variable = self.values.get_mut(name);
        if let Some(scoped_variable) = scoped_variable {
            *scoped_variable = value;
        } else {
            match &self.parent {
                Some(parent) => {
                    parent.borrow_mut().set_value(name, value)?;
                }
                _ => {
                    Err(Error::SymbolNotFound(name.to_owned()))?;
                }
            }
        }
        Ok(())
    }
}
