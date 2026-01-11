use ecs::{EntityIndex, PseudoSystemParameter};

use crate::{InterpreterValue, QueryType, Symbol};

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct Query {
    pub symbol: Symbol,
    pub type_of: QueryType,
}

pub struct QueryItem {
    symbol: Symbol,
    type_of: QueryType,
    pub components: Vec<Vec<InterpreterValue>>,
}

impl PseudoSystemParameter for QueryItem {
    type Item<'w> = QueryItem;
    type State = QueryState;

    fn instantiate_from_world(&self, _world: &ecs::World) -> Self::State {
        todo!()
    }

    fn get_param<'w>(_state: &mut Self::State, _world: &'w ecs::World) -> Self::Item<'w> {
        todo!()
    }
}

pub struct QueryState {
    symbol: Symbol,
    type_of: QueryType,
    entities: Vec<EntityIndex>,
}

impl PseudoSystemParameter for Query {
    type Item<'w> = QueryItem;
    type State = QueryState;

    fn instantiate_from_world(&self, world: &ecs::World) -> Self::State {
        let mut entities = Vec::new();
        for entity in world.get_entites() {
            if self.type_of.entity_conforms_condition(entity, world) {
                entities.push(entity);
            }
        }

        QueryState {
            symbol: self.symbol.clone(),
            type_of: self.type_of.clone(),
            entities,
        }
    }

    fn get_param<'w>(state: &mut Self::State, world: &'w ecs::World) -> Self::Item<'w> {
        let mut components = Vec::new();

        let requestes_comps = state.type_of.get_components();

        for entity in &state.entities {
            let Some(entt) = world.get_entity_mut(*entity) else {
                continue;
            };
            let mut entry_comps = Vec::new();

            for requested_comp in requestes_comps {
                if let Some(interpreter_value) =
                    entt.get_component_by_name::<InterpreterValue>(requested_comp)
                {
                    entry_comps.push(interpreter_value.clone());
                }
            }

            components.push(entry_comps);
        }

        QueryItem {
            symbol: state.symbol.clone(),
            type_of: state.type_of.clone(),
            components,
        }
    }
}
