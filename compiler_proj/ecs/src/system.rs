use std::marker::PhantomData;

use crate::World;

pub trait SystemParameter {
    type State;
    type Item<'w>;

    fn instantiate_from_world(world: &World) -> Self::State;

    fn get_parm<'w>(state: &mut Self::State, world: &'w World) -> Self::Item<'w>;
}

pub trait System {
    fn run(&mut self, world: &World);
}

pub struct IntoSystem<F, P> {
    pub func: F,
    pub marker: PhantomData<P>,
}

impl<F, P> System for IntoSystem<F, P>
where
    for<'a> F: FnMut(P::Item<'a>),
    P: SystemParameter,
{
    fn run(&mut self, world: &World) {
        let mut state = P::instantiate_from_world(world);
        (self.func)(P::get_parm(&mut state, world));
    }
}
