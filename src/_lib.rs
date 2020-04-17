#![cfg_attr(feature = "nightly",
    feature(doc_cfg, external_doc, trivial_bounds)
)]
#![cfg_attr(not(feature = "std"),
    no_std,
)]

#![allow(nonstandard_style)]
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

#[macro_use]
extern crate _mod;

#[macro_use]
extern crate require_unsafe_in_body;

extern crate proc_macro;
pub use ::proc_macro::ffi_export;

#[macro_use]
#[path = "utils/_mod.rs"]
pub(in crate)
mod utils;

#[macro_use]
#[path = "layout/_mod.rs"]
pub mod layout;

cfg_headers! {
    #[doc(hidden)] pub
    use ::inventory;

    #[doc(hidden)] pub
    struct TypeDef(
        pub
        fn (&'_ mut dyn layout::Definer)
          -> ::std::io::Result<()>
        ,
    );

    ::inventory::collect!(TypeDef);
}

cfg_alloc! {
    extern crate alloc;
}

cfg_alloc! {
    #[doc(inline)]
    pub use boxed::{Box, slice_boxed, str_boxed};
    pub mod boxed;
}

use self::c_char_module::c_char;
#[path = "c_char.rs"]
mod c_char_module;

pub mod char_p;

pub mod closure;

mod ffi_export;

pub
mod ptr;

#[doc(inline)]
pub use slice::{slice_ref, slice_mut};
pub mod slice;


#[doc(inline)]
pub use string::str_ref;

_mod!(pub string);

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

#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
#[doc(hidden)] pub
struct NotZeroSized;
