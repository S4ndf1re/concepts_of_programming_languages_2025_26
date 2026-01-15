use std::{cell::RefCell, rc::Rc};

use ecs::World;

use parser_macros::{BuiltinStruct, expose_funcs};
use parser_types::{BuildinCallback, BuiltinStruct, Error, FunctionExecutionStrategy, FunctionType, InterpreterValue, IsReturn, Scope, ScopeLike, Symbol, TypeSymbol, TypeSymbolType};


#[derive(Debug, BuiltinStruct)]
pub struct BuiltinList {
    pub container: Vec<InterpreterValue>,
    #[scope]
    pub defining_scope: Rc<RefCell<Scope>>,
}

// // TODO: Find a way to actually insert the struct as a type in eval member call within the interpreter
// // TODO: implement proc macros to generate these stubs and derive BuiltinStruct
#[expose_funcs]
impl BuiltinList {
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
}

//     pub fn push_converted(scope: Rc<RefCell<Scope>>, _world: &World) -> Result<IsReturn, Error> {
//         let slf = scope.resolve_value(&"self".to_owned())?;
//         let value = scope.resolve_value(&"value".to_owned())?;

//         match &slf.deref_value()? {
//             InterpreterValue::BuiltinStruct(_name, ptr) => unsafe {
//                 let val = (&mut *ptr.borrow_mut()) as *mut dyn BuiltinStruct as *mut Self;
//                 (*val).push(value)?;
//             },
//             _ => Err(Error::OperationUnsupported {
//                 operation: "builtin".to_owned(),
//                 type_of: "must be BuiltinValue".to_owned(),
//             })?,
//         }

//         Ok(IsReturn::Return(InterpreterValue::Empty))
//     }

//     pub fn pop_converted(scope: Rc<RefCell<Scope>>, _world: &World) -> Result<IsReturn, Error> {
//         let slf = scope.resolve_value(&"self".to_owned())?;

//         match &mut slf.deref_value()? {
//             InterpreterValue::BuiltinStruct(_name, ptr) => unsafe {
//                 let val = (&mut *ptr.borrow_mut()) as *mut dyn BuiltinStruct as *mut BuiltinList;
//                 let value = (*val).pop()?;
//                 Ok(IsReturn::Return(value))
//             },
//             _ => Err(Error::OperationUnsupported {
//                 operation: "builtin".to_owned(),
//                 type_of: "must be BuiltinValue".to_owned(),
//             }),
//         }
//     }
// }

// impl ScopeLike for BuiltinList {
//     fn resolve_value(&self, name: &Symbol) -> Result<InterpreterValue, Error> {
//         if name == "push" || name == "pop" {
//             Ok(InterpreterValue::Function(name.clone()))
//         } else {
//             Err(Error::SymbolNotFound(name.clone()))
//         }
//     }

//     fn set_value(&mut self, name: &Symbol, _value: InterpreterValue) -> Result<(), Error> {
//         Err(Error::SymbolNotFound(name.clone()))
//     }

//     fn resolve_type(&self, name: &Symbol) -> Result<TypeSymbol, Error> {
//         let Some(struct_type) = self
//             .defining_scope
//             .borrow()
//             .resolve_defined_type(&self.name())
//         else {
//             return Err(Error::SymbolNotFound(self.name()));
//         };

//         match &struct_type.type_of {
//             TypeSymbolType::Struct(strct) => {
//                 let method_result = strct
//                     .methods
//                     .iter()
//                     .find(|f| &f.0 == name)
//                     .map(|v| TypeSymbol::strong(TypeSymbolType::Function(v.1.clone())));

//                 if let Some(method) = method_result {
//                     return Ok(method);
//                 }

//                 let static_result = strct
//                     .statics
//                     .iter()
//                     .find(|f| &f.0 == name)
//                     .map(|v| TypeSymbol::strong(TypeSymbolType::Function(v.1.clone())));

//                 if let Some(r#static) = static_result {
//                     return Ok(r#static);
//                 }

//                 let field_result = strct
//                     .fields
//                     .iter()
//                     .find(|f| &f.0 == name)
//                     .map(|v| v.1.clone());

//                 if let Some(field) = field_result {
//                     return Ok(field);
//                 }

//                 Err(Error::SymbolNotFound(name.clone()))
//             }
//             _ => Err(Error::SymbolNotFound(name.clone())),
//         }
//     }

//     fn get_outer_scope(&self) -> Result<Rc<RefCell<Scope>>, Error> {
//         Ok(Rc::clone(&self.defining_scope))
//     }
// }

// impl Instantiable for BuiltinList {
//     fn instantiate(
//         &self,
//         local_scope: Rc<RefCell<Scope>>,
//         _params: std::collections::HashMap<Symbol, Box<InterpreterValue>>,
//     ) -> Result<InterpreterValue, Error> {
//         let new_value = Self {
//             container: vec![],
//             defining_scope: local_scope,
//         };

//         Ok(InterpreterValue::Strong(Rc::new(RefCell::new(
//             InterpreterValue::BuiltinStruct(
//                 "BuiltinList".to_owned(),
//                 Rc::new(RefCell::new(new_value)),
//             ),
//         ))))
//     }

//     fn get_required_parameters(&self) -> std::collections::HashMap<Symbol, TypeSymbol> {
//         // is emtpy, as no args are required
//         HashMap::new()
//     }
// }

// impl BuiltinStruct for BuiltinList {
//     fn to_type(self) -> Result<TypeSymbol, Error> {
//         Ok(TypeSymbol::strong(TypeSymbolType::Struct(StructType {
//             name: self.name(),
//             methods: vec![
//                 (
//                     "push".to_owned(),
//                     FunctionType {
//                         name: "push".to_owned(),
//                         is_method: true,
//                         params: vec![("value".to_owned(), TypeSymbol::strong(TypeSymbolType::Any))],
//                         return_type: None,
//                         execution_body: FunctionExecutionStrategy::Buildin(Self::push_converted),
//                     },
//                 ),
//                 (
//                     "pop".to_owned(),
//                     FunctionType {
//                         name: "pop".to_owned(),
//                         is_method: true,
//                         params: vec![],
//                         return_type: None,
//                         execution_body: FunctionExecutionStrategy::Buildin(Self::pop_converted),
//                     },
//                 ),
//             ],
//             statics: vec![],
//             fields: vec![],
//             prefab: Some(Rc::new(self)),
//         })))
//     }

//     fn name(&self) -> String {
//         "BuiltinList".to_owned()
//     }

//     fn resolve_builtin_type(&self) -> Option<TypeSymbol> {
//         self.defining_scope
//             .borrow()
//             .resolve_defined_type(&self.name())
//     }
// }



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
