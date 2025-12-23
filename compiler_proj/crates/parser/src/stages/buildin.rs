use std::{cell::RefCell, rc::Rc};

use crate::{
    BuildinCallback, Error, FunctionExecutionStrategy, FunctionType, InterpreterValue, IsReturn, Scope, Symbol, TypeSymbol, TypeSymbolType
};

pub fn println(scope: Rc<RefCell<Scope>>) -> Result<IsReturn, Error> {
    let scope = scope.borrow();
    if let Some(val) = scope.resolve_value(&"val".to_string()) {
        println!("{val}");
        Ok(IsReturn::Return(InterpreterValue::Empty))
    } else {
        Err(Error::SymbolNotFound("val".to_string()))
    }
}

pub fn assert(scope: Rc<RefCell<Scope>>) -> Result<IsReturn, Error> {
    let scope = scope.borrow();
    if let Some(attr) = scope.resolve_value(&"attr".to_string()) {
        assert!(attr.as_bool()?);
        Ok(IsReturn::Return(InterpreterValue::Empty))
    } else {
        Err(Error::SymbolNotFound("attr".to_string()))
    }
}

pub struct BuildinFunctionDescription {
    name: String,
    callback: BuildinCallback,
    params: Vec<(Symbol, TypeSymbol)>,
    return_type: Option<Box<TypeSymbol>>,
}

impl BuildinFunctionDescription {
    pub fn add_to_scope(self, scope: &mut Scope) -> Result<(), Error> {
        let value = InterpreterValue::Function(self.name.clone());
        let type_of = TypeSymbol::strong(crate::TypeSymbolType::Function(FunctionType {
            name: self.name.clone(),
            execution_body: FunctionExecutionStrategy::Buildin(self.callback),
            params: self.params,
            return_type: self.return_type,
        }));

        scope.declare_function(self.name, value, type_of, true, true, 0..1)?;

        Ok(())
    }
}

pub fn register_buildin(scope: &mut Scope) -> Result<(), Error> {
    let println_descriptor = BuildinFunctionDescription {
        name: "println".to_string(),
        callback: println,
        params: vec![("val".to_string(), TypeSymbol::strong(TypeSymbolType::Any))],
        return_type: None,
    };
    println_descriptor.add_to_scope(scope)?;

    let assert_descriptor = BuildinFunctionDescription {
        name: "assert".to_string(),
        callback: assert,
        params: vec![("attr".to_string(), TypeSymbol::strong(TypeSymbolType::Bool))],
        return_type: None,
    };
    assert_descriptor.add_to_scope(scope)?;

    Ok(())
}
