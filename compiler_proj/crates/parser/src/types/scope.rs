use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{Error, FunctionType, InterpreterValue, StructType, Symbol, TypeSymbol, TypeSymbolType};

pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    values: HashMap<Symbol, InterpreterValue>,
    types_for_variable: HashMap<Symbol, TypeSymbol>,
    defined_types: HashMap<Symbol, TypeSymbol>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
            types_for_variable: HashMap::new(),
            defined_types: HashMap::new(),
        }
    }

    pub fn declare_type(
        &mut self,
        name: Symbol,
        mut type_of: TypeSymbol,
        pre_resolve: bool,
    ) -> Result<(), Error> {
        if !pre_resolve {
            self.check_variable_type(&mut type_of)?;
        } else {
            let _ = self.check_variable_type(&mut type_of);
        }

        self.defined_types.insert(name, type_of);
        Ok(())
    }

    fn check_variable_type_helper(&self, type_of: &mut TypeSymbolType) -> Result<(), Error> {
        match type_of {
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
    ) -> Result<(), Error> {
        if !shadow {
            if self.types_for_variable.contains_key(&name) {
                return Err(Error::VariableAlreadyDeclared(name));
            }
        }

        if !pre_resolve {
            self.check_variable_type(&mut type_of)?;
        } else {
            let _ = self.check_variable_type(&mut type_of);
        }

        self.types_for_variable.insert(name.clone(), type_of);
        self.values.insert(name.clone(), value);

        Ok(())
    }

    pub fn declare_function(
        &mut self,
        name: Symbol,
        value: InterpreterValue,
        type_of: TypeSymbol,
        shadow: bool,
        pre_resolve: bool,
    ) -> Result<(), Error> {
        self.declare_variable(name, value, type_of, shadow, pre_resolve)
    }

    pub fn set_value(&mut self, name: Symbol, value: InterpreterValue) -> Result<(), Error> {
        // TODO: do type checking here
        self.values.insert(name, value);

        Ok(())
    }

    /// resolve value of a variable
    pub fn resolve_value(&self, name: Symbol) -> Option<InterpreterValue> {
        let mut value = self.values.get(&name).cloned();
        if value.is_none()
            && let Some(parent) = &self.parent
        {
            value = parent.borrow().resolve_value(name);
        }

        value
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