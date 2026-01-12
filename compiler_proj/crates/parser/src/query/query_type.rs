use ecs::{EntityIndex, World};

use crate::{QueryCond, QueryTerm, Symbol};

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum QueryType {
    List {
        select: QueryTerm,
        condition: Option<QueryCond>,
    },
    Single {
        select: QueryTerm,
        condition: Option<QueryCond>,
    },
    World,
    Resource(Symbol),
    EventReader(Symbol),
    EventWriter(Symbol),
}

impl QueryType {
    pub fn get_dependent_symbols(&self) -> Vec<&Symbol> {
        let mut result = Vec::new();
        match self {
            QueryType::List { select, condition } => {
                for symbol in &select.components {
                    result.push(symbol);
                }
                if let Some(cond) = condition {
                    result.extend(cond.get_dependent_symbols());
                }
            }
            QueryType::Single { select, condition } => {
                for symbol in &select.components {
                    result.push(symbol);
                }
                if let Some(cond) = condition {
                    result.extend(cond.get_dependent_symbols());
                }
            }
            QueryType::Resource(res) => result.push(res),
            QueryType::EventReader(evt) | QueryType::EventWriter(evt) => result.push(evt),
            _ => (),
        }

        result
    }

    pub fn entity_conforms_condition(&self, entity: EntityIndex, world: &World) -> bool {
        match self {
            Self::List { select, condition } => {
                select.entity_conforms_condition(entity, world)
                    && condition
                        .as_ref()
                        .map(|c| c.entity_conforms_condition(entity, world))
                        .unwrap_or(true)
            }
            Self::Single { select, condition } => {
                select.entity_conforms_condition(entity, world)
                    && condition
                        .as_ref()
                        .map(|c| c.entity_conforms_condition(entity, world))
                        .unwrap_or(true)
            }
            // false, because it is not really implemented yet
            _ => unimplemented!(),
        }
    }

    pub fn get_components(&self) -> &Vec<Symbol> {
        match self {
            Self::List {
                select,
                condition: _,
            }
            | Self::Single {
                select,
                condition: _,
            } => &select.components,
            _ => unimplemented!(),
        }
    }
}
