use core::ptr::NonNull;

use crate::stabby::boxed_impl::AllocSlice;
use crate::ඞ::{CLayoutOf, ReprC, __HasNiche__};
pub use stabby::sync::{Arc, ArcSlice, ArcStr, Weak, WeakSlice, WeakStr};
use stabby::{alloc::IAlloc, IStable};

type USIZE = <usize as IStable>::Size;

unsafe impl<T: IStable + ReprC, Alloc: IStable + IAlloc> ReprC for Arc<T, Alloc>
where
    Arc<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = CLayoutOf<NonNull<T>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it) && it.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable + ReprC, Alloc: IStable + IAlloc> __HasNiche__ for Arc<T, Alloc>
where
    Arc<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it.is_null()
    }
}

unsafe impl<T: IStable + ReprC, Alloc: IStable + IAlloc> ReprC for ArcSlice<T, Alloc>
where
    Arc<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = CLayoutOf<AllocSlice<T>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !(it.start.is_null() || it.end.is_null())
            && it.start.align_offset(core::mem::align_of::<T>()) == 0
            && it.end.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<Alloc: IStable + IAlloc> ReprC for ArcStr<Alloc>
where
    ArcSlice<u8, Alloc>: ReprC,
{
    type CLayout = CLayoutOf<ArcSlice<u8, Alloc>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        ArcSlice::<u8, Alloc>::is_valid(it)
    }
}

unsafe impl<T: IStable + ReprC, Alloc: IStable + IAlloc> ReprC for Weak<T, Alloc>
where
    Weak<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = CLayoutOf<NonNull<T>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it) && it.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable + ReprC, Alloc: IStable + IAlloc> __HasNiche__ for Weak<T, Alloc>
where
    Weak<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it.is_null()
    }
}

unsafe impl<T: IStable + ReprC, Alloc: IStable + IAlloc> ReprC for WeakSlice<T, Alloc>
where
    Weak<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = CLayoutOf<ArcSlice<T, Alloc>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        ArcSlice::<T, Alloc>::is_valid(it)
    }
}
unsafe impl<Alloc: IStable + IAlloc> ReprC for WeakStr<Alloc>
where
    WeakSlice<u8, Alloc>: ReprC,
{
    type CLayout = CLayoutOf<WeakSlice<u8, Alloc>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        WeakSlice::<u8, Alloc>::is_valid(it)
    }
}
