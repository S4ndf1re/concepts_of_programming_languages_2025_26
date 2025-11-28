use std::{
    collections::HashMap,
    fmt::Display,
    rc::{Rc, Weak},
};

use ecs::Entity;
use typed_generational_arena::Index;

use crate::{Error, StructType, Symbol, TypeSymbol, TypeSymbolType};

/// ActualTypeValue only represents the concrete value of a type. The actual type def is defined by
#[derive(Clone, Debug)]
pub enum InterpreterValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<InterpreterValue>),
    Map(HashMap<InterpreterValue, InterpreterValue>),
    Struct(Symbol, HashMap<Symbol, Box<InterpreterValue>>),
    Option(Option<Box<InterpreterValue>>),
    Result(Result<Box<InterpreterValue>, Box<InterpreterValue>>),
    Function(Symbol), // Functions execution body is contained in its type definition,
    // Reference counted values (everything afaik)
    Weak(Weak<InterpreterValue>),
    Strong(Rc<InterpreterValue>),

    // ECS Intergration
    Entity(Index<Entity>),
    Component(Symbol, HashMap<Symbol, Box<InterpreterValue>>),

    // Represents nothing, i.e. no value is returned
    Empty,
}

impl InterpreterValue {
    pub fn new_strong(inner: InterpreterValue) -> InterpreterValue {
        Self::Strong(Rc::new(inner))
    }

