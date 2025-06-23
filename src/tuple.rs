//! Tuple types with a guaranteed `#[repr(C)]` layout.
//!
//! Simplified for lighter documentation, but the actual `struct` definitions
//! and impls range from `Tuple1` up to `Tuple6`.

use_prelude!();

mod void {
    #[derive(Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct CVoid {
        _0: (),
    }
    // pub const CVoid: CVoid = CVoid { _0: () };
}
pub(crate) use void::CVoid;

unsafe impl CType for CVoid {
    type OPAQUE_KIND = crate::layout::OpaqueKind::Concrete;

    __cfg_headers__! {
        fn short_name () -> String {
            "void".into()
        }

        fn define_self__impl (
            _language: &'_ dyn HeaderLanguage,
            _definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            Ok(())
        }

        fn render(
            out: &'_ mut dyn io::Write,
            language: &'_ dyn HeaderLanguage,
        ) -> io::Result<()>
        {
            language.emit_void_output_type(out)?;
            Ok(())
        }
    }
}
from_CType_impl_ReprC! { CVoid }

unsafe impl ReprC for ::core::ffi::c_void {
    type CLayout = CVoid;

    fn is_valid(_: &'_ CVoid) -> bool {
        panic!("Trying to construct a `c_void` is a logic error");
    }
}

#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    #[derive(Debug)]
    /// Simplified for lighter documentation, but the actual impls
    /// range from `Tuple1` up to `Tuple6`.
    pub
    struct Tuple1[T0] {
        pub _0: T0,
    }
}

ReprC! {
    #[repr(C)]
    #[derive(Debug)]
    /// Simplified for lighter documentation, but the actual impls
    /// range from `Tuple1` up to `Tuple6`.
    pub
    struct Tuple2[T0, T1] {
        pub _0: T0,
        pub _1: T1,
    }
}

#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    #[derive(Debug)]
    pub
    struct Tuple3[T0, T1, T2] {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
    }
}

#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    #[derive(Debug)]
    pub
    struct Tuple4[T0, T1, T2, T3] {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
    }
}
#[cfg(not(docs))]
ReprC! {
    #[repr(C)]
    #[derive(Debug)]
    pub
    struct Tuple5[T0, T1, T2, T3, T4] {
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
    #[derive(Debug)]
    pub
    struct Tuple6[T0, T1, T2, T3, T4, T5] {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
        pub _4: T4,
        pub _5: T5,
    }
}
