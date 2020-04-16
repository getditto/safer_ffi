//! Tuple types with a guaranteed `#[repr(C)]` layout.
//!
//! Simplified for lighter documentation, but the actual `struct` definitions
//! and impls range from `Tuple1` up to `Tuple9`.

use_prelude!();

mod void {
    #[derive(Clone, Copy)]
    pub
    struct CVoid {
        _0: (),
    }
    // pub const CVoid: CVoid = CVoid { _0: () };
}
pub(in crate) use void::CVoid;

unsafe
impl CType
    for CVoid
{ cfg_headers! {
    fn with_short_name<R> (
        ret: impl FnOnce(&'_ dyn fmt::Display) -> R,
    ) -> R
    {
        ret(&"void")
    }

    fn c_fmt (
        fmt: &'_ mut fmt::Formatter<'_>,
        var_name: &'_ str,
    ) -> fmt::Result
    {
        fmt.write_str("void")
    }
}}
from_CType_impl_ReprC! { CVoid }

unsafe
impl ReprC
    for ::core::ffi::c_void
{
    type CLayout = CVoid;

    fn is_valid (it: &'_ CVoid)
      -> bool
    {
        panic!("Trying to construct a `c_void` is a logic error");
    }
}

#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    /// Simplified for lighter documentation, but the actual impls
    /// range from `Tuple1` up to `Tuple9`.
    pub
    struct Tuple1[T0]
    where {
        T0 : ReprC,
    }
    {
        pub _0: T0,
    }
}

ReprC! {
    #[repr(C)]
    /// Simplified for lighter documentation, but the actual impls
    /// range from `Tuple1` up to `Tuple9`.
    pub
    struct Tuple2[T0, T1]
    where {
        T0 : ReprC,
        T1 : ReprC,
    }
    {
        pub _0: T0,
        pub _1: T1,
    }
}

#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    pub
    struct Tuple3[T0, T1, T2]
    where {
        T0 : ReprC,
        T1 : ReprC,
        T2 : ReprC,
    }
    {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
    }
}

#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    pub
    struct Tuple4[T0, T1, T2, T3]
    where {
        T0 : ReprC,
        T1 : ReprC,
        T2 : ReprC,
        T3 : ReprC,
    }
    {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
    }
}
#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    pub
    struct Tuple5[T0, T1, T2, T3, T4]
    where {
        T0 : ReprC,
        T1 : ReprC,
        T2 : ReprC,
        T3 : ReprC,
        T4 : ReprC,
    }
    {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
        pub _4: T4,
    }
}
#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    pub
    struct Tuple6[T0, T1, T2, T3, T4, T5]
    where {
        T0 : ReprC,
        T1 : ReprC,
        T2 : ReprC,
        T3 : ReprC,
        T4 : ReprC,
        T5 : ReprC,
    }
    {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
        pub _4: T4,
        pub _5: T5,
    }
}
