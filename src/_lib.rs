#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![cfg_attr(feature = "nightly",
    feature(doc_cfg, trivial_bounds)
)]
#![cfg_attr(not(feature = "std"),
    no_std,
)]

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

#![feature(rustc_attrs)] #![allow(warnings)]

#[macro_use]
extern crate fstrings;

#[macro_use]
extern crate macro_rules_attribute;

#[macro_use]
extern crate with_builtin_macros;

#[macro_use]
#[path = "utils/_mod.rs"]
#[doc(hidden)] /** Not part of the public API **/ pub
mod __utils__;
use __utils__ as utils;

#[apply(hidden_export)]
use ::paste;

/// Export a function to be callable by C.
///
/// # Example
///
/// ```rust
/// use ::safer_ffi::prelude::ffi_export;
///
/// #[ffi_export]
/// /// Add two integers together.
/// fn add (x: i32, y: i32) -> i32
/// {
///     x + y
/// }
/// ```
///
///   - ensures that [the generated headers](/safer_ffi/headers/) will include the
///     following definition:
///
///     ```C
///     #include <stdint.h>
///
///     /* \brief
///      * Add two integers together.
///      */
///     int32_t add (int32_t x, int32_t y);
///     ```
///
///   - exports an `add` symbol pointing to the C-ABI compatible
///     `int32_t (*)(int32_t x, int32_t y)` function.
///
///     (The crate type needs to be `cdylib` or `staticlib` for this to work,
///     and, of course, the C compiler invocation needs to include
///     `-L path/to/the/compiled/library -l name_of_your_crate`)
///
///       - when in doubt, use `staticlib`.
///
/// # `ReprC`
///
/// You can use any Rust types in the singature of an `#[ffi_export]`-
/// function, provided each of the types involved in the signature is [`ReprC`].
///
/// Otherwise the layout of the involved types in the C world is **undefined**,
/// which `#[ffi_export]` will detect, leading to a compilation error.
///
/// To have custom structs implement [`ReprC`], it suffices to annotate the
/// `struct` definitions with the <code>#\[[derive_ReprC]\]</code>
/// (on top of the obviously required `#[repr(C)]`).
pub use ::safer_ffi_proc_macros::ffi_export;

/// Identity macro when `feature = "headers"` is enabled, otherwise
/// this macro outputs nothing.
pub use ::safer_ffi_proc_macros::cfg_headers;

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

/// Safely implement [`ReprC`]
/// for a `#[repr(C)]` struct **when all its fields are [`ReprC`]**.
///
/// # Examples
///
/// ### Simple `struct`
///
/// ```rust
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(C)]
/// struct Instant {
///     seconds: u64,
///     nanos: u32,
/// }
/// ```
///
///   - corresponding to the following C definition:
///
///     ```C
///     typedef struct {
///         uint64_t seconds;
///         uint32_t nanos;
///     } Instant_t;
///     ```
///
/// ### Field-less `enum`
///
/// ```rust
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(u8)]
/// enum Status {
///     Ok = 0,
///     Busy,
///     NotInTheMood,
///     OnStrike,
///     OhNo,
/// }
/// ```
///
///   - corresponding to the following C definition:
///
///     ```C
///     typedef uint8_t Status_t; enum {
///         STATUS_OK = 0,
///         STATUS_BUSY,
///         STATUS_NOT_IN_THE_MOOD,
///         STATUS_ON_STRIKE,
///         STATUS_OH_NO,
///     }
///     ```
///
/// ### Generic `struct`
///
/// In that case, it is required that the struct's generic types carry a
/// `: ReprC` bound each:
///
/// ```rust
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(C)]
/// struct Point<Coordinate : ReprC> {
///     x: Coordinate,
///     y: Coordinate,
/// }
/// #
/// # fn main() {}
/// ```
///
/// Each monomorphization leads to its own C definition:
///
///   - **`Point<i32>`**
///
///     ```C
///     typedef struct {
///         int32_t x;
///         int32_t y;
///     } Point_int32_t;
///     ```
///
///   - **`Point<f64>`**
///
///     ```C
///     typedef struct {
///         double x;
///         double y;
///     } Point_double_t;
///     ```
pub use ::safer_ffi_proc_macros::derive_ReprC;

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

#[cfg(feature = "dyn-traits")]
#[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "dyn-traits")))]
pub
mod dyn_traits;

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

