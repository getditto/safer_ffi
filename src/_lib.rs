#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![cfg_attr(feature = "nightly",
    feature(doc_cfg)
)]
#![cfg_attr(not(feature = "std"),
    no_std,
)]

#![allow(
    nonstandard_style,
    trivial_bounds,
    unused_parens,
)]
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

#[macro_use]
extern crate macro_rules_attribute;

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
    cfg_match! {
        feature = "inventory-0-3-1" => {
            #[doc(hidden)] pub
            use ::inventory_0_3_1 as inventory;
        },
        _ => {
            #[doc(hidden)] pub
            use ::inventory;
        },
    }

    #[cfg_attr(feature = "nightly",
        doc(cfg(feature = "headers")),
    )]
    #[path = "headers/_mod.rs"]
    pub
    mod headers;

    #[allow(missing_copy_implementations, missing_debug_implementations)]
    #[doc(hidden)] /** Not part of the public API */ pub
    struct FfiExport {
        pub
        name: &'static str,

        pub
        gen_def:
            fn(&mut dyn headers::Definer, headers::Language)
              -> std::io::Result<()>
        ,
    }

    self::inventory::collect!(FfiExport);
}

cfg_alloc! {
    extern crate alloc;
}

cfg_alloc! {
    pub
    mod boxed;
}

#[doc(inline)]
pub use self::c_char_module::c_char;
#[path = "c_char.rs"]
mod c_char_module;

pub
mod char_p;

pub
mod closure;

#[cfg(feature = "dyn-traits")]
#[cfg_attr(feature = "nightly",
    doc(cfg(feature = "dyn-traits")),
)]
#[path = "dyn_traits/_mod.rs"]
pub
mod dyn_traits;

#[cfg(feature = "futures")]
#[cfg_attr(all(docs, feature = "nightly"),
    doc(cfg(feature = "futures"))
)]
#[doc(no_inline)]
pub use dyn_traits::futures;

pub
mod libc;

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

#[doc(inline)]
pub use layout::impls::c_int;

