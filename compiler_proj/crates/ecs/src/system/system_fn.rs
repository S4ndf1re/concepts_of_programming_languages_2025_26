use pi_proc_macros::all_tuples;

use crate::{SystemParameter, SystemParamItem};


pub trait SystemFn<Marker>: 'static {
    type Out;
    type Param: SystemParameter;

    fn call<'w>(&mut self, item: <Self::Param as SystemParameter>::Item<'w>) -> Self::Out;
}


macro_rules! impl_system_fn {
    ($($param: ident), *) => {
        impl<Out, Func, $($param : SystemParameter),*> SystemFn<fn($($param,)*) -> Out> for Func
        where
            Func: 'static,
            for<'a> &'a mut Func: FnMut($($param),*) -> Out +
            FnMut($(SystemParamItem<$param>),*) -> Out,
            Out: 'static,
        {
            type Param = ($($param,)*);
            type Out = Out;

            #[inline]
            fn call<'w>(&mut self, param: SystemParamItem<($($param,)*)>) -> Self::Out {
                fn call_inner<Out, $($param,)*>(mut f: impl FnMut($($param,)*) -> Out, $($param: $param,)*) -> Out {
                    f($($param,)*)
                }
                let ($($param,)*) = param;
                call_inner(self, $($param),*)
            }
        }
    };
}

all_tuples!(impl_system_fn, 0, 16, F);
