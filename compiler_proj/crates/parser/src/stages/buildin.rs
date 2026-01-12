use std::{cell::RefCell, fmt::Debug, rc::Rc};

use ecs::World;

use crate::{
    BuildinCallback, Error, FunctionExecutionStrategy, FunctionType, InterpreterValue, IsReturn,
    Scope, ScopeLike, StructType, Symbol, TypeSymbol, TypeSymbolType,
};

pub trait BuiltinStruct: Debug + ScopeLike {
    fn to_type(&self) -> Result<TypeSymbol, Error>;
    fn instantiate(params: Vec<(Symbol, InterpreterValue)>) -> Result<InterpreterValue, Error>
    where
        Self: Sized;

    fn take_self_and_drop(self)
    where
        Self: Sized,
    {
    }
}

#[derive(Debug)]
pub struct BuiltinList {
    container: Vec<InterpreterValue>,
}

// TODO: Find a way to actually insert the struct as a type in eval member call within the interpreter
// TODO: implement proc macros to generate these stubs and derive BuiltinStruct
impl BuiltinList {
    pub fn push(&mut self, value: InterpreterValue) -> Result<(), Error> {
        self.container.push(value);
        Ok(())
    }
    pub fn pop(&mut self) -> Result<InterpreterValue, Error> {
        let value = self.container.pop();
        if let Some(value) = value {
            Ok(value)
        } else {
            Err(Error::CantBeEmpty)
        }
    }

    pub fn push_converted(scope: Rc<RefCell<Scope>>) -> Result<IsReturn, Error> {
        let slf = scope.resolve_value(&"self".to_owned())?;
        let value = scope.resolve_value(&"value".to_owned())?;

        match &slf.deref_value()? {
            InterpreterValue::BuiltinStruct(_name, ptr) => unsafe {
                let unboxxed_slf = *ptr as *mut (dyn BuiltinStruct + 'static) as *mut BuiltinList;
                let mut boxxed_slf = Box::from_raw(unboxxed_slf);

                boxxed_slf.push(value)?;

                Box::leak(boxxed_slf);
            },
            _ => Err(Error::OperationUnsupported {
                operation: "builtin".to_owned(),
                type_of: "must be BuiltinValue".to_owned(),
            })?,
        }

        Ok(IsReturn::Return(InterpreterValue::Empty))
    }

    pub fn pop_converted(scope: Rc<RefCell<Scope>>, _world: &World) -> Result<IsReturn, Error> {
        let slf = scope.resolve_value(&"self".to_owned())?;

        match &slf.deref_value()? {
            InterpreterValue::BuiltinStruct(_name, ptr) => unsafe {
                let unboxxed_slf = *ptr as *mut (dyn BuiltinStruct + 'static) as *mut BuiltinList;
                let mut boxxed_slf = Box::from_raw(unboxxed_slf);

                let value = boxxed_slf.pop()?;

                Box::leak(boxxed_slf);
                Ok(IsReturn::Return(value))
            },
            _ => Err(Error::OperationUnsupported {
                operation: "builtin".to_owned(),
                type_of: "must be BuiltinValue".to_owned(),
            }),
        }
    }
}

impl ScopeLike for BuiltinList {
    fn resolve_value(&self, name: &Symbol) -> Result<InterpreterValue, Error> {
        Err(Error::SymbolNotFound(name.clone()))
    }

    fn set_value(&mut self, name: &Symbol, _value: InterpreterValue) -> Result<(), Error> {
        Err(Error::SymbolNotFound(name.clone()))
    }

    fn resolve_type(&self, name: &Symbol) -> Result<TypeSymbol, Error> {
        Err(Error::SymbolNotFound(name.clone()))
    }

    fn get_outer_scope(&self) -> Result<Rc<RefCell<Scope>>, Error> {
        Err(Error::OperationUnsupported {
            operation: "can get scope on builtin".to_owned(),
            type_of: "".to_owned(),
        })
    }
}

impl BuiltinStruct for BuiltinList {
    fn to_type(&self) -> Result<TypeSymbol, Error> {
        Ok(TypeSymbol::strong(TypeSymbolType::Struct(StructType {
            name: "BuiltinList".to_owned(),
            methods: vec![(
                "push".to_owned(),
                FunctionType {
                    name: "push".to_owned(),
                    is_method: true,
                    params: vec![("value".to_owned(), TypeSymbol::strong(TypeSymbolType::Any))],
                    return_type: None,
                    execution_body: FunctionExecutionStrategy::Buildin(Self::pop_converted),
                },
            )],
            statics: vec![],
            fields: vec![],
        })))
    }

    fn instantiate(_params: Vec<(Symbol, InterpreterValue)>) -> Result<InterpreterValue, Error> {
        let boxxed = Box::new(Self { container: vec![] });
        Ok(InterpreterValue::Strong(Rc::new(RefCell::new(
            InterpreterValue::BuiltinStruct("BuiltinList".to_owned(), Box::leak(boxxed)),
        ))))
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
        let type_of = TypeSymbol::strong(crate::TypeSymbolType::Function(FunctionType {
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

    let stop_descriptor = BuildinFunctionDescription {
        name: "stop".to_string(),
        callback: stop,
        params: vec![],
        return_type: None,
    };
    stop_descriptor.add_to_scope(scope)?;

    Ok(())
}
