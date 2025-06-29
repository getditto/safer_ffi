//! See [the dedicated section of the guide](https://getditto.github.io/safer_ffi/dyn_traits/_.html).

use_prelude!();

#[path = "futures/_mod.rs"]
#[cfg(feature = "futures")]
#[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "futures")))]
pub mod futures;

#[doc(no_inline)]
pub use dyn_drop::*;

pub mod dyn_drop;

pub use self::ty::Erased as ErasedTy;

#[super::derive_ReprC]
#[repr(transparent)]
#[allow(missing_debug_implementations)]
pub struct ErasedRef<'a>(
    ptr::NonNullRef<ErasedTy>,
    ::core::marker::PhantomData<&'a ()>,
);

pub trait ReprCTrait {
    type VTable: ConcreteReprC;

    unsafe fn drop_ptr(
        ptr: ptr::NonNullOwned<ty::Erased>,
        vtable: &'_ Self::VTable,
    );
}

mod ty {
    #![allow(warnings)]

    #[super::derive_ReprC]
    #[repr(opaque)]
    pub struct Erased(());
}

pub trait VirtualPtrFrom<T>: ReprCTrait {
    fn into_virtual_ptr(this: T) -> VirtualPtr<Self>;
}

match_! {(
    ['r] &'r T,
    ['r] &'r mut T,
    ['r] ::core::pin::Pin<&'r T>,
    ['r] ::core::pin::Pin<&'r mut T>,
    #[apply(cfg_alloc)]
    [] rust::Box<T>,
    #[apply(cfg_alloc)]
    [] ::core::pin::Pin<rust::Box<T>>,
    #[apply(cfg_alloc)]
    [] ::std::rc::Rc<T>,
    #[apply(cfg_std)]
    [] ::std::sync::Arc<T>,
) {
    (
        $(
            $(#[$cfg:meta])?
            [$($($generics:tt)+)?] $T:ty
        ),* $(,)?
    ) => (
        $(
            $(#[$cfg])?
            impl<$($($generics)+ ,)? T, DynTrait>
                From<$T>
            for
                VirtualPtr<DynTrait>
            where
                DynTrait : ?Sized + VirtualPtrFrom<$T>,
            {
                fn from (it: $T)
                  -> VirtualPtr<DynTrait>
                {
                    DynTrait::into_virtual_ptr(it)
                }
            }
        )*
    )
}}

pub trait DynClone: ReprCTrait {
    fn dyn_clone(_: &VirtualPtr<Self>) -> VirtualPtr<Self>;
}

impl<DynTrait: ?Sized + DynClone> Clone for VirtualPtr<DynTrait> {
    fn clone(self: &'_ VirtualPtr<DynTrait>) -> VirtualPtr<DynTrait> {
        DynTrait::dyn_clone(self)
    }
}

use hack::VirtualPtr_;
mod hack {
    use ::safer_ffi::layout;

    #[super::derive_ReprC]
    #[repr(C)]
    #[allow(missing_debug_implementations)]
    pub struct VirtualPtr_<Ptr, VTable> {
        pub(super) ptr: Ptr,
        pub(super) vtable: VTable,
    }

    unsafe impl<Ptr, VTable> layout::__HasNiche__ for VirtualPtr_<Ptr, VTable>
    where
        Ptr: layout::ConcreteReprC + layout::__HasNiche__,
        VTable: layout::ConcreteReprC,
    {
        fn is_niche(it: &'_ <Self as super::ReprC>::CLayout) -> bool {
            Ptr::is_niche(&it.ptr)
        }
    }
}

#[cfg_attr(feature = "stabby", stabby::stabby)]
#[derive_ReprC]
#[repr(transparent)]
pub struct VirtualPtr<DynTrait: ?Sized + ReprCTrait>(
    VirtualPtr_</* ptr: */ ptr::NonNullOwned<ty::Erased>, /* vtable: */ DynTrait::VTable>,
);

impl<DynTrait: ?Sized + ReprCTrait> Drop for VirtualPtr<DynTrait> {
    fn drop(self: &'_ mut VirtualPtr<DynTrait>) {
        unsafe { DynTrait::drop_ptr(self.0.ptr.copy(), self.__vtable()) }
    }
}

impl<DynTrait: ?Sized + ReprCTrait> VirtualPtr<DynTrait> {
    pub unsafe fn from_raw_parts(
        ptr: ptr::NonNullOwned<ty::Erased>,
        vtable: DynTrait::VTable,
    ) -> VirtualPtr<DynTrait> {
        Self(VirtualPtr_ { ptr, vtable })
    }

    pub fn __ptr(self: &'_ VirtualPtr<DynTrait>) -> ptr::NonNull<ty::Erased> {
        self.0.ptr.0
    }

    pub fn __vtable<'vtable>(self: &'_ VirtualPtr<DynTrait>) -> &'_ DynTrait::VTable {
        &self.0.vtable
    }
}

unsafe impl<DynTrait: ?Sized + ReprCTrait> Send for VirtualPtr<DynTrait> where DynTrait: Send {}

unsafe impl<DynTrait: ?Sized + ReprCTrait> Sync for VirtualPtr<DynTrait> where DynTrait: Sync {}

impl<DynTrait: ?Sized + ReprCTrait> ::core::fmt::Debug for VirtualPtr<DynTrait> {
    fn fmt(
        self: &'_ VirtualPtr<DynTrait>,
        fmt: &'_ mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result {
        fmt.debug_struct(::core::any::type_name::<Self>())
            .field("ptr", &format_args!("{:p}", self.__ptr()))
            .field("vtable", &format_args!("{:p}", self.__vtable()))
            .finish()
    }
}
