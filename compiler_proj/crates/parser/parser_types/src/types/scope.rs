use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Iter},
    ops::{Deref, Range},
    rc::Rc,
};

use crate::{
    Error, FunctionType, InterpreterValue, StructType, Symbol, SystemType, TypeSymbol,
    TypeSymbolType,
};

pub trait ScopeLike {
    fn resolve_value(&self, name: &Symbol) -> Result<InterpreterValue, Error>;
    fn set_value(&mut self, name: &Symbol, value: InterpreterValue) -> Result<(), Error>;
    fn resolve_type(&self, name: &Symbol) -> Result<TypeSymbol, Error>;
    fn get_outer_scope(&self) -> Result<Rc<RefCell<Scope>>, Error>;
}

#[derive(Debug, Default, Clone)]
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

    pub fn get_parent_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent.as_ref().map(Rc::clone)
    }

    pub fn set_parent_scope(&mut self, parent: Option<Rc<RefCell<Scope>>>) {
        self.parent = parent;
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
                prefab: None,
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
                is_method: _,
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

    pub fn iter_values(&self) -> Iter<'_, Symbol, InterpreterValue> {
        self.values.iter()
    }

    pub fn iter_types(&self) -> Iter<'_, Symbol, TypeSymbol> {
        self.types_for_variable.iter()
    }
}

impl ScopeLike for Scope {
    /// resolve value of a variable
    fn resolve_value(&self, name: &Symbol) -> Result<InterpreterValue, Error> {
        let value = self.values.get(name).cloned();
        if value.is_none()
            && let Some(parent) = &self.parent
        {
            parent.borrow().resolve_value(name)
        } else {
            value.ok_or(Error::SymbolNotFound(name.clone()))
        }
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

    /// Resolve type of a variable
    fn resolve_type(&self, name: &Symbol) -> Result<TypeSymbol, Error> {
        let type_of = self.types_for_variable.get(name).cloned();
        if type_of.is_none()
            && let Some(parent) = &self.parent
        {
            parent.borrow().resolve_type(name)
        } else {
            type_of.ok_or(Error::SymbolNotFound(name.clone()))
        }
    }

    fn get_outer_scope(&self) -> Result<Rc<RefCell<Scope>>, Error> {
        Ok(Rc::new(RefCell::new(self.clone())))
    }
}

impl ScopeLike for Rc<RefCell<Scope>> {
    fn resolve_type(&self, name: &Symbol) -> Result<TypeSymbol, Error> {
        self.borrow().resolve_type(name)
    }

    fn resolve_value(&self, name: &Symbol) -> Result<InterpreterValue, Error> {
        self.borrow().resolve_value(name)
    }

    fn set_value(&mut self, name: &Symbol, value: InterpreterValue) -> Result<(), Error> {
        self.borrow_mut().set_value(name, value)
    }

    fn get_outer_scope(&self) -> Result<Rc<RefCell<Scope>>, Error> {
        Ok(Rc::clone(self))
    }
}

impl ScopeLike for InterpreterValue {
    fn resolve_value(&self, name: &Symbol) -> Result<InterpreterValue, Error> {
        match self {
            InterpreterValue::Module(slf) => slf.resolve_value(name),
            InterpreterValue::Struct(_, _, attributes) => attributes
                .get(name)
                .map(Box::deref)
                .cloned()
                .ok_or(Error::SymbolNotFound(name.clone())),
            InterpreterValue::Component(_, _, attributes) => attributes
                .get(name)
                .map(Box::deref)
                .cloned()
                .ok_or(Error::SymbolNotFound(name.clone())),
            InterpreterValue::Strong(inner) => inner.borrow().resolve_value(name),
            InterpreterValue::BuiltinStruct(name, value) => value.borrow().resolve_value(name),
            _ => Err(Error::OperationUnsupported {
                operation: "resolve_value".to_owned(),
                type_of: "type is not a scope or struct".to_owned(),
            }),
        }
    }

