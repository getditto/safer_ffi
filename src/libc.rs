#![cfg_attr(rustfmt, rustfmt::skip)]
//! On certain platforms, `::libc` has no definitions for pervasive types such as `size_t`.
//!
//! We polyfill them here, and reëxport them for downstream users to use at leisure
//! (_e.g._, so that they don't have to do that themselves too!).
//!
//! ```rust
//! # #[cfg(any())] macro_rules! ignore {
#![doc = stringified_module_code!()]
//! # }
#![allow(warnings, clippy::all)]

use_libc_or_else! {
    pub use ::libc::{
        /// Note: you should probably be using [`crate::c_char`] instead.
        c_char else u8,
        /// Note: you should probably be using [`crate::c_int`] instead.
        c_int else ::core::ffi::c_int,
        ///
        size_t else usize,
        ///
        uintptr_t else usize,
    };
}

macro_rules! use_libc_or_else_ {(
    pub use ::libc::{
        $(
            $(#$doc:tt)*
            $c_type:ident else $FallbackTy:ty
        ),* $(,)?
    };
) => (

    $(
        #[doc = concat!("A _`type` alias_ to [`::libc::", stringify!($c_type), "`].")]
        ///
        $(#$doc)*
        pub type $c_type = helper::$c_type;
    )*

    mod helper {
        mod real_libc {
            pub use ::libc::*;
            $(
                pub const $c_type: () = ();
            )*
        }

        pub use real_libc::{
            $(
                $c_type,
            )*
        };

        pub use fallback::*;
        mod fallback {
            $(
                pub type $c_type = $FallbackTy;
            )*
        }
    }
)} use use_libc_or_else_;

macro_rules! use_libc_or_else {(
    $($input:tt)*
) => (
    macro_rules! stringified_module_code {() => (
        stringify!($($input)*)
    )}

    use_libc_or_else_!($($input)*);
)} use use_libc_or_else;
