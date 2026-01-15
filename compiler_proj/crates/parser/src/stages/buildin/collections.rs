use std::{cell::RefCell, collections::HashMap, rc::Rc};

use parser_macros::{BuiltinStruct, expose_funcs};
use parser_types::{BuiltinStruct, Error, Instantiable, InterpreterValue, Scope, TypeSymbolType, instantiate_struct_as_t};

use crate::Optional;

#[derive(Debug, BuiltinStruct)]
pub struct BuiltinList {
    pub container: Vec<InterpreterValue>,
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[expose_funcs]
impl BuiltinList {
    #[expose]
    pub fn get(&mut self, idx: InterpreterValue) -> Result<InterpreterValue, Error> {
        if let InterpreterValue::Int(idx) = idx.deref_value()? {
            let (instance, optional_ref) =
                instantiate_struct_as_t!(self.scope, "Optional" => Optional, HashMap::new());

            let result = self.container.get(idx as usize);

            if let Some(value) = result {
                (*optional_ref).set(value.clone())?;
            }
            Ok(instance)
        } else {
            Err(Error::WrongType(
                "idx".to_owned(),
                "int".to_owned(),
                format!("{idx}"),
            ))
        }
    }

    #[expose]
    pub fn set(
        &mut self,
        idx: InterpreterValue,
        value: InterpreterValue,
    ) -> Result<InterpreterValue, Error> {
        if let InterpreterValue::Int(idx) = idx.deref_value()? {
            if self.container.len() > idx as usize {
                self.container[idx as usize] = value;
                Ok(InterpreterValue::Empty)
            } else {
                Err(Error::OperationUnsupported {
                    operation: "set".to_owned(),
                    type_of: "index out of bounds".to_owned(),
                })
            }
        } else {
            Err(Error::WrongType(
                "idx".to_owned(),
                "int".to_owned(),
                format!("{idx}"),
            ))
        }
    }

    #[expose]
    pub fn push(&mut self, value: InterpreterValue) -> Result<InterpreterValue, Error> {
        self.container.push(value);
        Ok(InterpreterValue::Empty)
    }

    #[expose]
    pub fn pop(&mut self) -> Result<InterpreterValue, Error> {
        let value = self.container.pop();
        if let Some(value) = value {
            Ok(value)
        } else {
            Err(Error::CantBeEmpty)
        }
    }

    #[expose]
    pub fn to_list(&self) -> Result<InterpreterValue, Error> {
        Ok(InterpreterValue::List(self.container.clone()))
    }

    #[expose]
    pub fn from_list(&mut self, list: InterpreterValue) -> Result<InterpreterValue, Error> {
        if let InterpreterValue::List(contents) = list.deref_value()? {
            self.container = contents;
            Ok(InterpreterValue::Empty)
        } else {
            Err(Error::OperationUnsupported {
                operation: "from_list".to_owned(),
                type_of: "list must be a list".to_owned(),
            })
        }
    }
}
