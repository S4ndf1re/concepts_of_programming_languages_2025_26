use std::{any::TypeId, cell::RefCell, collections::HashSet, rc::Rc};

use typed_generational_arena::{Index, NonzeroGeneration, StandardArena};

use crate::{Component, Entity, EntityCommandsMut, IntoSystem, System, SystemParameter};

pub struct World {
    pub(crate) entites: Rc<RefCell<StandardArena<Entity>>>,
    pub(crate) components: Rc<RefCell<HashSet<TypeId>>>,
    pub(crate) systems: Vec<*const dyn System>,
}

impl World {
    pub fn register_component<C: Component + 'static>(&self) {
        let id = TypeId::of::<C>();
        if self.components.borrow().contains(&id) {
            return;
        }

        self.components.borrow_mut().insert(id);
    }

    pub fn spawn<'w>(&'w self) -> EntityCommandsMut<'w> {
        let entity = Entity::new();
        let idx = self.entites.borrow_mut().insert(entity);

        EntityCommandsMut {
            world: self,
            entity: idx,
        }
    }

    pub fn get_entites(&self) -> Vec<Index<Entity, usize, NonzeroGeneration<usize>>> {
        self.entites.borrow().iter().map(|(k, _)| k).collect()
    }

    pub fn get_entity_mut<'w>(
        &'w self,
        entity: Index<Entity, usize, NonzeroGeneration<usize>>,
    ) -> Option<EntityCommandsMut<'w>> {
        if self.entites.borrow().contains(entity) {
            Some(EntityCommandsMut {
                world: self,
                entity,
            })
        } else {
            None
        }
    }

    pub fn add_system<Marker: 'static, I: IntoSystem<Marker>>(&mut self, into_system: I) {
        self.systems.push(Box::into_raw(into_system.into_system()));
    }

    pub fn run(&mut self) {
        unsafe {
            let systems = self.systems.clone();
            loop {
                // SAFETY: This is safe, as long as no further system manipulation can take place
                for system in &systems {
                    let sys = *system as *mut dyn System;
                    (*sys).run(self);
                }
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            entites: Rc::new(RefCell::new(StandardArena::new())),
            components: Rc::new(RefCell::new(HashSet::new())),
            systems: Vec::new(),
        }
    }
}

impl Drop for World {
    fn drop(&mut self) {
        for system in &self.systems {
            unsafe {
                let _ = Box::from_raw(*system as *mut dyn System);
            }
        }
    }
}

impl SystemParameter for &World {
    type Item<'w> = &'w World;
    type State = ();

    fn instantiate_from_world(_: &World) -> Self::State {}

    fn get_param<'w>(_: &mut Self::State, world: &'w World) -> Self::Item<'w> {
        world
    }
}
