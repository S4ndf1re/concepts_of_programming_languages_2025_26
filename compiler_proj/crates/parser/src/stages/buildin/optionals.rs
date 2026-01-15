use std::{cell::RefCell, rc::Rc};

use parser_macros::{BuiltinStruct, expose_funcs};
use parser_types::{Error, InterpreterValue, Scope};

#[derive(Debug, BuiltinStruct)]
pub struct Optional {
    pub value: Option<InterpreterValue>,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[expose_funcs]
impl Optional {
    #[expose]
    pub fn set(&mut self, value: InterpreterValue) -> Result<InterpreterValue, Error> {
        self.value = Some(value);

        Ok(InterpreterValue::Empty)
    }

    #[expose]
    pub fn is_some(&self) -> Result<InterpreterValue, Error> {
        Ok(InterpreterValue::Bool(self.value.is_some()))
    }

    #[expose]
    pub fn is_none(&self) -> Result<InterpreterValue, Error> {
        Ok(InterpreterValue::Bool(self.value.is_none()))
    }

    #[expose]
    pub fn get(&self) -> Result<InterpreterValue, Error> {
        if let Some(value) = self.value.clone() {
            Ok(value)
        } else {
            Err(Error::OptionIsNone)
        }
    }
}

#[derive(Debug, BuiltinStruct)]
pub struct CustomResult {
    // NOTE(Jan): this looks weird, but the problem is, that the value must be defaultable, hence, option is needed
    pub value: Option<Result<InterpreterValue, InterpreterValue>>,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[expose_funcs]
impl CustomResult {
    #[expose]
    pub fn set_ok(&mut self, value: InterpreterValue) -> Result<InterpreterValue, Error> {
        if let Some(result) = &mut self.value {
            *result = Ok(value);
        } else {
            self.value = Some(Ok(value))
        }

        Ok(InterpreterValue::Empty)
    }

    #[expose]
    pub fn is_ok(&self) -> Result<InterpreterValue, Error> {
        if let Some(result) = &self.value {
            Ok(InterpreterValue::Bool(result.is_ok()))
        } else {
            Ok(InterpreterValue::Bool(false))
        }
    }

    #[expose]
    pub fn is_err(&self) -> Result<InterpreterValue, Error> {
        if let Some(result) = &self.value {
            Ok(InterpreterValue::Bool(result.is_err()))
        } else {
            Ok(InterpreterValue::Bool(true))
        }
    }

    #[expose]
    pub fn get_ok(&self) -> Result<InterpreterValue, Error> {
        if let Some(result) = &self.value
            && let Ok(value) = result
        {
            Ok(value.clone())
        } else {
            Err(Error::ResultIsErr)
        }
    }

    #[expose]
    pub fn get_err(&self) -> Result<InterpreterValue, Error> {
        if let Some(result) = &self.value
            && let Err(value) = result
        {
            Ok(value.clone())
        } else {
            Err(Error::ResultIsOk)
        }
    }
}
