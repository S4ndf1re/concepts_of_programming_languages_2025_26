use std::fmt::Debug;

use crate::{Error, Instantiable, ScopeLike, TypeSymbol};

pub trait BuiltinStruct: Debug + ScopeLike + Instantiable {
    fn to_type(self) -> Result<TypeSymbol, Error>;
    fn resolve_builtin_type(&self) -> Option<TypeSymbol>;
    fn name(&self) -> String;
}
