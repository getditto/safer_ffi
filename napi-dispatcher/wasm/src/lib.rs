//! Polyfill of `::napi` using `wasm_bindgen`.

#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]

use ::{
    ref_cast::RefCast,
    wasm_bindgen::JsValue,
};
use crate::{
    utils::TurboFish,
};

pub
mod derive {
    #[doc(inline)]
    pub use ::napi_dispatcher_wasm_proc_macros::*;
}

mod env;

mod utils;

mod values;

pub struct CallContext<'env> {
    pub env: &'env mut Env,
    _private: (),
}

pub
struct Env {
    _private: (),
}

#[derive(::serde::Serialize)]
pub
struct Error {
    reason: String,
    status: Status,
}

utils::new_type_wrappers! {
    pub type JsBoolean = ::js_sys::Boolean;
    pub type JsBuffer = ::js_sys::Uint8Array;
    pub type JsFunction = ::js_sys::Function;
    pub type JsNumber = ::js_sys::Number;
    pub type JsObject = ::js_sys::Object;
    pub type JsString = ::js_sys::JsString;
    pub type JsNull = ::wasm_bindgen::JsValue;
    pub type JsUndefined = ::wasm_bindgen::JsValue;
    #[js_unknown]
    pub type JsUnknown = ::wasm_bindgen::JsValue;
}

// NapiValue
pub use ::wasm_bindgen::JsCast as NapiValue;

pub type Result<T, E = JsValue> = ::core::result::Result<T, E>;

#[derive(::serde::Serialize)]
#[non_exhaustive]
pub
enum Status {
    Ok,
    InvalidArg,
    GenericFailure,
}

#[non_exhaustive]
pub
enum ValueType {
    Function,
    Null,
    Object,
    String,
    Unknown,
}

// -- impls

impl CallContext<'_> {
    #[inline]
    #[doc(hidden)] /** Not part of the public API */ pub
    fn __new ()
      -> Self
    {
        Self {
            env: unsafe {
                ::core::mem::transmute::<
                    &'static mut [Env; 0],
                    &'static mut Env,
                >(&mut [])
            },
            _private: (),
        }
    }
}

impl Error {
    pub
    fn new (status: Status, reason: String)
      -> Self
    {
        Error { status, reason }
    }

    pub
    fn from_status (status: Status)
      -> Self
    {
        Error { status, reason: "".into() }
    }

    pub
    fn from_reason (reason: String)
      -> Self
    {
        Error { reason, status: Status::GenericFailure }
    }
}

impl From<Error> for JsValue {
    fn from (e: Error)
      -> JsValue
    {
        JsValue::from_serde(&e)
            .unwrap_or_else(|err| JsValue::from_str(&err.to_string()))
    }
}

#[doc(hidden)] /** Not part of the public API */ pub
mod __ {
    pub use ::wasm_bindgen::{self,
        prelude::wasm_bindgen,
    };
}
