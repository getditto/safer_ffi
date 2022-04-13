use_prelude!();

hidden_export! {
    trait __AssocConst<Ty> {
        const CONST: Ty;
    }
}
hidden_export! {
    #[cfg(feature = "alloc")]
    trait __assert_dyn_safe {
        fn m(self: rust::Box<Self>);
    }
}

pub use self::ty::{
    Erased as ErasedTy,
};

#[super::derive_ReprC]
#[repr(transparent)]
#[allow(missing_debug_implementations)]
pub
struct ErasedRef<'a>(
    ptr::NonNull<ErasedTy>,
    ::core::marker::PhantomData<&'a ()>,
);

// #[macro_export]
// macro_rules! const_ {(
//     $(
//         for $generics:tt $(where { $($wc:tt)* })? ,
//     )?
//         $VALUE:block : $T:ty
// ) => ({
//     struct __Generics $generics (
//         *mut Self,
//     )
//     // where
//     //     $($($wc)*)?
//     ;

//     impl $generics
//         $crate::dyn_traits::__AssocConst<$T>
//     for
//         __Generics $generics
//     where
//         $($($wc)*)?
//     {
//         const CONST: $T = $VALUE;
//     }

//     <__Generics $generics as $crate::dyn_traits::__AssocConst<$T>>::CONST
// })}

pub
trait ReprCTrait {
    type VTable : ConcreteReprC;

    unsafe
    fn drop_ptr (
        ptr: ptr::NonNullOwned<ty::Erased>,
        vtable: &'_ Self::VTable,
    )
    ;

    // fn type_name (
    //     vtable: &'_ Self::VTable,
    // ) -> &'static str
    // ;
}

mod ty {
    #![allow(warnings)]

    #[super::derive_ReprC]
    #[repr(transparent)]
    // #[ReprC::opaque]
    pub
    struct Erased(crate::tuple::CVoid); //  { _private: () }
}

pub
trait VirtualPtrFromBox<T> : ReprCTrait { // DynTrait : ?Sized + ReprCTrait > : Sized {
    fn boxed_into_virtual_ptr (
        this: rust::Box<T>,
    ) -> VirtualPtr<Self>
    ;
}

impl<
    T,
    DynTrait : ?Sized + VirtualPtrFromBox<T>, // + ReprCTrait,
    // T : BoxedIntoVirtualPtr<DynTrait>,
>
    From<rust::Box<T>>
for
    VirtualPtr<DynTrait>
{
    fn from (boxed: rust::Box<T>)
      -> VirtualPtr<DynTrait>
    {
        DynTrait::boxed_into_virtual_ptr(boxed)
    }
}

use hack::VirtualPtr_;
mod hack {
    #[super::derive_ReprC]
    #[repr(C)]
    #[allow(missing_debug_implementations)]
    pub
    struct VirtualPtr_<Ptr, VTable> {
        pub(in super) ptr: Ptr,
        pub(in super) vtable: VTable,
    }
}

#[derive_ReprC]
#[repr(transparent)]
pub
struct VirtualPtr<DynTrait : ?Sized + ReprCTrait>(
    VirtualPtr_<
        /* ptr: */ ptr::NonNullOwned<ty::Erased>,
        /* vtable: */ ptr::NonNullRef<DynTrait::VTable>,
    >
);

impl<DynTrait : ?Sized + ReprCTrait>
    Drop
for
    VirtualPtr<DynTrait>
{
    fn drop (self: &'_ mut VirtualPtr<DynTrait>)
    {
        unsafe {
            DynTrait::drop_ptr(self.0.ptr.copy(), self.__vtable())
        }
    }
}

impl<DynTrait : ?Sized + ReprCTrait> VirtualPtr<DynTrait> {
    pub
    unsafe
    fn from_raw_parts (
        ptr: ptr::NonNullOwned<ty::Erased>,
        vtable: ptr::NonNullRef<DynTrait::VTable>,
    ) -> VirtualPtr<DynTrait>
    {
        Self(VirtualPtr_ { ptr, vtable })
    }

    pub
    fn __ptr (self: &'_ VirtualPtr<DynTrait>)
      -> ptr::NonNull<ty::Erased>
    {
        self.0.ptr.0
    }

    pub
    fn __vtable<'vtable> (self: &'_ VirtualPtr<DynTrait>)
      -> &'vtable DynTrait::VTable
    {
      unsafe { &*self.0.vtable.as_ptr() }
    }
}

unsafe
impl<DynTrait : ?Sized + ReprCTrait>
    Send
for
    VirtualPtr<DynTrait>
where
    DynTrait : Send,
{}

unsafe
impl<DynTrait : ?Sized + ReprCTrait>
    Sync
for
    VirtualPtr<DynTrait>
where
    DynTrait : Sync,
{}

impl<DynTrait : ?Sized + ReprCTrait>
    ::core::fmt::Debug
for
    VirtualPtr<DynTrait>
{
    fn fmt (
        self: &'_ VirtualPtr<DynTrait>,
        fmt: &'_ mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result
    {
        fmt .debug_struct(::core::any::type_name::<Self>())
            .field("ptr", &format_args!("{:p}", self.__ptr()))
            .field("vtable", &format_args!(
                concat!(
                    "{:p}",
                    // " ({})",
                ),
                self.__vtable(),
                // DynTrait::type_name(self.__vtable()),
            ))
            .finish()
    }
}
