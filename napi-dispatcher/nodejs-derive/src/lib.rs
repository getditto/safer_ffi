#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]

#[doc(inline)]
pub use ::napi_derive::js_function;
#[doc(inline)]
pub use ::napi_dispatcher_nodejs_derive_proc_macros::*;
