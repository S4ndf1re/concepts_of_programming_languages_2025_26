use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{AstTypeDefinition, Symbol};


// TODO: Set value to something relevant
pub type Value = String;

pub struct Scope {
    parent_scope: Option<Rc<RefCell<Scope>>>,
    symbols: HashMap<Symbol, (AstTypeDefinition, Value)>
}



impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
        Self {
            parent_scope: parent,
            symbols: HashMap::new(),
        }
    }

    pub fn add_symbol_with_value(&mut self, name: Symbol, value: Value, type_of: AstTypeDefinition) {
        self.symbols.insert(name, (type_of, value));
    }


    pub fn set_existing_symbol_value(&mut self, name: Symbol, value: Value, type_of_value: AstTypeDefinition) -> bool {
        todo!()
    }
}