#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![cfg_attr(feature = "nightly",
    feature(doc_cfg)
)]
#![no_std]

#![allow(nonstandard_style, trivial_bounds, unused_parens)]
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

#![doc = include_str!("../README.md")]
#![cfg(not(rustfmt))]

#[macro_use]
#[path = "utils/_mod.rs"]
#[doc(hidden)] /** Not part of the public API **/ pub
mod __utils__;
use __utils__ as utils;

hidden_export! {
    use ::paste;
}

pub use ::safer_ffi_proc_macros::{ffi_export, cfg_headers};
cfg_proc_macros! {
    /// Creates a compile-time checked [`char_p::Ref`]`<'static>` out of a
    /// string literal.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ::safer_ffi::prelude::*;
    ///
    /// #[ffi_export]
    /// fn concat (s1: char_p::Ref<'_>, s2: char_p::Ref<'_>)
    ///   -> char_p::Box
    /// {
    ///     format!("{}{}", s1.to_str(), s2.to_str())
    ///         .try_into()
    ///         .unwrap() // No inner nulls in our format string
    /// }
    ///
    /// fn main ()
    /// {
    ///     assert_eq!(
    ///         concat(c!("Hello, "), c!("World!")).as_ref(),
    ///         c!("Hello, World!"),
    ///     );
    /// }
    /// ```
    ///
    /// If the string literal contains an inner null byte, then the macro
    /// will detect it at compile time and thus cause a compile-time error
    /// (allowing to skip the then unnecessary runtime check!):
    ///
    /// ```rust,compile_fail
    /// let _ = ::safer_ffi::c!("Hell\0, World!"); // <- Compile error
    /// ```
    ///
    /// [`char_p::Ref`]: `crate::prelude::char_p::Ref`
    pub use ::safer_ffi_proc_macros::c_str as c;

    #[doc(inline)]
    pub use ::safer_ffi_proc_macros::derive_ReprC;
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
    struct FfiExport {
        pub name: &'static str,
        pub gen_def:
            fn (&'_ mut dyn headers::Definer, headers::Language)
              -> ::std::io::Result<()>
        ,
    }

    ::inventory::collect!(FfiExport);
}

cfg_alloc! {
    extern crate alloc;
}
cfg_std! {
    #[doc(hidden)] /** Not part of the public API */ pub
    extern crate std;
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

#[path = "ffi_export.rs"]
mod __ffi_export;

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

hidden_export! {
    use ::core;
}

hidden_export! {
    use ::scopeguard;
}

#[doc(hidden)] /** Not part of the public API **/ pub
use layout::impls::c_int;

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
        };
        cfg_alloc! {
            #[doc(no_inline)]
            pub use crate::char_p::{
                char_p_boxed as Box,
                new,
            };
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
        pub use crate::c;
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

#[cfg(feature = "log")]
hidden_export! {
    use ::log;
}

#[cfg(feature = "node-js")]
// hidden_export! {
    #[path = "node_js/_mod.rs"]
    pub mod node_js;
// }

hidden_export! {
    #[allow(missing_copy_implementations, missing_debug_implementations)]
    struct __PanicOnDrop__; impl Drop for __PanicOnDrop__ {
        fn drop (self: &'_ mut Self)
        {
            panic!()
        }
    }
}

#[cfg(feature = "log")]
hidden_export! {
    macro_rules! __abort_with_msg__ { ($($tt:tt)*) => ({
        $crate::log::error!($($tt)*);
        let _panic_on_drop = $crate::__PanicOnDrop__;
        $crate::core::panic!($($tt)*);
    })}
}
#[cfg(all(
    not(feature = "log"),
    feature = "std",
))]
hidden_export! {
    macro_rules! __abort_with_msg__ { ($($tt:tt)*) => ({
        $crate::std::eprintln!($($tt)*);
        $crate::std::process::abort();
    })}
}
#[cfg(all(
    not(feature = "log"),
    not(feature = "std"),
))]
hidden_export! {
    macro_rules! __abort_with_msg__ { ($($tt:tt)*) => ({
        let _panic_on_drop = $crate::__PanicOnDrop__;
        $crate::core::panic!($($tt)*);
    })}
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
mod libc {
    pub type c_int = i32;
    pub type size_t = u32;
    pub type uintptr_t = u32;
}
#[cfg(not(target_arch = "wasm32"))]
use ::libc;

extern crate self as safer_ffi;
