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
    pub use boxed::Box;
    #[cfg_attr(docs, doc(cfg(feature = "alloc")))]
    pub mod boxed;
}
pub use slice::*;
pub mod slice;

pub use self::str::*;
pub mod str;

cfg_alloc! {
    pub use string::String;
    pub mod string;

    pub use vec::Vec;
    pub mod vec;
}
