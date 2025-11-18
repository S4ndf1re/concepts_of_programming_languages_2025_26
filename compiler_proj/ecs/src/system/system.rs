use std::marker::PhantomData;

use crate::{SystemFn, SystemParameter, World};



pub trait System {
    fn run(&mut self, world: &World);
}

pub trait IntoSystem<Marker> {
    fn into_system(self) -> Box<dyn System>;
}

pub struct SystemWrapper<Marker, F>
where
    F: SystemFn<Marker>,
{
    pub func: F,
    marker: PhantomData<fn() -> Marker>,
}

impl<Marker, F> SystemWrapper<Marker, F>
where
    F: SystemFn<Marker>,
{
    pub fn new(f: F) -> Self {
        Self {
            func: f,
            marker: PhantomData,
        }
    }
}

impl<Marker, F> System for SystemWrapper<Marker, F>
where
    F: SystemFn<Marker>,
{
    fn run(&mut self, world: &World) {
        let mut state = F::Param::instantiate_from_world(world);
        self.func.call(F::Param::get_param(&mut state, world));
    }
}

impl<F, Marker> IntoSystem<Marker> for F
where
    F: SystemFn<Marker>,
    Marker: 'static,
{
    fn into_system(self) -> Box<dyn System> {
        let wrapper = Box::new(SystemWrapper::new(self));

        Box::new(*wrapper)
    }
}
