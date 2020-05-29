#![cfg_attr(feature = "nightly",
    feature(doc_cfg, external_doc, trivial_bounds)
)]
#![cfg_attr(not(feature = "std"),
    no_std,
)]

#![allow(nonstandard_style, trivial_bounds)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
)]
#![deny(
    bare_trait_objects,
    elided_lifetimes_in_paths,
    unconditional_recursion,
    unused_must_use,
)]

//! See the [user guide](https://getditto.github.io/safer_ffi).

#[cfg(feature = "proc_macros")]
#[macro_use]
extern crate require_unsafe_in_body;

#[doc(hidden)] pub
extern crate paste;

#[macro_use]
#[path = "utils/_mod.rs"]
#[doc(hidden)] /** Not part of the public API **/ pub
mod __utils__;
use __utils__ as utils;

extern crate proc_macro;
pub use ::proc_macro::{ffi_export, cfg_headers};
cfg_proc_macros! {
    #[::proc_macro_hack::proc_macro_hack]
    /// Creates a compile-time checked [`char_p::Ref`]`<'static>` out of a
    /// string literal.
    ///
    /// [`char_p::Ref`]: `crate::prelude::char_p::Ref`
    pub use ::proc_macro::c_str as c;

    #[doc(inline)]
    pub use ::proc_macro::derive_ReprC;
}

#[macro_use]
#[path = "layout/_mod.rs"]
pub mod layout;

__cfg_headers__! {
    #[doc(hidden)] pub
    use ::inventory;

    #[cfg_attr(feature = "nightly",
        doc(cfg(feature = "headers")),
    )]
    #[path = "headers/_mod.rs"]
    pub
    mod headers;

    #[allow(missing_copy_implementations, missing_debug_implementations)]
    #[doc(hidden)] pub
    struct FfiExport(
        pub
        fn (&'_ mut dyn headers::Definer)
          -> ::std::io::Result<()>
        ,
    );

    ::inventory::collect!(FfiExport);
}

cfg_alloc! {
    extern crate alloc;
}

cfg_alloc! {
    pub
    mod boxed;
}

use self::c_char_module::c_char;
#[path = "c_char.rs"]
mod c_char_module;

pub
mod char_p;

pub
mod closure;

const _: () = {
    #[path = "ffi_export.rs"]
    mod ffi_export;
};

pub
mod ptr;

pub
mod slice;

#[path = "string/_mod.rs"]
pub
mod string;

#[doc(no_inline)]
pub
use tuple::*;

pub
mod tuple;

cfg_alloc! {
    #[doc(inline)]
    pub use string::String;

    #[doc(inline)]
    pub use vec::Vec;
    pub mod vec;
}

macro_rules! reexport_primitive_types {(
    $($ty:ident)*
) => (
    $(
        #[doc(hidden)]
        pub use $ty;
    )*
)} reexport_primitive_types! {
    u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    f32 f64
    char
    bool
    str
}
#[doc(hidden)] pub use ::core;
cfg_std! {
    #[doc(hidden)] pub use ::std;
}

#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
#[doc(hidden)] pub
struct NotZeroSized;

pub
mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        closure::*,
        ffi_export,
        layout::ReprC,
    };
    pub
    mod char_p {
        #[doc(no_inline)]
        pub use crate::char_p::{
            char_p_raw as Raw,
            char_p_ref as Ref,
            new,
        };
        cfg_alloc! {
            #[doc(no_inline)]
            pub use crate::char_p::char_p_boxed as Box;
        }
    }
    pub
    mod c_slice {
        #[doc(no_inline)]
        pub use crate::slice::{
            slice_mut as Mut,
            slice_raw as Raw,
            slice_ref as Ref,
        };
        cfg_alloc! {
            #[doc(no_inline)]
            pub use crate::slice::slice_boxed as Box;
        }
    }
    pub
    mod repr_c {
        cfg_alloc! {
            #[doc(no_inline)]
            pub use crate::{
                boxed::Box,
                string::String,
                vec::Vec,
            };
        }
    }
    pub
    mod str {
        #[doc(no_inline)]
        pub use crate::string::{
            // str_raw as Raw,
            str_ref as Ref,
        };
        cfg_alloc! {
            #[doc(no_inline)]
            pub use crate::string::str_boxed as Box;
        }
    }
    cfg_proc_macros! {
        #[doc(no_inline)]
        pub use crate::layout::derive_ReprC;
        #[doc(no_inline)]
        pub use c;
    }
    #[doc(no_inline)]
    pub use ::core::{
        convert::{
            TryFrom as _,
            TryInto as _,
        },
        ops::Not as _,
    };

    #[cfg(feature = "out-refs")]
    #[cfg_attr(all(docs, feature = "nightly"),
        doc(cfg(feature = "out-refs"))
    )]
    pub use ::uninit::prelude::{
        // Out reference itself
        Out,
        // Helper trait to go from `&mut T` and `&mut MaybeUninit<T>` to `Out<T>`
        AsOut,
        // Helper trait to have `AsOut` when `T : !Copy`
        ManuallyDropMut,
    };
}

#[macro_export]
macro_rules! NULL {() => (
    $crate::core::ptr::null_mut()
)}
