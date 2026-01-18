use std::marker::PhantomData;

use crate::{Component, EntityIndex, SystemParameter};

pub struct SingleQuery<'s, T> {
    pub components: Vec<(EntityIndex, &'s mut T)>,
    _data: PhantomData<T>,
}

pub struct SingleQueryEntities {
    entities: Vec<EntityIndex>,
}

impl<'s, T> SystemParameter for SingleQuery<'s, T>
where
    T: Component + 'static,
{
    type Item<'w> = SingleQuery<'w, T>;
    type State = SingleQueryEntities;

    fn get_param<'w>(state: &mut Self::State, world: &'w crate::World) -> Self::Item<'w> {
        let mut components = Vec::new();
        for entity in &state.entities {
            if let Some(mut entt) = world.get_entity_mut(*entity)
                && let Some(comp) = entt.get_component_mut::<T>()
            {
                components.push((entt.id(), comp));
            }
        }

        SingleQuery {
            components,
            _data: PhantomData,
        }
    }

    fn instantiate_from_world(world: &crate::World) -> Self::State {
        let mut entities = vec![];

        for entity in world.get_entites() {
            if let Some(entt) = world.get_entity_mut(entity)
                && entt.has_component::<T>()
            {
                entities.push(entity);
            }
        }

        SingleQueryEntities { entities }
    }
}
