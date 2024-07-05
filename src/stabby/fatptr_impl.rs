use crate::{
    derive_ReprC,
    ඞ::{CLayoutOf, ConcreteReprC, ReprC},
};
use stabby::abi::{vtable::HasDropVt, IPtrOwned};

unsafe impl<Ptr: ConcreteReprC + IPtrOwned + ReprC, VTable: HasDropVt + ReprC> ReprC
    for stabby::abi::Dyn<'_, Ptr, VTable>
{
    type CLayout = CLayoutOf<StabbyDyn<Ptr, VTable>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        StabbyDyn::<Ptr, VTable>::is_valid(it)
    }
}

unsafe impl<'a, VTable: HasDropVt + ReprC> ReprC for stabby::abi::DynRef<'a, VTable> {
    type CLayout = CLayoutOf<StabbyDyn<&'a (), VTable>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        StabbyDyn::<&'a (), VTable>::is_valid(it)
    }
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug)]
pub struct StabbyDyn<Ptr, VTable: 'static> {
    pub data: Ptr,
    pub vtable: &'static VTable,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VtDrop {
    pub v_drop: extern "C" fn(*const ()),
}
unsafe impl ReprC for stabby::abi::vtable::VtDrop {
    type CLayout = CLayoutOf<VtDrop>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        VtDrop::is_valid(it)
    }
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VTable<Head, Tail> {
    /// The rest of the vtable.
    ///
    /// It comes first to allow upcasting vtables.
    pub tail: Tail,
    /// The head of the vtable (the last trait listed in the macros)
    pub head: Head,
}
unsafe impl<Head, Tail> ReprC for stabby::abi::vtable::VTable<Head, Tail>
where
    VTable<Head, Tail>: ReprC,
{
    type CLayout = CLayoutOf<VTable<Head, Tail>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        VTable::<Head, Tail>::is_valid(it)
    }
}

unsafe impl<Tail: ReprC> ReprC for stabby::abi::vtable::VtSend<Tail> {
    type CLayout = CLayoutOf<Tail>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        Tail::is_valid(it)
    }
}

unsafe impl<Tail: ReprC> ReprC for stabby::abi::vtable::VtSync<Tail> {
    type CLayout = CLayoutOf<Tail>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        Tail::is_valid(it)
    }
}

unsafe impl<T: ReprC, Cond> ReprC for stabby::abi::StableIf<T, Cond> {
    type CLayout = CLayoutOf<T>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        T::is_valid(it)
    }
}

unsafe impl<T: ReprC, As> ReprC for stabby::abi::StableLike<T, As> {
    type CLayout = CLayoutOf<T>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        T::is_valid(it)
    }
}
