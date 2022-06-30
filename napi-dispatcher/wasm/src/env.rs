#![cfg_attr(rustfmt, rustfmt::skip)]

use super::*;

impl Env {
    pub
    fn create_array (self: &'_ Env)
      -> Result< JsObject >
    {
        Ok(JsObject {
            __wasm: ::js_sys::Array::new().unchecked_into(),
        })
    }
    pub
    fn create_buffer_copy (self: &'_ Env, xs: &'_ [u8])
      -> Result< JsBuffer >
    {
        Ok(JsBuffer {
            __wasm: xs.into(),
        })
    }

    pub
    fn create_int32 (self: &'_ Env, i: i32)
      -> Result< JsNumber >
    {
        i.try_into_()
    }

    pub
    fn create_int64 (self: &'_ Env, i: i64)
      -> Result< JsNumber >
    {
        i.try_into_()
    }

    pub
    fn create_double (self: &'_ Env, f: f64)
      -> Result< JsNumber >
    {
        f.try_into_()
    }

    pub
    fn create_object (self: &'_ Env)
      -> Result< JsObject >
    {
        Ok(JsObject {
            __wasm: ::js_sys::Object::new()
        })
    }

    pub
    fn create_string (self: &'_ Env, s: &'_ str)
      -> Result< JsString >
    {
        Ok(JsString {
            __wasm: s.into(),
        })
    }

    pub
    fn create_string_from_std (self: &'_ Env, s: String)
      -> Result< JsString >
    {
        Ok(JsString {
            __wasm: s.into(),
        })
    }

    pub
    fn create_uint32 (self: &'_ Env, n: u32)
      -> Result< JsNumber >
    {
        n.try_into_()
    }

    pub
    fn get_boolean (self: &'_ Env, b: bool)
      -> Result< JsBoolean >
    {
        Ok(JsBoolean {
            __wasm: b.into(),
        })
    }

    pub
    fn get_null (self: &'_ Env)
      -> Result< JsNull >
    {
        Ok(JsNull {
            __wasm: JsValue::NULL,
        })
    }

    pub
    fn get_undefined (self: &'_ Env)
      -> Result< JsUndefined >
    {
        Ok(JsUndefined {
            __wasm: JsValue::UNDEFINED,
        })
    }
}
