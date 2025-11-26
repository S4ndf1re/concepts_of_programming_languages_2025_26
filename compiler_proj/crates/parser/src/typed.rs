use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    iter::zip,
    rc::{Rc, Weak},
};

use derivative::Derivative;

use crate::{AstNode, Error, Symbol};

// TODO: type lookup by symbol

pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    values: HashMap<Symbol, ActualTypedValue>,
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
        value: ActualTypedValue,
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
        value: ActualTypedValue,
        type_of: TypeSymbol,
        shadow: bool,
        pre_resolve: bool,
    ) -> Result<(), Error> {
        self.declare_variable(name, value, type_of, shadow, pre_resolve)
    }

    pub fn set_value(&mut self, name: Symbol, value: ActualTypedValue) -> Result<(), Error> {
        // TODO: do type checking here
        self.values.insert(name, value);

        Ok(())
    }

    /// resolve value of a variable
    pub fn resolve_value(&self, name: Symbol) -> Option<ActualTypedValue> {
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

/// ActualTypeValue only represents the concrete value of a type. The actual type def is defined by
#[derive(Clone)]
pub enum ActualTypedValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct(HashMap<Symbol, Box<ActualTypedValue>>),
    Option(Option<Box<ActualTypedValue>>),
    Result(Result<Box<ActualTypedValue>, Box<ActualTypedValue>>),
    Function, // Functions execution body is contained in its type definition,
    // Reference counted values (everything afaik)
    Weak(Weak<ActualTypedValue>),
    Strong(Rc<ActualTypedValue>),
}

impl ActualTypedValue {}

#[derive(Derivative)]
#[derivative(Debug, Clone, Hash, Eq)]
pub struct FunctionType {
    pub name: Symbol,
    pub params: Vec<(Symbol, TypeSymbol)>,
    pub return_type: Option<Box<TypeSymbol>>,
    #[derivative(Hash = "ignore")]
    pub execution_body: Vec<Box<AstNode>>,
}

impl PartialEq for FunctionType {
    fn eq(&self, other: &Self) -> bool {
        let mut equals = true;

        for (p1, p2) in zip(&self.params, &other.params) {
            equals = equals && p1.1 == p2.1;
        }

        equals = equals && self.return_type == other.return_type;

        equals
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ret) = &self.return_type {
            write!(
                f,
                "fn {}({}): {}",
                self.name,
                self.params
                    .iter()
                    .map(|p| format!("{}: {}", p.0, p.1))
                    .collect::<Vec<String>>()
                    .join(", "),
                ret,
            )
        } else {
            write!(
                f,
                "fn {}({})",
                self.name,
                self.params
                    .iter()
                    .map(|p| format!("{}: {}", p.0, p.1))
                    .collect::<Vec<String>>()
                    .join(", "),
            )
        }
    }
}

#[derive(Debug, Clone, Hash, Eq)]
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
