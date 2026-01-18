use ecs::{EntityIndex, World};

use crate::Symbol;

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum QueryCond {
    Component(Symbol),
    Not(Box<QueryCond>),
    And(Box<QueryCond>, Box<QueryCond>),
    Or(Box<QueryCond>, Box<QueryCond>),
}

impl QueryCond {
    pub fn get_dependent_symbols(&self) -> Vec<&Symbol> {
        match self {
            QueryCond::Component(s) => vec![s],
            QueryCond::Not(cond) => cond.get_dependent_symbols(),
            QueryCond::And(c1, c2) => c1
                .get_dependent_symbols()
                .into_iter()
                .chain(c2.get_dependent_symbols())
                .collect::<Vec<_>>(),
            QueryCond::Or(c1, c2) => c1
                .get_dependent_symbols()
                .into_iter()
                .chain(c2.get_dependent_symbols())
                .collect::<Vec<_>>(),
        }
    }

    pub fn entity_conforms_condition(&self, entity: EntityIndex, world: &World) -> bool {
        match self {
            Self::Component(comp) => {
                if let Some(entt) = world.get_entity_mut(entity) {
                    if comp == "Entity" {
                        true
                    } else {
                        entt.has_component_by_name(comp)
                    }
                } else {
                    false
                }
            }
            Self::Not(cond) => !cond.entity_conforms_condition(entity, world),
            Self::And(left, right) => {
                left.entity_conforms_condition(entity, world)
                    && right.entity_conforms_condition(entity, world)
            }
            Self::Or(left, right) => {
                left.entity_conforms_condition(entity, world)
                    || right.entity_conforms_condition(entity, world)
            }
        }
    }
}
