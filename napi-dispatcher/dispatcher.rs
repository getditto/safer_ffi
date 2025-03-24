#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! emit {( $($_:tt)* ) => ( $($_)* )}

#[cfg(target_arch = "wasm32")] emit! {
    extern crate napi_wasm;
    #[doc(no_inline)]
    pub use ::napi_wasm::*;
}

#[cfg(not(target_arch = "wasm32"))] emit! {
    #[allow(unused_extern_crates)]
    extern crate napi;
    pub use ::{
        napi::*,
    };

    pub extern crate napi_derive as derive;

    pub type Result<T, E = Error> = ::core::result::Result<T, E>;
}
