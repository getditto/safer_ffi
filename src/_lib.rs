#![allow(nonstandard_style)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    elided_lifetimes_in_paths,
)]
#![cfg_attr(feature = "nightly",
    feature(doc_cfg, external_doc)
)]

#![cfg_attr(not(feature = "std"),
    no_std,
)]

#[macro_use]
extern crate _mod;

extern crate proc_macro;
pub use ::proc_macro::ffi_export;

#[macro_use]
#[path = "utils/_mod.rs"]
pub(in crate)
mod utils;

#[macro_use]
#[path = "layout/_mod.rs"]
pub mod layout;

pub
mod tuple;

cfg_alloc! {
    extern crate alloc;
}

cfg_alloc! {
    #[doc(inline)]
    pub use boxed::{Box, BoxedSlice, BoxedStr};
    pub mod boxed;
}

use self::c_char_module::c_char;
#[path = "c_char.rs"]
mod c_char_module;

pub mod closure;

pub mod c_str;

mod ptr;

#[doc(inline)]
pub use slice::{RefSlice, MutSlice};
pub mod slice;


#[doc(inline)]
pub use string::RefStr;

_mod!(pub string);

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
    ::paste::item! {
        mod hack {
            $(
                pub(in super)
                type [< $ty _hack >] = $ty;
            )*
        }
    }
    $(
        ::paste::item! {
            #[doc(hidden)]
            pub type $ty = hack::[< $ty _hack >];
        }
    )*
)} reexport_primitive_types! {
    u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
    char
    bool
    // str
}
#[doc(hidden)] pub use ::core;
cfg_std! {
    #[doc(hidden)] pub use ::std;
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct NotZeroSized;
