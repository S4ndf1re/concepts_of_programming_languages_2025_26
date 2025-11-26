use std::{collections::HashMap, rc::{Rc, Weak}};

use crate::Symbol;

/// ActualTypeValue only represents the concrete value of a type. The actual type def is defined by
#[derive(Clone)]
pub enum InterpreterValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct(HashMap<Symbol, Box<InterpreterValue>>),
    Option(Option<Box<InterpreterValue>>),
    Result(Result<Box<InterpreterValue>, Box<InterpreterValue>>),
    Function, // Functions execution body is contained in its type definition,
    // Reference counted values (everything afaik)
    Weak(Weak<InterpreterValue>),
    Strong(Rc<InterpreterValue>),
}

impl InterpreterValue {}
