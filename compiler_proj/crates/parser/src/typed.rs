use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    iter::zip,
    rc::{Rc, Weak},
};

use thiserror::Error;

use crate::{AstNode, Symbol};

#[derive(Error, Debug)]
pub enum Error {
    #[error("variable {0} already declared")]
    VariableAlreadyDeclared(Symbol),
    #[error("value {0} and type {1} do not match")]
    ValueAndTypeDoNotMatch(String, String),
}

// TODO: type lookup by symbol

pub struct Scope {
    parent: Option<Rc<RefCell<Scope>>>,
    values: HashMap<Symbol, ActualTypeValue>,
    types: HashMap<Symbol, TypeSymbol>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// declare a variable, or shadow it if the shadow flag is set to true.
    /// i.e. when shadow: true:
    /// a := 10;
    /// a := "";
    /// is allowed
    pub fn declare_variable(
        &mut self,
        name: Symbol,
        value: ActualTypeValue,
        type_of: TypeSymbol,
        shadow: bool,
    ) -> Result<(), Error> {
        if !shadow {
            if self.types.contains_key(&name) {
                return Err(Error::VariableAlreadyDeclared(name));
            }
        }

        // TODO: do type checking here (or assume its already done im prechecking)
        self.types.insert(name.clone(), type_of);
        self.values.insert(name.clone(), value);

        Ok(())
    }

    pub fn set_value(&mut self, name: Symbol, value: ActualTypeValue) -> Result<(), Error> {
        // TODO: do type checking here
        self.values.insert(name, value);

        Ok(())
    }

    pub fn resolve_value(&self, name: Symbol) -> Option<ActualTypeValue> {
        let mut value = self.values.get(&name).cloned();
        if value.is_none()
            && let Some(parent) = &self.parent
        {
            value = parent.borrow().resolve_value(name);
        }

        value
    }

    pub fn resolve_type(&self, name: Symbol) -> Option<TypeSymbol> {
        let mut type_of = self.types.get(&name).cloned();
        if type_of.is_none()
            && let Some(parent) = &self.parent
        {
            type_of = parent.borrow().resolve_type(name);
        }

        type_of
    }
}

/// ActualTypeValue only represents the concrete value of a type. The actual type def is defined by
#[derive(Clone)]
pub enum ActualTypeValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct(HashMap<Symbol, Box<ActualTypeValue>>),
    Option(Option<Box<ActualTypeValue>>),
    Result(Result<Box<ActualTypeValue>, Box<ActualTypeValue>>),
    Function(Box<AstNode>), // Function contains only an execution body,
    // Reference counted values (everything afaik)
    Weak(Weak<ActualTypeValue>),
    Strong(Rc<ActualTypeValue>),
}

impl ActualTypeValue {}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub name: Symbol,
    pub params: Vec<(Symbol, TypeSymbol)>,
    pub return_type: Box<TypeSymbol>,
    pub body: Option<Box<AstNode>>,
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
        write!(
            f,
            "fn {}({}): {}",
            self.name,
            self.params
                .iter()
                .map(|p| format!("{}: {}", p.0, p.1))
                .collect::<Vec<String>>()
                .join(", "),
            self.return_type
        )
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone)]
pub struct TypeSymbol {
    is_weak: bool,
    type_of: TypeSymbolType,
    resolved: bool,
    inferred: bool,
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
