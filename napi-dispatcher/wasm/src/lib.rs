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
use gloo_utils::format::JsValueSerdeExt;

pub
mod derive {
    #[doc(inline)]
    pub use ::napi_dispatcher_wasm_proc_macros::*;
}

mod env;

mod utils;

mod values;

#[repr(C)]
pub struct CallContext<'env> {
    env: Box<Env>,
    _lifetime: ::core::marker::PhantomData<&'env ()>,
}

mod __mock_env_field_as_ref_hack {
    use super::*;

    #[repr(C)]
    pub struct CallContextDerefTarget<'env> {
        pub env: &'env mut Env,
        _lifetime: (),
    }

    impl<'env> ::core::ops::Deref for CallContext<'env> {
        type Target = CallContextDerefTarget<'env>;

        fn deref (self: &'_ CallContext<'env>)
          -> &'_ CallContextDerefTarget<'env>
        {
            unsafe {
                // Safety: same layout, and `&'_ &'env mut Env` necesarily
                // acts as a `&'_ Env`, usability-wise.
                ::core::mem::transmute(self)
            }
        }
    }
}

trait DropGlue {}
impl<T> DropGlue for T {}

pub
struct Env {
    cleanup: ::core::cell::RefCell<Vec<Box<dyn DropGlue>>>,
}

#[derive(Debug, ::serde::Deserialize, ::serde::Serialize)]
pub
struct Error {
    reason: String,
    status: Status,
}

utils::new_type_wrappers! {
    pub type JsBigint = ::wasm_bindgen::JsValue; // ;__;
    pub type JsBoolean = ::js_sys::Boolean;
    pub type JsBuffer = ::js_sys::Uint8Array;
    pub type JsFunction = ::js_sys::Function;
    pub type JsNumber = ::js_sys::Number;
    pub type JsPromise = ::js_sys::Promise;
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

#[derive(Debug, PartialEq, Eq, ::serde::Deserialize, ::serde::Serialize)]
#[non_exhaustive]
pub
enum Status {
    Ok,
    InvalidArg,
    GenericFailure,
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub
enum ValueType {
    Bigint,
    Boolean,
    Function,
    Null,
    Number,
    Object,
    String,
    Symbol,
    Undefined,
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
            env: Box::new(Env::__new()),
            _lifetime: Default::default(),
        }
    }
}

impl Env {
    #[doc(hidden)] /** Not part of the public API */ pub
    fn __new ()
      -> Self
    {
        Env { cleanup: vec![].into() }
    }

    #[doc(hidden)] /** Not part of the public API */ pub
    fn __push_drop_glue (
        self: &'_ Env,
        drop_glue: impl 'static + Sized,
    )
    {
        self.cleanup.borrow_mut().push(Box::new(drop_glue))
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
            .unwrap_or_else(|err| JsValue::from_str(&format!("{}: {:?}", err, e)))
    }
}

impl From<JsValue> for Error {
    fn from (e: JsValue)
      -> Error
    {
        JsValue::into_serde(&e)
            .unwrap_or_else(|err| Error::from_reason(format!("{}: {:?}", err, e)))
    }
}

#[doc(hidden)] /** Not part of the public API */ pub
mod __ {
    pub use ::js_sys;
    pub use ::wasm_bindgen::{self,
        prelude::wasm_bindgen,
    };
    pub use ::wasm_bindgen_futures;
}