pub
mod prelude {
    #[doc(no_inline)]
    pub use crate::{
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

            pub
            type Arc<T> = <T as crate::boxed::FitForCArc>::CArcWrapped;
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

#[cfg(feature = "js")]
// #[apply(hidden_export)]
#[path = "js/_mod.rs"]
pub mod js;

#[apply(hidden_export)]
#[allow(missing_copy_implementations, missing_debug_implementations)]
struct __PanicOnDrop__ {} impl Drop for __PanicOnDrop__ {
    fn drop (self: &'_ mut Self)
    {
        panic!()
    }
}

#[apply(hidden_export)]
macro_rules! __abort_with_msg__ { ($($tt:tt)*) => (
    match ($crate::__PanicOnDrop__ {}) { _ => {
        $crate::ඞ::__error__!($($tt)*);
        $crate::ඞ::panic!($($tt)*);
    }}
)}

extern crate self as safer_ffi;

#[apply(hidden_export)]
use __ as ඞ;

#[apply(hidden_export)]
mod __ {
    #[cfg(feature = "alloc")]
    pub extern crate alloc;

    pub use {
        ::core::{
            self,
            marker::PhantomData,
            pin::Pin,
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
        crate::{
            ptr,
            layout::{
                CLayoutOf,
                ConcreteReprC,
                CType,
                OpaqueKind,
                ReprC,
                __HasNiche__,
            },
            prelude::*,
        },
    };

    #[cfg(feature = "headers")]
    pub use {
        crate::{
            headers::{
                Definer,
                Language,
                languages::{
                    self,
                    EnumVariant,
                    FunctionArg,
                    HeaderLanguage,
                    StructField,
                },
            },
            inventory,
            FfiExport,
        },
    };

    cfg_match! {
        feature = "std" => {
            pub use ::std::{*,
                self,
                prelude::rust_2021::*,
            };
        },
        feature = "alloc" => {
            pub use {
                ::core::{*,
                    prelude::rust_2021::*,
                },
                ::alloc::{
                    boxed::Box,
                    string::String,
                    vec::Vec,
                },
            };
        },
        _ => {
            pub use ::core::{*,
                prelude::rust_2021::*,
            };
        }
    }

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

    cfg_match! {
        feature = "log" => {
            #[apply(hidden_export)]
            macro_rules! __error__ {( $($msg:tt)* ) => (
                $crate::log::error! { $($msg)* }
            )}
        },
        feature = "std" => {
            #[apply(hidden_export)]
            macro_rules! __error__ {( $($msg:tt)* ) => (
                $crate::ඞ::eprintln! { $($msg)* }
            )}
        },
        _ => {
            #[apply(hidden_export)]
            macro_rules! __error__ {( $($msg:tt)* ) => (
                /* nothing we can do */
            )}
        },
    }
    pub use __error__;

    #[allow(missing_debug_implementations)]
    pub
    struct UnwindGuard /* = */ (
        pub &'static str,
    );

    impl Drop for UnwindGuard {
        fn drop (self: &'_ mut Self)
        {
            let &mut Self(fname) = self;
            __abort_with_msg__!("\
                Error, attempted to panic across the FFI \
                boundary of `{fname}()`, \
                which is Undefined Behavior.\n\
                Aborting for soundness.\
            ");
        }
    }

    #[cfg(feature = "alloc")]
    pub
    fn append_unqualified_name (
        out: &'_ mut String,
        ty_name: &'_ str,
    )
    {
        #[inline(never)]
        fn mb_split_with<'r> (
            orig: &'r str,
            splitter: fn(&'r str) -> Option<(&'r str, &'r str)>,
        ) -> (&'r str, Option<&'r str>)
        {
            splitter(orig).map_or((orig, None), |(l, r)| (l, Some(r)))
        }

        let ty_name = ty_name.trim();
        if let Some(tuple_innards) = ty_name.strip_prefix('(') {
            // Tuple
            tuple_innards
                .strip_suffix(')').unwrap()
                .split(',')
                .for_each(|generic| {
                    append_unqualified_name(out, generic);
                })
            ;
        } else if let Some(bracketed_innards) = ty_name.strip_prefix('[') {
            // Array or Slice
            let (elem_ty, mb_len) = mb_split_with(
                bracketed_innards.strip_suffix(']').unwrap(),
                |s| s.rsplit_once(';'),
            );
            append_unqualified_name(out, elem_ty);
            if let Some(len) = mb_len {
                append_unqualified_name(out, len);
            }
        } else {
            // Canonical Type Path
            out.push('_');
            let (mut path, mb_generics) = mb_split_with(
                ty_name,
                |s| s.split_once('<'),
            );
            let is_valid_for_ident = |c: char| {
                c.is_alphanumeric() || matches!(c, '_')
            };
            if let Some(trait_path) = path.strip_prefix("dyn ") {
                out.push_str("dyn_");
                path = trait_path;
            }
            if path.chars().all(|c| is_valid_for_ident(c) || c == ':') {
                let unqualified = path.rsplitn(2, ':').next().unwrap().trim();
                out.push_str(unqualified);
            } else {
                // Weird type, fall back to replacing non_alphanumerics:
                path.chars().for_each(|c| {
                    out.push(if is_valid_for_ident(c) { c } else { '_' });
                });
            }
            if let Some(generics) = mb_generics {
                let generics = generics.strip_suffix('>').unwrap();
                generics.split(',').for_each(|generic| {
                    append_unqualified_name(out, generic);
                });
            }
        }
    }

    #[doc(hidden)] /** Not part of the public API! */
    #[macro_export]
    macro_rules! ඞassert_expr {( $e:expr $(,)? ) => ( $e )}
    #[doc(inline)]
    pub use ඞassert_expr as assert_expr;
}
