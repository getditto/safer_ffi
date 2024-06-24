use crate::{
    CVoid, Tuple2_Layout,
    ඞ::{ReprC, __HasNiche__},
};
pub use stabby::sync::{Arc, ArcSlice, ArcStr, Weak, WeakSlice, WeakStr};
use stabby::{alloc::IAlloc, IStable};

type USIZE = <usize as IStable>::Size;

unsafe impl<T: IStable, Alloc: IStable + IAlloc> ReprC for Arc<T, Alloc>
where
    Arc<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = *const CVoid;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it) && it.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable, Alloc: IStable + IAlloc> __HasNiche__ for Arc<T, Alloc>
where
    Arc<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it.is_null()
    }
}

unsafe impl<T: IStable, Alloc: IStable + IAlloc> ReprC for ArcSlice<T, Alloc>
where
    Arc<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = Tuple2_Layout<*const CVoid, *const CVoid>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it)
            && it._0.align_offset(core::mem::align_of::<T>()) == 0
            && it._1.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable, Alloc: IStable + IAlloc> __HasNiche__ for ArcSlice<T, Alloc>
where
    Arc<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it._0.is_null() || it._1.is_null()
    }
}
unsafe impl<Alloc: IStable + IAlloc> ReprC for ArcStr<Alloc>
where
    ArcSlice<u8, Alloc>: ReprC,
{
    type CLayout = <ArcSlice<u8, Alloc> as ReprC>::CLayout;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        ArcSlice::<u8, Alloc>::is_valid(it)
    }
}
unsafe impl<Alloc: IStable + IAlloc> __HasNiche__ for ArcStr<Alloc>
where
    ArcSlice<u8, Alloc>: __HasNiche__,
{
    fn is_niche(it: &'_ Self::CLayout) -> bool {
        ArcSlice::<u8, Alloc>::is_niche(it)
    }
}

unsafe impl<T: IStable, Alloc: IStable + IAlloc> ReprC for Weak<T, Alloc>
where
    Weak<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = *const CVoid;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it) && it.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable, Alloc: IStable + IAlloc> __HasNiche__ for Weak<T, Alloc>
where
    Weak<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it.is_null()
    }
}

unsafe impl<T: IStable, Alloc: IStable + IAlloc> ReprC for WeakSlice<T, Alloc>
where
    Weak<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = Tuple2_Layout<*const CVoid, *const CVoid>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it)
            && it._0.align_offset(core::mem::align_of::<T>()) == 0
            && it._1.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable, Alloc: IStable + IAlloc> __HasNiche__ for WeakSlice<T, Alloc>
where
    Weak<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it._0.is_null() || it._1.is_null()
    }
}
unsafe impl<Alloc: IStable + IAlloc> ReprC for WeakStr<Alloc>
where
    WeakSlice<u8, Alloc>: ReprC,
{
    type CLayout = <WeakSlice<u8, Alloc> as ReprC>::CLayout;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        WeakSlice::<u8, Alloc>::is_valid(it)
    }
}
unsafe impl<Alloc: IStable + IAlloc> __HasNiche__ for WeakStr<Alloc>
where
    WeakSlice<u8, Alloc>: __HasNiche__,
{
    fn is_niche(it: &'_ Self::CLayout) -> bool {
        WeakSlice::<u8, Alloc>::is_niche(it)
    }
}
