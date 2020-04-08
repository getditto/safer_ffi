#![allow(nonstandard_style)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    elided_lifetimes_in_paths,
)]
#![cfg_attr(feature = "nightly",
    feature(doc_cfg, doc_spotlight, external_doc)
)]

#![feature(fundamental)]

#![cfg_attr(not(feature = "std"),
    no_std,
)]

#[macro_use]
extern crate _mod;

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
    #[cfg_attr(docs, doc(cfg(feature = "alloc")))]
    pub mod boxed;
}

#[doc(inline)]
pub use slice::{RefSlice, MutSlice};
pub mod slice;

#[doc(inline)]
pub use self::str::RefStr;
pub mod str;

cfg_alloc! {
    _mod!(pub closure);

    #[doc(inline)]
    pub use string::String;
    pub mod string;

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
#[doc(hidden)] #[cfg(feature = "std")] pub use ::std;
