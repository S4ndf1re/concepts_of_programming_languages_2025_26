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

            fn instantiate_from_world(world: &World) -> Self::State {
                (
                    $($param::instantiate_from_world(world),)*
                )
            }

            fn get_param<'w>(state: &mut Self::State, world: &'w World) -> Self::Item<'w> {
                let ($($param,)*) = state;
                (
                    $($param::get_param($param, world),)*
                )
            }
        }
    };
}

all_tuples!(impl_param_tuples, 0, 16, P);
