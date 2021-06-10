#![cfg_attr(rustfmt, rustfmt::skip)]

use super::*;

impl JsBoolean {
    pub
    fn get_value (self: &'_ JsBoolean)
      -> Result<bool>
    {
        Ok(self.__wasm.value_of())
    }
}

impl JsBuffer {
    pub
    fn into_value (self: &'_ JsBuffer)
      -> Result< Vec<u8> >
    {
        Ok(self.__wasm.to_vec())
    }
}

impl JsFunction {
    pub
    fn call (
        self: &'_ JsFunction,
        this: Option<&'_ JsObject>,
        args: &'_ [JsUnknown],
    ) -> Result<JsUnknown>
    {
        self.__wasm
            .apply(
                this.map_or(&JsValue::UNDEFINED, |it| it.as_ref()),
                &args.iter().map(|it| &it.__wasm).collect(),
            )
            .map(|__wasm| JsUnknown { __wasm })
    }
}

crate::utils::match_! {[
    u8, u16, u32, usize, u64,
    i8, i16, i32, isize, i64,
    f32, f64,
]
{(
    $($xN:ident),* $(,)?
) => (
    $(
        impl ::core::convert::TryFrom<JsNumber> for $xN {
            type Error = JsValue;

            fn try_from (js_number: JsNumber)
              -> Result<$xN, Self::Error>
            {
                Ok(js_number.__wasm.into_::<f64>() as _)
            }
        }

        impl ::core::convert::TryFrom<$xN> for JsNumber {
            type Error = JsValue;

            fn try_from ($xN: $xN)
              -> Result<JsNumber, Self::Error>
            {
                Ok(JsNumber {
                    __wasm: ($xN as f64).into(),
                })
            }
        }
    )*
)}}

impl JsPromise {
    #[doc(hidden)] /** Not part of the public API */ pub
    fn __resolve (value: &'_ JsValue)
      -> Self
    {
        Self {
            __wasm: ::js_sys::Promise::resolve(value),
        }
    }
}

impl JsObject {
    pub
    fn get_named_property<T : NapiValue> (
        self: &'_ JsObject,
        name: &'_ str,
    ) -> Result<T>
    {
        ::js_sys::Reflect::get(
            self.as_ref_::<JsValue>(),
            &JsValue::from_str(name),
        )
        // FIXME
        .and_then(|js_value| Ok(js_value.unchecked_into())) // .dyn_into())
    }

    pub
    fn set_named_property (
        self: &'_ mut JsObject,
        name: &'_ str,
        value: impl NapiValue,
    ) -> Result<()>
    {
        let success = ::js_sys::Reflect::set(
            self.as_ref_::<JsValue>(),
            &JsValue::from_str(name),
            value.as_ref_::<JsValue>(),
        )?;
        if success == false {
            return Err(JsValue::from_str(&format!(
                "`Reflect::set({:?}, {}, {:?})` yielded `false`",
                self.as_ref_::<JsValue>(),
                name,
                value.as_ref_::<JsValue>(),
            )));
        }
        Ok(())
    }
}

#[derive(Debug)] pub struct Utf8String(String);
impl JsString {
    pub
    fn into_utf8 (self: Self)
      -> Result<Utf8String>
    {
        impl Utf8String {
            pub
            fn as_str (self: &'_ Self)
              -> Result<&'_ str>
            {
                Ok(&self.0)
            }

            pub
            fn into_owned (self: Self)
              -> Result<String>
            {
                Ok(self.0)
            }

            pub
            fn take (self: Self)
              -> Vec<u8>
            {
                self.0.into()
            }
        }

        Ok(Utf8String(self.__wasm.into()))
    }
}

impl JsUnknown {
    pub
    fn get_type (self: &'_ JsUnknown)
      -> Result<ValueType>
    {
        #[::wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
            export function typeof_(x) {
                return typeof x;
            }
        "#)]
        extern "C" {
            fn typeof_ (x: &JsValue)
              -> String
            ;
        }
        Ok(match typeof_(self.as_ref()).as_str() {
            | "undefined" => ValueType::Undefined,
            | "object" if self.__wasm.is_null() => ValueType::Null,
            | "object" => ValueType::Object,
            | "boolean" => ValueType::Boolean,
            | "number" => ValueType::Number,
            | "bigint" => ValueType::BigInt,
            | "string" => ValueType::String,
            | "symbol" => ValueType::Symbol,
            | "function" => ValueType::Function,
            | _ => ValueType::Unknown,
        })
    }

    pub
    fn is_buffer (self: &'_ Self)
      -> Result<bool>
    {
        Ok(self.has_type::<JsBuffer>())
    }

    pub
    unsafe
    fn cast<Dst : NapiValue> (self: JsUnknown)
      -> Dst
    {
        self.unchecked_into()
    }
}