    fn set_value(&mut self, name: &Symbol, value: InterpreterValue) -> Result<(), Error> {
        match self {
            InterpreterValue::Module(slf) => slf.set_value(name, value),
            InterpreterValue::Struct(_, _, attributes) => {
                if !attributes.contains_key(name) {
                    Err(Error::SymbolNotFound(name.clone()))
                } else {
                    attributes.insert(name.clone(), Box::new(value));
                    Ok(())
                }
            }
            InterpreterValue::Component(_, _, attributes) => {
                if !attributes.contains_key(name) {
                    Err(Error::SymbolNotFound(name.clone()))
                } else {
                    attributes.insert(name.clone(), Box::new(value));
                    Ok(())
                }
            }
            InterpreterValue::Strong(inner) => inner.borrow_mut().set_value(name, value),
            InterpreterValue::BuiltinStruct(name, self_val) => {
                self_val.borrow_mut().set_value(name, value)
            }
            _ => unimplemented!(),
        }
    }

    fn resolve_type(&self, name: &Symbol) -> Result<TypeSymbol, Error> {
        match self {
            InterpreterValue::Module(slf) => slf.resolve_type(name),
            InterpreterValue::Struct(struct_name, outer_scope, _) => {
                let struct_type = outer_scope
                    .borrow()
                    .resolve_defined_type(struct_name)
                    .ok_or(Error::SymbolNotFound(struct_name.clone()))?;
                if let TypeSymbolType::Struct(struct_type) = &struct_type.type_of {
                    if let Some(method) = struct_type
                        .methods
                        .iter()
                        .find(|(fn_name, _)| fn_name == name)
                    {
                        Ok(TypeSymbol::strong(TypeSymbolType::Function(
                            method.1.clone(),
                        )))
                    } else if let Some(static_fn) = struct_type
                        .statics
                        .iter()
                        .find(|(fn_name, _)| fn_name == name)
                    {
                        Ok(TypeSymbol::strong(TypeSymbolType::Function(
                            static_fn.1.clone(),
                        )))
                    } else {
                        struct_type
                            .fields
                            .iter()
                            .find(|(attrib_name, _)| attrib_name == name)
                            .map(|attrib| attrib.1.clone())
                            .ok_or(Error::SymbolNotFound(struct_name.clone()))
                    }
                } else {
                    Err(Error::SymbolNotFound(struct_name.clone()))
                }
            }
            InterpreterValue::Component(struct_name, outer_scope, _) => {
                let struct_type = outer_scope
                    .borrow()
                    .resolve_defined_type(struct_name)
                    .ok_or(Error::SymbolNotFound(struct_name.clone()))?;
                if let TypeSymbolType::Struct(struct_type) = &struct_type.type_of {
                    struct_type
                        .fields
                        .iter()
                        .find(|(attrib_name, _)| attrib_name == name)
                        .map(|attrib| attrib.1.clone())
                        .ok_or(Error::SymbolNotFound(struct_name.clone()))
                } else {
                    Err(Error::SymbolNotFound(struct_name.clone()))
                }
            }
            InterpreterValue::Strong(inner) => inner.borrow().resolve_type(name),
            InterpreterValue::BuiltinStruct(_, self_val) => self_val.borrow().resolve_type(name),
            _ => unimplemented!(),
        }
    }

    fn get_outer_scope(&self) -> Result<Rc<RefCell<Scope>>, Error> {
        match self {
            InterpreterValue::Module(slf) => Ok(Rc::clone(slf)),
            InterpreterValue::Struct(_, outer_scope, _) => Ok(Rc::clone(outer_scope)),
            InterpreterValue::Component(_, outer_scope, _) => Ok(Rc::clone(outer_scope)),
            InterpreterValue::Strong(inner) => inner.borrow().get_outer_scope(),
            InterpreterValue::BuiltinStruct(_, value) => value.borrow().get_outer_scope(),
            _ => Err(Error::CantDerefWeak),
        }
    }
}