#[apply(hidden_export)]
use layout::impls::c_int;

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

    #[doc(no_inline)]
    pub use {
        crate::layout::derive_ReprC,
        ::safer_ffi_proc_macros::derive_ReprC2,
        crate::c,
        ::core::{
            convert::{
                TryFrom as _,
                TryInto as _,
            },
            ops::Not as _,
        },
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

    #[cfg(feature = "dyn-traits")]
    #[cfg_attr(all(docs, feature = "nightly"),
        doc(cfg(feature = "dyn-traits"))
    )]
    pub use crate::dyn_traits::VirtualPtr;
}

#[macro_export]
macro_rules! NULL {() => (
    $crate::ඞ::ptr::null_mut()
)}

#[cfg(feature = "log")]
#[apply(hidden_export)]
use ::log;

#[cfg(feature = "node-js")]
// #[apply(hidden_export)]
#[path = "node_js/_mod.rs"]
pub mod node_js;

#[apply(hidden_export)]
#[allow(missing_copy_implementations, missing_debug_implementations)]
struct __PanicOnDrop__; impl Drop for __PanicOnDrop__ {
    fn drop (self: &'_ mut Self)
    {
        panic!()
    }
}

#[cfg(feature = "log")]
#[apply(hidden_export)]
macro_rules! __abort_with_msg__ { ($($tt:tt)*) => ({
    $crate::log::error!($($tt)*);
    let _panic_on_drop = $crate::__PanicOnDrop__;
    $crate::ඞ::panic!($($tt)*);
})}

#[cfg(all(
    not(feature = "log"),
    feature = "std",
))]
#[apply(hidden_export)]
macro_rules! __abort_with_msg__ { ($($tt:tt)*) => ({
    $crate::ඞ::eprintln!($($tt)*);
    $crate::ඞ::process::abort();
})}

#[cfg(all(
    not(feature = "log"),
    not(feature = "std"),
))]
#[apply(hidden_export)]
macro_rules! __abort_with_msg__ { ($($tt:tt)*) => ({
    let _panic_on_drop = $crate::__PanicOnDrop__;
    $crate::ඞ::panic!($($tt)*);
})}

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

#[apply(hidden_export)]
use __ as ඞ;

#[apply(hidden_export)]
mod __ {
    pub use {
        ::core::{
            self,
            marker::PhantomData,
            primitive::{
                u8, u16, u32, usize, u64, u128,
                i8, i16, i32, isize, i64, i128,
                bool,
                char,
                str,
            },
        },
        ::scopeguard::{
            self,
        },
        ::std::{
            self,
            *,
            prelude::rust_2021::*,
        },
        crate::{
            headers::{
                Definer,
                languages::{
                    self,
                    EnumVariant,
                    FunctionArg,
                    HeaderLanguage,
                    StructField,
                },
            },
            layout::{
                CLayoutOf,
                ConcreteReprC,
                CType,
                OpaqueKind,
                ReprC,
                __HasNiche__,
            },
        }
    };

    /// Hack needed to `feature(trivial_bounds)` in stable Rust:
    ///
    /// Instead of `where Ty : Bounds…`, it suffices to write:
    /// `where for<'trivial> Identity<'trivial, Ty> : Bounds…`.
    pub
    type Identity<'hrtb, T> =
        <T as IdentityIgnoring<'hrtb>>::ItSelf
    ;
    // where
    pub
    trait IdentityIgnoring<'__> {
        type ItSelf : ?Sized;
    }
    impl<T : ?Sized> IdentityIgnoring<'_> for T {
        type ItSelf = Self;
    }

    // TODO: correctly handle more type (currently only supports a subset of
    // transitive type paths).
    pub
    fn append_unqualified_name (
        out: &'_ mut String,
        full_type_name: &'_ str,
    )
    {
        let (before_generic, generics) = {
            let mut i = full_type_name.trim().splitn(2, "<");
            (i.next().unwrap(), i.next())
        };
        let unqualified = before_generic.rsplitn(2, ":").next().unwrap();
        out.push('_');
        out.push_str(unqualified.trim());
        if let Some(generics) = generics {
            // "pop" the `>`.
            let generics = &generics[.. generics.len() - 1];
            generics.split(',').for_each(|generic| {
                append_unqualified_name(out, generic);
            });
        }
    }
}
