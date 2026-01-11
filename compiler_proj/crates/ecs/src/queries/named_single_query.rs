use std::marker::PhantomData;

use crate::{Component, EntityIndex, SystemParameter};

pub trait NamedComponent: Component + Default {
    const NAME: &'static str;

    fn get_ident(&self) -> String {
        Self::NAME.to_owned()
    }
}

pub struct NamedSingleQuery<'s, T> {
    pub components: Vec<&'s mut T>,
    _data: PhantomData<T>,
}

pub struct NamedSingleQueryEntities {
    entities: Vec<EntityIndex>,
}

impl<'s, T> SystemParameter for NamedSingleQuery<'s, T>
where
    T: NamedComponent + 'static,
{
    type Item<'w> = NamedSingleQuery<'w, T>;
    type State = NamedSingleQueryEntities;

    fn get_param<'w>(state: &mut Self::State, world: &'w crate::World) -> Self::Item<'w> {
        let mut components = Vec::new();
        for entity in &state.entities {
            if let Some(mut entt) = world.get_entity_mut(*entity)
                && let Some(comp) = entt.get_component_mut_by_value(T::default())
            {
                components.push(comp);
            }
        }

        NamedSingleQuery {
            components,
            _data: PhantomData,
        }
    }

    fn instantiate_from_world(world: &crate::World) -> Self::State {
        let mut entities = vec![];

        for entity in world.get_entites() {
            if let Some(entt) = world.get_entity_mut(entity)
                && entt.has_component_by_value(&T::default())
            {
                entities.push(entity);
            }
        }

        NamedSingleQueryEntities { entities }
    }
}
