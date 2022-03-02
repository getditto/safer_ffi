#![cfg_attr(rustfmt, rustfmt::skip)]
use super::*;

use ::core::convert::{TryFrom, TryInto};

match_! {(
    (u32, create_uint32 => u8, u16, u32),
    (i32, create_int32 => i8, i16, i32),

    #[cfg(target_arch = "wasm32")]
    (u32, create_uint32 => usize),

    #[cfg(target_arch = "wasm32")]
    (i32, create_int32 => isize),

    (f64, create_double => f32, f64),
) {
    (
        $(
            $( #[$cfg:meta] )?
            ($x32:ident, $create_x32:ident => $($xN:ident),* $(,)?)
            $(, $($rest:tt)*)?
        )?
    ) => ($(
        __recurse__! { $($($rest)*)? }
        $(#[$cfg])?
        const _: () = {$(
            impl ReprNapi for $xN {
                type NapiValue = JsNumber;

                fn from_napi_value (
                    _: &'_ Env,
                    napi_value: JsNumber,
                ) -> Result<$xN>
                {
                    let n: $x32 = napi_value.try_into()?;
                    let n_mb_smaller: $xN = n as _;
                    if n_mb_smaller as $x32 != n {
                        Err(Error::new(
                            Status::InvalidArg,
                            $crate::std::format!(
                                "Numeric overflow: \
                                parameter `{:?}` does not fit into a `{}`",
                                n,
                                ::core::any::type_name::<$xN>(),
                            ),
                        ).into())
                    } else {
                        Ok(n_mb_smaller)
                    }
                }

                fn to_napi_value (
                    self: $xN,
                    env: &'_ Env,
                ) -> Result<JsNumber>
                {
                    let n: $x32 = self.try_into().map_err(|_| {
                        Error::from_reason($crate::std::format!(
                            "Numeric overflow: \
                            value `{:?}` cannot be losslessly converted into Js",
                            self,
                        ))
                    })?;
                    env.$create_x32(n)
                }
            }
        )*};
    )?);
}}

match_! {(
    usize => u64 get_u64,
    isize => i64 get_i64,
) {
    (
        $( $xsize:tt => $x64:tt $get_x64:tt, )*
    ) => (
        $(
            impl ReprNapi for $x64 {
                type NapiValue = JsUnknown;

                fn from_napi_value (
                    _: &'_ Env,
                    napi_value: JsUnknown
                ) -> Result<Self>
                {
                    match napi_value.get_type()? {
                        | ValueType::Bigint => {
                            let big_int: JsBigint = unsafe {
                                napi_value.cast()
                            };
                            let (value, was_lossless) = big_int.$get_x64()?;
                            if was_lossless {
                                Ok(value)
                            } else {
                                Err(Error::new(
                                    Status::InvalidArg,
                                    ::std::format!(
                                        "Numeric overflow: \
                                        parameter does not fit into a `{}`",
                                        ::core::any::type_name::<$x64>(),
                                    ),
                                ).into())
                            }
                        },
                        | ValueType::Number => {
                            let num: JsNumber = unsafe { napi_value.cast() };
                            let i: i64 = num.try_into()?;
                            i.try_into().map_err(|_| Error::new(
                                Status::InvalidArg,
                                ::std::format!(
                                    "Numeric overflow: \
                                    parameter {} does not fit into a `{}`",
                                    i,
                                    ::core::any::type_name::<$x64>(),
                                ),
                            ).into())
                        },
                        | _ => {
                            Err(Error::new(
                                Status::InvalidArg,
                                ::std::format!("`BigInt` or `number` expected"),
                            ).into())
                        },
                    }
                }

                fn to_napi_value (
                    self: Self,
                    env: &'_ Env,
                ) -> Result<JsUnknown>
                {
                    const MIN: i128 = 0 - ((1 << 53) - 1);
                    const MAX: i128 = 0 + ((1 << 53) - 1);
                    match self as i128 {
                        | MIN ..= MAX => {
                            env .create_int64(
                                    self.try_into()
                                        .expect("Unreachable")
                                )
                                .map(|it| it.into_unknown())
                        },
                        #[cfg(not(target_arch = "wasm32"))]
                        | i128 => {
                            let is_negative = i128 < 0;
                            let le_words = {
                                let u128: u128 = if is_negative {
                                    (-i128) as _
                                } else {
                                    i128 as _
                                };
                                ::alloc::vec![
                                    u128 as u64,
                                    (u128 >> 64) as u64,
                                ]
                            };
                            env .create_bigint_from_words(
                                    is_negative,
                                    le_words,
                                )?
                                .into_unknown()
                        },
                        #[cfg(target_arch = "wasm32")]
                        | i128 => Ok(
                            JsBigint::from_str_base_10(&i128.to_string())
                                .into_unknown()
                        ),
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            impl ReprNapi for $xsize {
                type NapiValue = <$x64 as ReprNapi>::NapiValue;

                fn from_napi_value (
                    env: &'_ Env,
                    napi_value: JsUnknown
                ) -> Result<Self>
                {
                    $x64::from_napi_value(env, napi_value)?
                        .try_into()
                        .map_err(|_| {
                            Error::new(
                                Status::InvalidArg,
                                ::std::format!(
                                    "Numeric overflow: \
                                    parameter does not fit into a `{}`",
                                    ::core::any::type_name::<$xsize>(),
                                ),
                            ).into()
                        })
                }

                fn to_napi_value (
                    self: Self,
                    env: &'_ Env,
                ) -> Result<Self::NapiValue>
                {
                    (self as $x64).to_napi_value(env)
                }
            }
        )*
    )
}}

match_! {( const, mut ) {
    ( $($mut:ident),* ) => (
        $(
            impl<T : 'static> ReprNapi for *$mut T
            where
                T : crate::layout::CType,
            {
                type NapiValue = JsUnknown;

                fn to_napi_value (self: *$mut T, env: &'_ Env)
                  -> Result<JsUnknown>
                {
                    let addr = (self as usize).to_napi_value(env)?;
                    let ty: JsString =
                        env.create_string_from_std(::std::format!(
                            "{pointee} {mut}*",
                            mut = if stringify!($mut) == "const" { "const " } else { "" },
                            pointee = <T as crate::layout::CType>::c_var(""),
                        ))?
                    ;

                    let mut obj = env.create_object()?;
                    obj.set_named_property("addr", addr)?;
                    obj.set_named_property("type", ty)?;
                    let map = obj;

                    // let Map: JsFunction = env.get_global()?.get_named_property("Map")?;
                    // let mut map = Map.new::<JsUnknown>(&[])?;

                    // map .get_named_property::<JsFunction>("set")?
                    //     .call(Some(&map), &[
                    //         env.create_string("addr")?.into_unknown(),
                    //         addr.into_unknown(),
                    //     ])?
                    // ;

                    // map .get_named_property::<JsFunction>("set")?
                    //     .call(Some(&map), &[
                    //         env.create_string("type")?.into_unknown(),
                    //         ty.into_unknown(),
                    //     ])?
                    // ;

                    Ok(map.into_unknown())
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
                            let js_buffer = JsBuffer::try_from(js_val)?;
                            let (buf, _storage): (&[u8], _);
                            #[cfg(target_arch = "wasm32")] {
                                _storage = ();
                                let bytes = js_buffer.into_value()?.into_boxed_slice();
                                let raw = Box::into_raw(bytes);
                                env.__push_drop_glue(::scopeguard::guard(raw, |raw| unsafe {
                                    drop(Box::from_raw(raw))
                                }));
                                buf = unsafe { &*raw };
                            } /* else */
                            #[cfg(not(target_arch = "wasm32"))] {
                                _storage = js_buffer.into_value()?;
                                buf = &_storage;
                            }
                            let buf = if let Ok(it) = ::core::str::from_utf8(buf) { it } else {
                                return Err(Error::new(
                                    Status::InvalidArg,
                                    ::std::format!(
                                        "Expected valid UTF-8 bytes {:#x?} for a string",
                                        buf,
                                    ),
                                ).into());
                            };
                            if buf.bytes().position(|b| b == b'\0') != Some(buf.len() - 1) {
                                return Err(Error::new(
                                    Status::InvalidArg,
                                    ::std::format!(
                                        "Invalid null terminator for {:?}",
                                        buf,
                                    ),
                                ).into());
                            }
                            return Ok(buf.as_ptr() as _);
                        },
                        | Js::Object => unsafe { js_val.cast() },
                        | _ => return Err(Error::new(
                            Status::InvalidArg,
                            "Expected a pointer (`{ addr }` object)".into(),
                        ).into()),
                    };
                    let addr = obj.get_named_property("addr")?;
                    let ty: JsString = obj.get_named_property("type")?;
                    let expected_ty: &str = &::std::format!(
                        "{pointee} {mut}*",
                        mut = if stringify!($mut) == "const" { "const " } else { "" },
                        pointee = <T as crate::layout::CType>::c_var(""),
                    );
                    let actual_ty = ty.into_utf8()?;
                    let mut actual_ty = actual_ty.as_str()?;
                    let storage;
                    if stringify!($mut) == "const" {
                        storage = actual_ty.replace("const *", "*").replace(" *", " const *");
                        actual_ty = &storage;
                    }
                    if actual_ty != expected_ty {
                        return Err(Error::new(
                            Status::InvalidArg,
                            ::std::format!(
                                "Got `{}`, expected a `{}`",
                                actual_ty, expected_ty,
                            ),
                        ).into());
                    }
                    // let addr: JsNumber =
                    //     obj .get_named_property::<JsFunction>("get")?
                    //         .call(Some(&obj), &[
                    //             env.create_string("addr")?.into_unknown(),
                    //         ])?
                    //         .try_into()?
                    // ;
                    <usize as ReprNapi>::from_napi_value(env, addr)
                        .map(|addr| addr as _)
                }
            }
        )*
    );
}}

match_! {(
    for[T] ::core::marker::PhantomData<T>,
    for[] crate::tuple::CVoid,
) {
    (
        $(
            for[$($generics:tt)*] $T:ty
        ),* $(,)?
    ) => (
        $(
            impl<$($generics)*> ReprNapi for $T
            where
                Self : crate::layout::CType,
            {
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
                        Ok(::core::mem::transmute(()))
                    }
                }
            }
        )*
    )
}}

impl<const N: usize> ReprNapi for [u8;N] {

    type NapiValue = JsBuffer;

    #[inline]
    fn to_napi_value(self, env: &'_ Env) -> Result<JsBuffer> {
        env.create_buffer_copy(&self[..]).map(|x| {x.into_raw() })
    }

    fn from_napi_value(_env: &'_ Env, buffer: JsBuffer) -> Result<Self> {
        let val = buffer.into_value()?;
        if let Ok(output) = Self::try_from(&val[..]) {
            Ok(output)
        } else {
            return Err(Error::new(Status::InvalidArg, ::std::format!("Length mismatch. Expected {}, Got {}", N, val.len())).into())
        }
    }
}
