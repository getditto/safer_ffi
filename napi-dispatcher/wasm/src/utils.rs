#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! new_type_wrappers {(
    $(
        $( #[$js_unknown:ident] )?
        $pub:vis
        type $JsTy:ident = $OrigTy:ty;
    )*
) => ($(
    #[repr(transparent)]
    #[derive(Debug, ::ref_cast::RefCast)]
    $pub
    struct $JsTy {
        pub __wasm: $OrigTy,
    }

    const _: () = {
        use ::wasm_bindgen::{
            convert::{
                FromWasmAbi,
                IntoWasmAbi,
            },
            JsCast,
        };

        impl $JsTy {
            pub
            fn into_unknown (self: $JsTy)
              -> JsUnknown
            {
                JsUnknown {
                    __wasm: self.into()
                }
            }
        }


        // do not emit this `impl` for `$JsTy = JsUnknown`.
        $(
            #[doc = stringify!($js_unknown)]
            #[cfg(any())]
        )?
        impl ::core::convert::TryFrom<JsUnknown> for $JsTy {
            type Error = JsValue;

            #[inline]
            fn try_from (it: JsUnknown)
              -> Result<$JsTy, JsValue>
            {
                Ok($JsTy {
                    __wasm: it.__wasm.dyn_into()?,
                })
            }
        }

        impl AsRef<JsValue> for $JsTy {
            #[inline]
            fn as_ref (self: &'_ Self)
              -> &'_ JsValue
            {
                &self.__wasm
            }
        }

        impl From<$JsTy> for JsValue {
            #[inline]
            fn from (it: $JsTy)
              -> JsValue
            {
                it.__wasm.into()
            }
        }

        impl JsCast for $JsTy {
            #[inline]
            fn instanceof (val: &'_ JsValue)
              -> bool
            {
                <$OrigTy>::instanceof(val)
            }

            #[inline]
            fn unchecked_from_js(val: JsValue)
              -> $JsTy
            {
                $JsTy {
                    __wasm: <$OrigTy>::unchecked_from_js(val),
                }
            }

            #[inline]
            fn unchecked_from_js_ref (val: &'_ JsValue)
              -> &'_ $JsTy
            {
                $JsTy::ref_cast(<$OrigTy>::unchecked_from_js_ref(val))
            }
        }

        impl FromWasmAbi for $JsTy {
            type Abi = <$OrigTy as FromWasmAbi>::Abi;

            #[inline]
            unsafe
            fn from_abi (
                abi: Self::Abi,
            ) -> $JsTy
            {
                $JsTy {
                    __wasm: <$OrigTy as FromWasmAbi>::from_abi(abi)
                }
            }
        }

        impl IntoWasmAbi for $JsTy {
            type Abi = <$OrigTy as IntoWasmAbi>::Abi;

            #[inline]
            fn into_abi (
                self: $JsTy,
            ) -> Self::Abi
            {
                <$OrigTy as IntoWasmAbi>::into_abi(self.__wasm)
            }
        }

        impl ::wasm_bindgen::describe::WasmDescribe for $JsTy {
            #[inline]
            fn describe ()
            {
                <$OrigTy as ::wasm_bindgen::describe::WasmDescribe>::describe()
            }
        }
    };
)*)}
pub(crate) use new_type_wrappers;

macro_rules! match_ {(
    $args:tt $rules:tt
) => (
    macro_rules! __recurse__ $rules
    __recurse__! $args;
)}
pub(crate) use match_;

pub(crate) trait TurboFish {
    fn into_<Dst>(self: Self) -> Dst
    where
        Self: Into<Dst>,
    {
        self.into()
    }

    fn try_into_<Dst>(self: Self) -> Result<Dst, <Self as ::core::convert::TryInto<Dst>>::Error>
    where
        Self: ::core::convert::TryInto<Dst>,
    {
        ::core::convert::TryInto::try_into(self)
    }

    fn as_ref_<Dst>(self: &'_ Self) -> &'_ Dst
    where
        Self: AsRef<Dst>,
    {
        self.as_ref()
    }
}
impl<T: ?Sized> TurboFish for T {}
