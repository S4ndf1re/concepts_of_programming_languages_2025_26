use ecs::{EntityIndex, World};

use crate::Symbol;

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct QueryTerm {
    pub components: Vec<Symbol>,
}

impl QueryTerm {
    pub fn entity_conforms_condition(&self, entity: EntityIndex, world: &World) -> bool {
        let mut has_all = true;

        let Some(entity) = world.get_entity_mut(entity) else {
            return false;
        };

        for comp in &self.components {
            has_all = has_all && entity.has_component_by_name(comp);
            if !has_all {
                break;
            }
        }

        has_all
    }
}
