use parser_types::{BuiltinStruct, Error, Scope, TypeSymbol, TypeSymbolType};
use std::{cell::RefCell, rc::Rc};

pub mod functions;
pub use functions::*;

pub mod collections;
pub use collections::*;

pub mod optionals;
pub use optionals::*;




pub fn register_buildin_struct<T: BuiltinStruct>(
    scope: Rc<RefCell<Scope>>,
    strct: T,
) -> Result<(), Error> {
    let name = strct.name();
    let type_of = strct.to_type()?;
    scope
        .borrow_mut()
        .declare_type(name, type_of, false, 0..1)?;
    Ok(())
}

pub fn register_buildin(scope: Rc<RefCell<Scope>>) -> Result<(), Error> {
    let println_descriptor = BuildinFunctionDescription {
        name: "println".to_string(),
        callback: println,
        params: vec![("val".to_string(), TypeSymbol::strong(TypeSymbolType::Any))],
        return_type: None,
    };
    println_descriptor.add_to_scope(&mut scope.borrow_mut())?;

    let assert_descriptor = BuildinFunctionDescription {
        name: "assert".to_string(),
        callback: assert,
        params: vec![("attr".to_string(), TypeSymbol::strong(TypeSymbolType::Bool))],
        return_type: None,
    };
    assert_descriptor.add_to_scope(&mut scope.borrow_mut())?;

    let stop_descriptor = BuildinFunctionDescription {
        name: "stop".to_string(),
        callback: stop,
        params: vec![],
        return_type: None,
    };
    stop_descriptor.add_to_scope(&mut scope.borrow_mut())?;

    register_buildin_struct(
        Rc::clone(&scope),
        BuiltinList {
            container: vec![],
            scope: Rc::clone(&scope),
        },
    )?;

    register_buildin_struct(
        Rc::clone(&scope),
        Optional {
            value: None,
            scope: Rc::clone(&scope),
        },
    )?;

    Ok(())
}
