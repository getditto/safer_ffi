use super::*;

use ::core::convert::{TryFrom, TryInto};

match_! {(
    (u32, create_uint32 => u8, u16, u32),
    (i32, create_int32 => i8, i16, i32),
    (i64, create_int64 => u64, i64, isize, usize),
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

match_! {( const, mut ) {
    ( $($mut:ident),* ) => (
        $(
            impl<T : 'static> ReprNapi for *$mut T
            where
                Self : crate::layout::CType,
            {
                type NapiValue = JsUnknown;

                fn to_napi_value (self: *$mut T, env: &'_ Env)
                  -> Result<JsUnknown>
                {
                    let addr: JsNumber =
                        <isize as ReprNapi>::to_napi_value(self as isize, env)?
                    ;
                    let mut obj = env.create_object()?;
                    obj.set_named_property("addr", addr)?;
                    Ok(obj.into_unknown())
                }

                fn from_napi_value (env: &'_ Env, js_val: JsUnknown)
                  -> Result<*$mut T>
                {
                    use ValueType as Js;
                    use ::core::any::TypeId as Ty;
                    let obj: JsObject = match js_val.get_type()? {
                        | Js::Null => return Ok(0 as _),

                        | _ if Ty::of::<Self>()
                            == Ty::of::<*const crate::c_char>()
                            && js_val.is_buffer()?
                        => {
                            let buf: &[u8] =
                                &JsBuffer::try_from(js_val)?
                                    .into_value()?
                            ;
                            let buf = if let Ok(it) = ::core::str::from_utf8(buf) { it } else {
                                return Err(Error::new(
                                    Status::InvalidArg,
                                    format!(
                                        "Expected valid UTF-8 bytes {:#x?} for a string",
                                        buf,
                                    ),
                                ));
                            };
                            if buf.bytes().position(|b| b == b'\0') != Some(buf.len() - 1) {
                                return Err(Error::new(
                                    Status::InvalidArg,
                                    format!(
                                        "Invalid null terminator for {:?}",
                                        buf,
                                    ),
                                ));
                            }
                            return Ok(buf.as_ptr() as _);
                        },
                        | Js::Object => unsafe { js_val.cast() },
                        | _ => return Err(Error::new(
                            Status::InvalidArg,
                            "Expected a pointer (`{ addr }` object)".into(),
                        )),
                    };
                    let addr: JsNumber = obj.get_named_property("addr")?;
                    <isize as ReprNapi>::from_napi_value(env, addr)
                        .map(|addr| addr as _)
                }
            }
        )*
    );
}}

pub trait ZstAsUndefined : crate::layout::CType {}

impl<T : ZstAsUndefined> ReprNapi for T {
    type NapiValue = JsUndefined;

    fn to_napi_value (self, env: &'_ Env)
      -> Result<JsUndefined>
    {
        env.get_undefined()
    }

    fn from_napi_value (_: &'_ Env, _: JsUndefined)
      -> Result<Self>
    {
        unsafe {
            assert_eq!(::core::mem::size_of::<Self>(), 0);
            Ok(::core::mem::transmute_copy(&()))
        }
    }
}

impl<T> ZstAsUndefined for ::core::marker::PhantomData<T>
where
    Self : crate::layout::CType,
{}

impl ZstAsUndefined for crate::tuple::CVoid {}
// impl<T> ZstAsUndefined for ::core::marker::PhantomData<T> {}
// // Zsts
// match_! {(
//     for [T] ::core::marker::PhantomData<T>,
// ) {
//     ($(
//         for [$($generics:tt)*] $T:ty
//         impl<$($generics)*> ReprNapi for $T {
//             type NapiValue = JsUndefined;


//         }
//     ))
// }}

// impl ToNapi for crate::prelude::char_p::Ref<'_> {
//     type NapiValue = JsString;

//     fn to_napi_value (
//         self: Self,
//         env: &'_ Env,
//     ) -> Result<JsString>
//     {
//         env.create_string(self.to_str())
//     }
// }
// // There could be an impl for `char_p::Box` as well.

// impl FromNapi for crate::prelude::char_p::Box {
//     type NapiValue = JsString;

//     fn from_napi_value (
//         _: &'_ Env,
//         js_val: JsString,
//     ) -> Result<Self>
//     {
//         Self::try_from(
//             js_val
//                 .into_utf8()?
//                 .into_owned()?
//         ).map_err(|e| Error::from_reason(format!(
//             "\
//                 Failed to convert `{:?}` to a C string: \
//                 encountered inner null byte\
//             ",
//             e.0,
//         )))
//     }
// }
