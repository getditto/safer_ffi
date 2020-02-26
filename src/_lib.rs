#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    elided_lifetimes_in_paths,
)]

#![no_std]

#[macro_use]
#[path = "utils/_mod.rs"]
pub(in crate)
mod utils;

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
    pub mod closure;

    #[doc(inline)]
    pub use string::String;
    pub mod string;

    #[doc(inline)]
    pub use vec::Vec;
    pub mod vec;
}
