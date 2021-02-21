use super::*;

use ::core::convert::{TryFrom, TryInto};

match_! {(
    (u32, create_uint32 => u8, u16, u32),
    (i32, create_int32 => i8, i16, i32),
    (i64, create_int64 => u64, i64, isize),
) {
    (
        $(
            ($x32:ident, $create_x32:ident => $($xN:ident),* $(,)?)
            $(, $($rest:tt)*)?
        )?
    ) => ($(
        __recurse__! { $($($rest)*)? }

        $(
            impl ReprNapi for $xN {
                type NapiValue = JsNumber;

                fn from_napi_value (
                    _: &'_ Env,
                    napi_value: JsNumber,
                ) -> Result<$xN>
                {
                    let n: $x32 = napi_value.try_into()?;
                    n   .try_into()
                        .map_err(|_| {
                            Error::new(
                                Status::InvalidArg,
                                format!(
                                    "Numeric overflow: \
                                    parameter `{:?}` does not fit into a `{}`",
                                    n,
                                    ::core::any::type_name::<$xN>(),
                                ),
                            )
                        })
                }

                fn to_napi_value (
                    self: $xN,
                    env: &'_ Env,
                ) -> Result<JsNumber>
                {
                    let n: $x32 = self.try_into().map_err(|_| {
                        Error::from_reason(format!(
                            "Numeric overflow: \
                            value `{:?}` cannot be lossly converted into Js",
                            self,
                        ))
                    })?;
                    env.$create_x32(n)
                }
            }
        )*
    )?);
}}

impl ToNapi for crate::tuple::CVoid {
    type NapiValue = JsUndefined;

    fn to_napi_value (self: Self, env: &'_ Env)
      -> Result<JsUndefined>
    {
        env.get_undefined()
    }
}

match_! {( const, mut ) {
    ( $($mut:ident),* ) => (
        $(
            impl<T : 'static> ReprNapi for *$mut T {
                type NapiValue = JsUnknown;

                fn to_napi_value (self: *$mut T, env: &'_ Env)
                  -> Result<JsUnknown>
                {
                    if  ::core::any::TypeId::of::<T>()
                    ==  ::core::any::TypeId::of::<crate::c_char>()
                    {
                        // FIXME: although unlikely, this could be a `char_p::Ref`
                        let s: crate::prelude::char_p::Box = unsafe {
                            ::core::mem::transmute(self)
                        };
                        env .create_string(s.to_str())
                            .map(JsString::into_unknown)
                    } else {
                        <isize as ReprNapi>::to_napi_value(self as isize, env)
                            .map(JsNumber::into_unknown)
                    }
                }

                fn from_napi_value (env: &'_ Env, js_val: JsUnknown)
                  -> Result<*$mut T>
                {
                    if  ::core::any::TypeId::of::<T>()
                    ==  ::core::any::TypeId::of::<crate::c_char>()
                    {
                        crate::prelude::char_p::Box::try_from(
                            js_val
                                .coerce_to_string()?
                                .into_utf8()?
                                .into_owned()?
                        ).map_err(|e| Error::from_reason(format!(
                            "\
                                Failed to convert `{:?}` to a C string: \
                                encountered inner null byte\
                            ",
                            e.0,
                        )))
                        .map(|it| unsafe {
                            // FIXME: This leaks memory :/
                            ::core::mem::transmute(it)
                        })
                    } else {
                        let js_val = js_val.coerce_to_number()?;
                        <isize as ReprNapi>::from_napi_value(env, js_val)
                            .map(|addr| addr as _)
                    }
                }
            }
        )*
    );
}}

impl ToNapi for crate::prelude::char_p::Ref<'_> {
    type NapiValue = JsString;

    fn to_napi_value (
        self: Self,
        env: &'_ Env,
    ) -> Result<JsString>
    {
        env.create_string(self.to_str())
    }
}
// There could be an impl for `char_p::Box` as well.

impl FromNapi for crate::prelude::char_p::Box {
    type NapiValue = JsString;

    fn from_napi_value (
        env: &'_ Env,
        js_val: JsString,
    ) -> Result<Self>
    {
        Self::try_from(
            js_val
                .into_utf8()?
                .into_owned()?
        ).map_err(|e| Error::from_reason(format!(
            "\
                Failed to convert `{:?}` to a C string: \
                encountered inner null byte\
            ",
            e.0,
        )))
    }
}
