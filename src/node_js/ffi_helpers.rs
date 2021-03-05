use super::*;

use ::core::convert::TryFrom;

#[js_function(2)]
pub
fn with_js_string_as_utf8 (ctx: CallContext<'_>)
  -> Result<impl NapiValue>
{
    use ValueType as Js;
    let fst: JsUnknown = ctx.get(0)?;
    let cb: JsFunction = ctx.get(1)?;
    let mut bytes;
    let ptr: *const crate::c_char = match fst.get_type() {
        | Ok(Js::Null) => crate::NULL!(),
        | Ok(Js::String) => {
            let s = unsafe { fst.cast::<JsString>() };
            bytes = s.into_utf8()?.take();
            match bytes.iter().position(|&b| b == b'\0') {
                | Some(n) if n == bytes.len() - 1 => {},
                | Some(inner_nul_idx) => return Err(Error::new(
                    Status::InvalidArg,
                    format!(
                        "String `{:?}` contains an inner nul byte at byte-index {}",
                        String::from_utf8_lossy(&bytes),
                        inner_nul_idx,
                    ),
                )),
                | None => {
                    bytes.reserve_exact(1);
                    bytes.push(b'\0');
                },
            }
            bytes.as_ptr().cast()
        },
        | _ => return Err(Error::new(
            Status::InvalidArg,
            "First parameter must be `null` or a string".into(),
        )),
    };
    cb.call(None, &[
        ReprNapi::to_napi_value(ptr, ctx.env)?.into_unknown(),
    ])
}

#[js_function(2)]
pub
fn with_js_buffer_as_slice_uint8_t_ref (ctx: CallContext<'_>)
  -> Result<impl NapiValue>
{
    let fst: JsUnknown = ctx.get(0)?;
    let cb: JsFunction = ctx.get(1)?;
    match () {
        _case if fst.is_buffer()? => {
            let xs: &'_ [u8] = &JsBuffer::try_from(fst)?.into_value()?;
            let xs: crate::prelude::c_slice::Ref<'_, u8> = xs.into();
            cb.call(None, &[
                crate::slice::slice_raw_Layout::<u8>::to_napi_value(
                    unsafe { ::core::mem::transmute(xs) },
                    ctx.env,
                )?.into_unknown(),
            ])
        },
        _case if matches!(fst.get_type(), Ok(ValueType::Null)) => {
            cb.call(None, &[
                ReprNapi::to_napi_value(
                    crate::slice::slice_raw_Layout::<u8> {
                        ptr: crate::NULL!(),
                        len: 0xbad000,
                    },
                    ctx.env,
                )?.into_unknown(),
            ])
        },
        _default => {
            Err(Error::new(
                Status::InvalidArg,
                "Expected a `Buffer`".into(),
            ))
        },
    }
}

#[js_function(1)]
pub
fn char_p_boxed_to_js_string (ctx: CallContext<'_>)
  -> Result<JsUnknown>
{Ok({
    let p: *mut crate::c_char = extract_arg(&ctx, 0)?;
    if p.is_null() {
        // return Err(Error::new(Status::InvalidArg, "Got `NULL`".into()));
        ctx .env
            .get_null()?
            .into_unknown()
    } else {
        let p: crate::prelude::char_p::Box = unsafe {
            crate::layout::from_raw_unchecked(p)
        };
        ctx .env
            .create_string(p.to_str())?
            .into_unknown()
    }
})}

#[js_function(1)]
pub
fn slice_box_uint8_t_to_js_buffer (ctx: CallContext<'_>)
  -> Result<JsUnknown>
{Ok({
    let fat: crate::slice::slice_boxed_Layout<u8> = extract_arg(&ctx, 0)?;
    if fat.ptr.is_null() {
        // return Err(Error::new(Status::InvalidArg, "Got `NULL`".into()));
        ctx .env
            .get_null()?
            .into_unknown()
    } else {
        let p: crate::prelude::c_slice::Box<u8> = unsafe {
            crate::layout::from_raw_unchecked(fat)
        };
        ctx .env
            .create_buffer_copy(p.as_ref().as_slice())?
            .into_unknown()
    }
})}

match_! {(
    "withCString": with_js_string_as_utf8,
    "boxCStringIntoString": char_p_boxed_to_js_string,
    "withCBytes": with_js_buffer_as_slice_uint8_t_ref,
    "boxCBytesIntoBuffer": slice_box_uint8_t_to_js_buffer,
) {[ $( $name:literal : $fun:ident ),* $(,)? ] => (
    #[macro_export]
    macro_rules! node_js_export_ffi_helpers {() => (const _: () = {
        $(
            $crate::node_js::registering::submit! {
                #![crate = $crate::node_js::registering]

                $crate::node_js::registering::NapiRegistryEntry::NamedMethod {
                    name: $name,
                    method: $crate::node_js::ffi_helpers::$fun,
                }
            }
        )*
    };)}
)}}

pub use node_js_export_ffi_helpers as register;
