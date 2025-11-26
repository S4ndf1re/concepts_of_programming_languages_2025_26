use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{Error, StructType, Symbol, TypeSymbol, TypeSymbolType};

/// ActualTypeValue only represents the concrete value of a type. The actual type def is defined by
#[derive(Clone)]
pub enum InterpreterValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct(Symbol, HashMap<Symbol, Box<InterpreterValue>>),
    Option(Option<Box<InterpreterValue>>),
    Result(Result<Box<InterpreterValue>, Box<InterpreterValue>>),
    Function(Symbol), // Functions execution body is contained in its type definition,
    // Reference counted values (everything afaik)
    Weak(Weak<InterpreterValue>),
    Strong(Rc<InterpreterValue>),

    // Represents nothing, i.e. no value is returned
    Empty,
}

impl InterpreterValue {
    pub fn new_strong(inner: InterpreterValue) -> InterpreterValue {
        Self::Strong(Rc::new(inner))
    }

    pub fn downgrade(&self) -> Result<InterpreterValue, Error> {
        if let InterpreterValue::Strong(s) = self {
            Ok(InterpreterValue::Weak(Rc::downgrade(s)))
        } else {
            Err(Error::CantDowncastToWeak)
        }
    }

    pub fn upgrade(&self) -> Result<InterpreterValue, Error> {
        if let InterpreterValue::Weak(s) = self
            && let Some(upgraded) = s.upgrade()
        {
            Ok(InterpreterValue::Strong(upgraded))
        } else {
            Err(Error::CantDowncastToWeak)
        }
    }

    pub fn must_upgrade_before_deref(&self) -> bool {
        if let InterpreterValue::Weak(_) = self {
            true
        } else {
            false
        }
    }

    pub fn deref(&self) -> Result<&InterpreterValue, Error> {
        if let InterpreterValue::Strong(s) = self {
            Ok(s.as_ref())
        } else {
            Err(Error::CantDerefWeak)
        }
    }


    fn preprocess_for_operation(mut a: Self, mut b: Self) -> Result<(InterpreterValue, InterpreterValue), Error> {
        if a.must_upgrade_before_deref() {
            a = a.upgrade()?;
        }

        if b.must_upgrade_before_deref() {
            b = b.upgrade()?;
        }

        let lval = a.deref()?;
        let rval = a.deref()?;

        // TODO: Performance optimization
        Ok((lval.clone(), rval.clone()))
    }

    pub fn add(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Int(l + r)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l as f64 + r)),
                InterpreterValue::String(r) => Ok(InterpreterValue::String(format!("{}{}", l, r))),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Float(l + r as f64)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l + r)),
                InterpreterValue::String(r) => Ok(InterpreterValue::String(format!("{}{}", l, r))),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::String(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::String(format!("{}{}", l, r))),
                InterpreterValue::Float(r) => Ok(InterpreterValue::String(format!("{}{}", l, r))),
                InterpreterValue::String(r) => Ok(InterpreterValue::String(format!("{}{}", l, r))),
                _ => Err(Error::CantBeEmpty),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn subtract(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Int(l - r)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l as f64 - r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Float(l - r as f64)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l - r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn multiply(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Int(l * r)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l as f64 * r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Float(l * r as f64)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l * r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn divide(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Int(l / r)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l as f64 / r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Float(l / r as f64)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l / r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }
}

impl From<InterpreterValue> for Option<TypeSymbol> {
    fn from(value: InterpreterValue) -> Self {
        match value {
            InterpreterValue::Int(_) => Some(TypeSymbol::strong(TypeSymbolType::Int)),
            InterpreterValue::Float(_) => Some(TypeSymbol::strong(TypeSymbolType::Float)),
            InterpreterValue::Bool(_) => Some(TypeSymbol::strong(TypeSymbolType::Bool)),
            InterpreterValue::String(_) => Some(TypeSymbol::strong(TypeSymbolType::String)),
            InterpreterValue::Struct(name, fields) => {
                Some(TypeSymbol::strong(TypeSymbolType::Struct(StructType {
                    name,
                    fields: fields
                        .iter()
                        .map(|v| {
                            (
                                v.0.clone(),
                                Into::<Option<TypeSymbol>>::into(v.1.as_ref().clone()),
                            )
                        })
                        .filter(|v| v.1.is_some())
                        .map(|v| (v.0, v.1.expect("must be some")))
                        .collect::<Vec<_>>(),
                    methods: vec![],
                    statics: vec![],
                })))
            }

            _ => None,
        }
    }
}
