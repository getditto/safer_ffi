pub use stabby::*;

mod boxed_impl;
mod sync_impl;
mod fatptr_impl {
    use core::ops::Not;

    use crate::{
        CVoid, Tuple2,
        ඞ::{CLayoutOf, ConcreteReprC, ReprC},
    };
    use stabby::abi::{vtable::HasDropVt, IPtrOwned};

    unsafe impl<Ptr: ConcreteReprC + IPtrOwned, VTable: HasDropVt> ReprC
        for stabby::abi::Dyn<'_, Ptr, VTable>
    {
        type CLayout = CLayoutOf<Tuple2<Ptr, *const CVoid>>;
        fn is_valid(it: &'_ Self::CLayout) -> bool {
            Ptr::is_valid(&it._0) && it._1.is_null().not()
        }
    }

    unsafe impl<VTable: HasDropVt> ReprC for stabby::abi::DynRef<'_, VTable> {
        type CLayout = CLayoutOf<Tuple2<*const CVoid, *const CVoid>>;
        fn is_valid(it: &'_ Self::CLayout) -> bool {
            it._0.is_null().not() && it._1.is_null().not()
        }
    }
}