    pub fn make_reference_counted(self) -> Result<InterpreterValue, Error> {
        match self {
            InterpreterValue::Strong(_) => Ok(self),
            InterpreterValue::Weak(_) => self.upgrade(),
            _ => Ok(InterpreterValue::new_strong(self)),
        }
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

    pub fn is_reference_counted(&self) -> bool {
        match self {
            InterpreterValue::Weak(_) => true,
            InterpreterValue::Strong(_) => true,
            _ => false,
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

    fn preprocess_for_operation(
        mut a: Self,
        mut b: Self,
    ) -> Result<(InterpreterValue, InterpreterValue), Error> {
        let lval = if a.is_reference_counted() {
            if a.must_upgrade_before_deref() {
                a = a.upgrade()?;
            }

            a.deref()?
        } else {
            &a
        };

        let rval = if b.is_reference_counted() {
            if b.must_upgrade_before_deref() {
                b = b.upgrade()?;
            }

            b.deref()?
        } else {
            &b
        };

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

    pub fn modulo(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Int(l % r)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l as f64 % r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Float(l % r as f64)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Float(l % r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn logical_and(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Bool(l) => match rval {
                InterpreterValue::Bool(r) => Ok(InterpreterValue::Bool(l && r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn logical_or(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Bool(l) => match rval {
                InterpreterValue::Bool(r) => Ok(InterpreterValue::Bool(l || r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn equals(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Bool(l) => match rval {
                InterpreterValue::Bool(r) => Ok(InterpreterValue::Bool(l == r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Bool(l == r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Float(r) => Ok(InterpreterValue::Bool(l == r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::String(l) => match rval {
                InterpreterValue::String(r) => Ok(InterpreterValue::Bool(l == r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Option(l) => match rval {
                InterpreterValue::Option(r) => {
                    // TODO: Optimize clone away
                    if let Some(l) = l.clone()
                        && let Some(r) = r.clone()
                    {
                        l.equals(*r)
                    } else if let None = l
                        && let None = r
                    {
                        Ok(InterpreterValue::Bool(true))
                    } else {
                        Ok(InterpreterValue::Bool(false))
                    }
                }
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Result(l) => match rval {
                InterpreterValue::Result(r) => {
                    // TODO: Optimize clone away
                    if let Ok(l) = l.clone()
                        && let Ok(r) = r.clone()
                    {
                        l.equals(*r)
                    } else if let Err(l) = l
                        && let Err(r) = r
                    {
                        l.equals(*r)
                    } else {
                        Ok(InterpreterValue::Bool(false))
                    }
                }
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Struct(l, lfields) => match rval {
                InterpreterValue::Struct(r, rfields) => {
                    // TODO: Optimize clone away
                    let mut eqls = true;
                    eqls = eqls && l == r;

                    for (l, lfield) in &lfields {
                        if let Some(rfield) = rfields.get(l)
                            // Must clone here, otherwise its a moved value in the next comparison
                            && let InterpreterValue::Bool(b) = lfield.clone().equals(*rfield.clone())?
                        {
                            eqls = eqls && b;
                        }
                    }

                    for (r, rfield) in rfields {
                        if let Some(lfield) = lfields.get(&r)
                            && let InterpreterValue::Bool(b) = rfield.equals(*lfield.clone())?
                        {
                            eqls = eqls && b;
                        }
                    }

                    Ok(InterpreterValue::Bool(eqls))
                }
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn not_equals(self, other: Self) -> Result<InterpreterValue, Error> {
        self.equals(other)?.negate_bool()
    }

    pub fn less_than(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Bool(l < r)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Bool((l as f64) < r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Bool(l < r as f64)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Bool(l < r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn less_than_equals(self, other: Self) -> Result<InterpreterValue, Error> {
        self.clone()
            .less_than(other.clone())?
            .logical_and(self.equals(other)?)
    }

    pub fn greater_than(self, other: Self) -> Result<InterpreterValue, Error> {
        let (lval, rval) = Self::preprocess_for_operation(self, other)?;

        match lval {
            InterpreterValue::Int(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Bool(l > r)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Bool((l as f64) > r)),
                _ => Err(Error::OperationUnsupported),
            },
            InterpreterValue::Float(l) => match rval {
                InterpreterValue::Int(r) => Ok(InterpreterValue::Bool(l > r as f64)),
                InterpreterValue::Float(r) => Ok(InterpreterValue::Bool(l > r)),
                _ => Err(Error::OperationUnsupported),
            },
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn greater_than_equals(self, other: Self) -> Result<InterpreterValue, Error> {
        self.clone()
            .greater_than(other.clone())?
            .logical_and(self.equals(other)?)
    }

    pub fn negate_bool(self) -> Result<InterpreterValue, Error> {
        match self {
            InterpreterValue::Bool(b) => Ok(InterpreterValue::Bool(!b)),
            _ => Err(Error::OperationUnsupported),
        }
    }
    pub fn negate_number(self) -> Result<InterpreterValue, Error> {
        match self {
            InterpreterValue::Int(i) => Ok(InterpreterValue::Int(-i)),
            InterpreterValue::Float(f) => Ok(InterpreterValue::Float(-f)),
            _ => Err(Error::OperationUnsupported),
        }
    }

    pub fn as_bool(&self) -> Result<bool, Error> {
        let v = match self {
            InterpreterValue::Bool(b) => *b,
            InterpreterValue::Strong(s) => s.as_bool()?,
            InterpreterValue::Weak(_) => self.upgrade()?.as_bool()?,
            _ => false,
        };

        Ok(v)
    }

    pub fn as_list(self) -> Result<Vec<InterpreterValue>, Error> {
        match self {
            InterpreterValue::List(l) => Ok(l),
            InterpreterValue::Strong(s) => Ok(s.as_ref().clone().as_list()?),
            InterpreterValue::Weak(_) => Ok(self.upgrade()?.as_list()?),
            _ => Err(Error::CantCastAsType("list".to_owned())),
        }
    }
}

impl From<InterpreterValue> for Option<TypeSymbol> {
    fn from(value: InterpreterValue) -> Self {
        let ts = match value {
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
            InterpreterValue::Strong(inner) => Into::<Option<TypeSymbol>>::into((*inner).clone()),
            InterpreterValue::Weak(_) => {
                let inner = value
                    .upgrade()
                    .expect("must be possible, otherwise this is a compiler error");
                Into::<Option<TypeSymbol>>::into(inner).map(|v| v.make_weak())
            }
            _ => None,
        };

        ts
    }
}

impl Display for InterpreterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterValue::Int(i) => write!(f, "{i}"),
            InterpreterValue::Float(fl) => write!(f, "{fl}"),
            InterpreterValue::Bool(b) => write!(f, "{b}"),
            InterpreterValue::String(s) => write!(f, "{s}"),
            InterpreterValue::Struct(name, fields) => {
                let fields = fields
                    .iter()
                    .map(|(name, value)| format!("{name}: {value}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{name} {{ {} }}", fields)
            }
            InterpreterValue::Strong(inner) => write!(f, "{inner}"),
            InterpreterValue::Weak(_) => {
                let inner = self
                    .upgrade()
                    .expect("must be possible, otherwise this is a compiler error");

                write!(f, "{inner}")
            }
            _ => std::fmt::Result::Err(std::fmt::Error),
        }
    }
}
