use stabby::IStable;

use crate::ඞ::{CType, ReprC, __HasNiche__};

/// Coherence wrapper for a blanket `ReprC` implementation off
/// a [`stabby::IStable`] one, without running into overlapping impls.
#[stabby::stabby]
pub struct Stabbied<T>(pub T);
unsafe impl<T: IStable> ReprC for Stabbied<T>
where
    T::CType: CType,
{
    type CLayout = T::CType;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !unsafe { T::is_invalid(it as *const _ as *const u8) }
    }
}
unsafe impl<T: IStable<HasExactlyOneNiche = stabby::abi::B1>> __HasNiche__ for Stabbied<T>
where
    Self: ReprC,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        !Self::is_valid(it)
    }
}

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
