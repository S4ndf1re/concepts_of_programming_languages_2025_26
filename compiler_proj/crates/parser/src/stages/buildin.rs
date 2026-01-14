use std::{cell::RefCell, rc::Rc};

use ecs::World;

use parser_types::{BuildinCallback, BuiltinList, BuiltinStruct, Error, FunctionExecutionStrategy, FunctionType, InterpreterValue, IsReturn, Scope, ScopeLike, Symbol, TypeSymbol, TypeSymbolType};

pub fn println(scope: Rc<RefCell<Scope>>, _world: &World) -> Result<IsReturn, Error> {
    let scope = scope.borrow();
    let val = scope.resolve_value(&"val".to_string())?;
    println!("{val}");
    Ok(IsReturn::Return(InterpreterValue::Empty))
}

pub fn assert(scope: Rc<RefCell<Scope>>, _world: &World) -> Result<IsReturn, Error> {
    let scope = scope.borrow();
    let attr = scope.resolve_value(&"attr".to_string())?;
    assert!(attr.as_bool()?);
    Ok(IsReturn::Return(InterpreterValue::Empty))
}

pub fn stop(_scope: Rc<RefCell<Scope>>, world: &World) -> Result<IsReturn, Error> {
    world.stop();
    Ok(IsReturn::Return(InterpreterValue::Empty))
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
        let type_of = TypeSymbol::strong(TypeSymbolType::Function(FunctionType {
            name: self.name.clone(),
            is_method: false,
            execution_body: FunctionExecutionStrategy::Buildin(self.callback),
            params: self.params,
            return_type: self.return_type,
        }));

        scope.declare_function(self.name, value, type_of, true, true, 0..1)?;

        Ok(())
    }
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

    let list = BuiltinList {
        container: vec![],
        defining_scope: Rc::clone(&scope),
    };
    let name = list.name();
    let type_of = list.to_type()?;
    scope
        .borrow_mut()
        .declare_type(name, type_of, false, 0..1)?;

    Ok(())
}
