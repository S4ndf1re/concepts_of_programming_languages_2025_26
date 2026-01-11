#![allow(non_snake_case)]

use pi_proc_macros::all_tuples;

use crate::World;

pub type SystemParamItem<'w, P> = <P as SystemParameter>::Item<'w>;

pub trait SystemParameter {
    type State;
    type Item<'w>: SystemParameter<State = Self::State>;

    fn instantiate_from_world(world: &World) -> Self::State;

    fn get_param<'w>(state: &mut Self::State, world: &'w World) -> Self::Item<'w>;
}

macro_rules! impl_param_tuples {
    ($($param: ident),*) => {
        impl<$($param : SystemParameter),*> SystemParameter for ($($param,)*) {
            type State = ($($param::State,)*);
            type Item<'w> = ($($param::Item<'w>,)*);

            #[allow(unused)]
            fn instantiate_from_world(world: &World) -> Self::State {
                #[allow(clippy::unused_unit)]
                (
                    $($param::instantiate_from_world(world),)*
                )
            }

            #[allow(unused)]
            fn get_param<'w>(state: &mut Self::State, world: &'w World) -> Self::Item<'w> {
                let ($($param,)*) = state;
                #[allow(clippy::unused_unit)]
                (
                    $($param::get_param($param, world),)*
                )
            }
        }
    };
}

all_tuples!(impl_param_tuples, 0, 16, P);


/// This is not an actual system parameter, but one that can be used when access to the world exists.
/// The main difference is, that instantiate from world takes &self.
/// This is sometimes needed, when calling systems with completely dynamic system params.
/// This can still be used with worlds, but not the same way normal systems can be used (using `SystemFn`s)
pub trait PseudoSystemParameter {
    type State;
    type Item<'w>: PseudoSystemParameter<State = Self::State>;

    fn instantiate_from_world(&self,world: &World) -> Self::State;

    fn get_param<'w>(state: &mut Self::State, world: &'w World) -> Self::Item<'w>;
}
