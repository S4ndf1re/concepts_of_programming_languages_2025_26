pub mod query_cond;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ecs::{EntityIndex, World};
pub use query_cond::*;

pub mod query_type;
pub use query_type::*;

pub mod query_term;
pub use query_term::*;

use crate::{
    BuiltinStruct, Error, Instantiable, InterpreterValue, Scope, Symbol, TypeSymbolType, WorldObj,
    instantiate_as_t,
};

/// This is not an actual system parameter, but one that can be used when access to the world exists.
/// The main difference is, that instantiate from world takes &self.
/// This is sometimes needed, when calling systems with completely dynamic system params.
/// This can still be used with worlds, but not the same way normal systems can be used (using `SystemFn`s)
pub trait PseudoSystemParameter {
    type State;
    type Item<'w>: PseudoSystemParameter<State = Self::State>;

    fn instantiate_from_world(&self, world: &World) -> Self::State;

    fn get_param<'w>(
        state: &mut Self::State,
        world: &'w World,
        scope: Rc<RefCell<Scope>>,
    ) -> Result<Self::Item<'w>, Error>;
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Query {
    pub symbol: Symbol,
    pub type_of: QueryType,
}

#[allow(unused)]
pub struct QueryItem {
    symbol: Symbol,
    type_of: QueryType,
    pub components: InterpreterValue,
}

impl PseudoSystemParameter for QueryItem {
    type Item<'w> = QueryItem;
    type State = QueryState;

    fn instantiate_from_world(&self, _world: &ecs::World) -> Self::State {
        todo!()
    }

    fn get_param<'w>(
        _state: &mut Self::State,
        _world: &'w ecs::World,
        _scope: Rc<RefCell<Scope>>,
    ) -> Result<Self::Item<'w>, Error> {
        todo!()
    }
}

pub struct QueryState {
    symbol: Symbol,
    type_of: QueryType,
    entities: Vec<EntityIndex>,
}

impl PseudoSystemParameter for Query {
    type State = QueryState;
    type Item<'w> = QueryItem;

    fn instantiate_from_world(&self, world: &ecs::World) -> Self::State {
        let mut entities = Vec::new();
        for entity in world.get_entites() {
            if self.type_of.entity_conforms_condition(entity, world) {
                entities.push(entity);

                // If single, only accept the first found entity. this is possibly nondeterministic.
                if matches!(
                    self.type_of,
                    QueryType::Single {
                        select: _,
                        condition: _
                    }
                ) {
                    break;
                }
            }
        }

        QueryState {
            symbol: self.symbol.clone(),
            type_of: self.type_of.clone(),
            entities,
        }
    }

    fn get_param<'w>(
        state: &mut Self::State,
        world: &'w ecs::World,
        scope: Rc<RefCell<Scope>>,
    ) -> Result<Self::Item<'w>, Error> {
        let mut components = Vec::new();

        if matches!(state.type_of, QueryType::World) {
            let (instance, _world_ref) =
                instantiate_as_t!(scope, "WorldObj" => WorldObj, HashMap::new());

            return Ok(QueryItem {
                symbol: state.symbol.clone(),
                type_of: state.type_of.clone(),
                components: instance,
            });
        }

        let requested_components = state.type_of.get_components();
        for entity in &state.entities {
            let Some(entt) = world.get_entity_mut(*entity) else {
                continue;
            };
            let mut entry_comps = Vec::new();

            for requested_comp in requested_components {
                if requested_comp == "Entity" {
                    entry_comps.push(InterpreterValue::Entity(entt.id()));
                } else if let Some(interpreter_value) =
                    entt.get_component_by_name::<InterpreterValue>(requested_comp)
                {
                    entry_comps.push(interpreter_value.clone());
                }
            }

            components.push(InterpreterValue::List(entry_comps));
        }

        Ok(QueryItem {
            symbol: state.symbol.clone(),
            type_of: state.type_of.clone(),
            components: InterpreterValue::List(components),
        })
    }
}
