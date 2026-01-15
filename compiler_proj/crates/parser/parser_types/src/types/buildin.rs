use std::{cell::RefCell, fmt::Debug, rc::Rc};

use ecs::World;
use parser_macros::{BuiltinStruct, expose_funcs};

use crate::{Error, Instantiable, InterpreterValue, Scope, ScopeLike, TypeSymbol};

pub trait BuiltinStruct: Debug + ScopeLike + Instantiable {
    fn to_type(self) -> Result<TypeSymbol, Error>;
    fn resolve_builtin_type(&self) -> Option<TypeSymbol>;
    fn name(&self) -> String;
}

#[macro_export]
macro_rules! instantiate_as_t {
    ($s:expr, $name:expr => $type_as:ty, $params:expr) => {{
        let type_of = $s
            .borrow()
            .resolve_defined_type(&$name.to_owned())
            .ok_or(Error::SymbolNotFound($name.to_owned()))?;

        let TypeSymbolType::Struct(obj) = &type_of.type_of else {
            return Err(Error::SymbolNotFound($name.to_string()));
        };

        let instance = obj.instantiate(Rc::clone(&$s), $params)?;
        let InterpreterValue::BuiltinStruct(_, obj) = instance.deref_value()? else {
            return Err(Error::SymbolNotFound($name.to_string()));
        };

        unsafe {
            let obj_casted =
                &mut *(&mut *obj.borrow_mut() as *mut dyn BuiltinStruct as *mut $type_as);
            (instance, &mut *obj_casted)
        }
    }};
}

#[derive(Debug, BuiltinStruct)]
pub struct WorldObj {
    #[scope]
    pub scope: Rc<RefCell<Scope>>,
}

#[expose_funcs]
impl WorldObj {
    #[expose]
    pub fn stop(&self, world: &World) -> Result<InterpreterValue, Error> {
        world.stop();
        Ok(InterpreterValue::Empty)
    }
}
