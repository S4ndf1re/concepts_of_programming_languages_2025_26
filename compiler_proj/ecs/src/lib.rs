pub mod system;
pub use system::*;

pub mod world;
pub use world::*;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use typed_generational_arena::{Index, NonzeroGeneration};

#[derive(Default)]
pub struct Entity {
    components: HashMap<TypeId, Box<dyn Component>>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

pub struct EntityCommandsMut<'w> {
    world: &'w World,
    entity: Index<Entity, usize, NonzeroGeneration<usize>>,
}

impl<'w> EntityCommandsMut<'w> {
    pub fn add_component<C: Component + 'static>(&mut self, component: C) {
        self.world.register_component::<C>();

        let c_boxed = Box::new(component);
        let boxed: Box<dyn Component> = Box::new(*c_boxed);
        if let Some(e) = self.world.entites.borrow_mut().get_mut(self.entity) {
            e.components.insert(TypeId::of::<C>(), boxed);
        }
    }

    pub fn remove_component<C: Component + 'static>(&mut self) {
        if let Some(e) = self.world.entites.borrow_mut().get_mut(self.entity) {
            e.components.remove(&TypeId::of::<C>());
        }
    }

    pub fn has_component<C: Component + 'static>(&self) -> bool {
        self.world
            .entites
            .borrow()
            .get(self.entity)
            .is_some_and(|e| e.components.contains_key(&TypeId::of::<C>()))
    }

    pub fn get_component_mut<C: Component + 'static>(&mut self) -> Option<&'w mut C> {
        if self.has_component::<C>() {
            let mut world_entities = self.world.entites.borrow_mut();
            let entity = world_entities.get_mut(self.entity).unwrap();

            let comp = entity
                .components
                .get_mut(&TypeId::of::<C>())
                .unwrap()
                .as_mut();

            let any = comp as *mut dyn Component as *mut C;

            unsafe {
                let any_ref_mut = &mut *any;
                Some(any_ref_mut)
            }
        } else {
            None
        }
    }
}

pub trait Component {}
