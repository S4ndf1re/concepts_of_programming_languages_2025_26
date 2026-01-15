use std::{cell::RefCell, rc::Rc};

use ecs::World;

use parser_types::{
    BuildinCallback, Error, FunctionExecutionStrategy, FunctionType, InterpreterValue, IsReturn,
    Scope, ScopeLike, Symbol, TypeSymbol, TypeSymbolType,
};

pub struct BuildinFunctionDescription {
    pub name: String,
    pub callback: BuildinCallback,
    pub params: Vec<(Symbol, TypeSymbol)>,
    pub return_type: Option<Box<TypeSymbol>>,
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
