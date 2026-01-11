pub mod system;
pub use system::*;

pub mod world;
pub use world::*;

pub mod queries;
pub use queries::*;

use std::collections::HashMap;

use typed_generational_arena::{Index, NonzeroGeneration};

pub type EntityIndex = Index<Entity, usize, NonzeroGeneration<usize>>;

#[derive(Default)]
pub struct Entity {
    components: HashMap<String, Box<dyn Component>>,
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
    entity: EntityIndex,
}

impl<'w> EntityCommandsMut<'w> {
    pub fn add_component<C: Component + 'static>(&mut self, component: C) {
        self.world.register_component::<C>();

        let c_boxed = Box::new(component);
        let boxed: Box<dyn Component> = Box::new(*c_boxed);
        if let Some(e) = self.world.entites.borrow_mut().get_mut(self.entity) {
            e.components.insert(C::ident(), boxed);
        }
    }

    pub fn remove_component<C: Component + 'static>(&mut self) {
        if let Some(e) = self.world.entites.borrow_mut().get_mut(self.entity) {
            e.components.remove(&C::ident());
        }
    }

    pub fn remove_component_by_value<C: Component + 'static>(&mut self, component: C) {
        if let Some(e) = self.world.entites.borrow_mut().get_mut(self.entity) {
            e.components.remove(&component.get_ident());
        }
    }

    pub fn has_component<C: Component + 'static>(&self) -> bool {
        self.world
            .entites
            .borrow()
            .get(self.entity)
            .is_some_and(|e| e.components.contains_key(&C::ident()))
    }

    pub fn has_component_by_value<C: Component + 'static>(&self, component: &C) -> bool {
        self.world
            .entites
            .borrow()
            .get(self.entity)
            .is_some_and(|e| e.components.contains_key(&component.get_ident()))
    }
    pub fn has_component_by_name(&self, name: &String) -> bool {
        self.world
            .entites
            .borrow()
            .get(self.entity)
            .is_some_and(|e| e.components.contains_key(name))
    }

    pub fn get_component_mut<C: Component + 'static>(&mut self) -> Option<&'w mut C> {
        if self.has_component::<C>() {
            let mut world_entities = self.world.entites.borrow_mut();
            let entity = world_entities.get_mut(self.entity).unwrap();

            let comp = entity.components.get_mut(&C::ident()).unwrap().as_mut();

            let any = comp as *mut dyn Component as *mut C;

            unsafe {
                let any_ref_mut = &mut *any;
                Some(any_ref_mut)
            }
        } else {
            None
        }
    }

    pub fn get_component_mut_by_value<C: Component + 'static>(
        &mut self,
        component: C,
    ) -> Option<&'w mut C> {
        if self.has_component_by_value(&component) {
            let mut world_entities = self.world.entites.borrow_mut();
            let entity = world_entities.get_mut(self.entity).unwrap();

            let comp = entity
                .components
                .get_mut(&component.get_ident())
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
    pub fn get_component_mut_by_name<C: Component + 'static>(
        &mut self,
        name: &String,
    ) -> Option<&'w mut C> {
        if self.has_component_by_name(name) {
            let mut world_entities = self.world.entites.borrow_mut();
            let entity = world_entities.get_mut(self.entity).unwrap();

            let comp = entity.components.get_mut(name).unwrap().as_mut();

            let any = comp as *mut dyn Component as *mut C;

            unsafe {
                let any_ref_mut = &mut *any;
                Some(any_ref_mut)
            }
        } else {
            None
        }
    }

    pub fn get_component<C: Component + 'static>(&self) -> Option<&'w C> {
        if self.has_component::<C>() {
            let world_entities = self.world.entites.borrow();
            let entity = world_entities.get(self.entity).unwrap();

            let comp = entity.components.get(&C::ident()).unwrap().as_ref();

            let any = comp as *const dyn Component as *const C;

            unsafe {
                let any_ref_mut = & *any;
                Some(any_ref_mut)
            }
        } else {
            None
        }
    }

    pub fn get_component_by_value<C: Component + 'static>(
        &self,
        component: C,
    ) -> Option<&'w C> {
        if self.has_component_by_value(&component) {
            let world_entities = self.world.entites.borrow();
            let entity = world_entities.get(self.entity).unwrap();

            let comp = entity
                .components
                .get(&component.get_ident())
                .unwrap()
                .as_ref();

            let any = comp as *const dyn Component as *const C;

            unsafe {
                let any_ref_mut = & *any;
                Some(any_ref_mut)
            }
        } else {
            None
        }
    }
    pub fn get_component_by_name<C: Component + 'static>(
        &self,
        name: &String,
    ) -> Option<&'w C> {
        if self.has_component_by_name(name) {
            let world_entities = self.world.entites.borrow();
            let entity = world_entities.get(self.entity).unwrap();

            let comp = entity.components.get(name).unwrap().as_ref();

            let any = comp as *const dyn Component as *const C;

            unsafe {
                let any_ref_mut = & *any;
                Some(any_ref_mut)
            }
        } else {
            None
        }
    }

    pub fn id(&self) -> EntityIndex {
        self.entity
    }
}

pub trait Component {
    fn ident() -> String
    where
        Self: Sized + 'static,
    {
        std::any::type_name::<Self>().to_owned()
    }

    fn get_ident(&self) -> String;
}
