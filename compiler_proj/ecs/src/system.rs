use std::marker::PhantomData;

use crate::World;

pub trait SystemParameter {
    type State;
    type Item<'w>;

    fn instantiate_from_world(world: &World) -> Self::State;

    fn get_parm<'w>(state: &mut Self::State, world: &'w World) -> Self::Item<'w>;
}

pub trait SystemFn<Param, Out> {
    fn call(&mut self, p: Param) -> Out;
}

impl<F, A, Out> SystemFn<(A,), Out> for F
where
    F: FnMut(A) -> Out,
{
    fn call(&mut self, (a,): (A,)) -> Out {
        self(a)
    }
}
impl<F, A, B, Out> SystemFn<(A, B), Out> for F
where
    F: FnMut(A, B) -> Out,
{
    fn call(&mut self, (a, b): (A, B)) -> Out {
        self(a, b)
    }
}

pub trait System {
    fn run(&mut self, world: &World);
}

pub trait IntoBoxedSystem{
    fn into_boxed(self) -> Box<dyn System>;
}

pub struct SystemWrapper<F, P> {
    pub func: F,
    pub marker: PhantomData<P>,
}

impl<F, P> SystemWrapper<F, P> {
    pub fn new(f: F) -> Self {
        Self {
            func: f,
            marker: PhantomData,
        }
    }
}

impl<F, P> System for SystemWrapper<F, P>
where
    P: SystemParameter,
    for<'a> F: FnMut(P::Item<'a>),
{
    fn run(&mut self, world: &World) {
        let mut state = P::instantiate_from_world(world);
        (self.func)(P::get_parm(&mut state, world));
    }
}

impl<F, P1, P2> System for SystemWrapper<F, (P1, P2)>
where
    P1: SystemParameter,
    P2: SystemParameter,
    for<'a, 'b> F: FnMut(P1::Item<'a>, P2::Item<'b>),
{
    fn run(&mut self, world: &World) {
        let mut state1 = P1::instantiate_from_world(world);
        let mut state2 = P2::instantiate_from_world(world);
        (self.func)(
            P1::get_parm(&mut state1, world),
            P2::get_parm(&mut state2, world),
        );
    }
}

impl<F, P> From<F> for SystemWrapper<F, P>
where
    P: SystemParameter,
    for<'a> F: FnMut(P::Item<'a>),
{
    fn from(value: F) -> Self {
        Self {
            func: value,
            marker: PhantomData,
        }
    }
}

impl<F, P1, P2> From<F> for SystemWrapper<F, (P1, P2)>
where
    P1: SystemParameter,
    P2: SystemParameter,
    for<'a, 'b> F: FnMut(P1::Item<'a>, P2::Item<'b>),
{
    fn from(value: F) -> Self {
        Self {
            func: value,
            marker: PhantomData,
        }
    }
}


impl<SW> IntoBoxedSystem for SW
where SW: System + 'static {

    fn into_boxed(self) -> Box<dyn System> {
        let boxed = Box::new(self);
        Box::new(*boxed)
    }
}
