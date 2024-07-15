use crate::{
    CVoid,
    ඞ::{CLayoutOf, ReprC, __HasNiche__},
};
use safer_ffi_proc_macros::derive_ReprC;
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

unsafe impl<T: IStable + ReprC, Alloc: IStable + IAlloc> ReprC for BoxedSlice<T, Alloc>
where
    Box<T, Alloc>: IStable<Size = USIZE>,
{
    type CLayout = CLayoutOf<AllocSlice<T>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        !(it.start.is_null() || it.end.is_null())
            && it.start.align_offset(core::mem::align_of::<T>()) == 0
            && it.end.align_offset(core::mem::align_of::<T>()) == 0
    }
}
unsafe impl<Alloc: IStable + IAlloc> ReprC for BoxedStr<Alloc>
where
    BoxedSlice<u8, Alloc>: ReprC,
{
    type CLayout = CLayoutOf<BoxedSlice<u8, Alloc>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        BoxedSlice::<u8, Alloc>::is_valid(it)
    }
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct AllocSlice<T> {
    pub start: *const T,
    pub end: *const T,
}
