use crate::{
    CVoid, Tuple2_Layout,
    ඞ::{ReprC, __HasNiche__},
};
pub use stabby::boxed::{Box, BoxedSlice, BoxedStr};
use stabby::{alloc::IAlloc, IStable};

type USIZE = <usize as IStable>::Size;

unsafe impl<T: IStable, Alloc: IStable + IAlloc> ReprC for Box<T, Alloc>
where
    Box<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = *const CVoid;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it) && it.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable, Alloc: IStable + IAlloc> __HasNiche__ for Box<T, Alloc>
where
    Box<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it.is_null()
    }
}

unsafe impl<T: IStable, Alloc: IStable + IAlloc> ReprC for BoxedSlice<T, Alloc>
where
    Box<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = Tuple2_Layout<*const CVoid, *const CVoid>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !Self::is_niche(it)
            && it._0.align_offset(core::mem::align_of::<T>()) == 0
            && it._1.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<T: IStable, Alloc: IStable + IAlloc> __HasNiche__ for BoxedSlice<T, Alloc>
where
    Box<T, Alloc>: IStable<Size = USIZE>,
{
    fn is_niche(it: &'_ <Self as ReprC>::CLayout) -> bool {
        it._0.is_null() || it._1.is_null()
    }
}
unsafe impl<Alloc: IStable + IAlloc> ReprC for BoxedStr<Alloc>
where
    BoxedSlice<u8, Alloc>: ReprC,
{
    type CLayout = <BoxedSlice<u8, Alloc> as ReprC>::CLayout;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        BoxedSlice::<u8, Alloc>::is_valid(it)
    }
}
unsafe impl<Alloc: IStable + IAlloc> __HasNiche__ for BoxedStr<Alloc>
where
    BoxedSlice<u8, Alloc>: __HasNiche__,
{
    fn is_niche(it: &'_ Self::CLayout) -> bool {
        BoxedSlice::<u8, Alloc>::is_niche(it)
    }
}
