#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#[doc(inline)]
pub use ::{
    napi_derive::js_function,
    napi_dispatcher_nodejs_derive_proc_macros::*,
};
