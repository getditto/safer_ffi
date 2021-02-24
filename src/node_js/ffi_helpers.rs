use super::*;

#[js_function(2)]
pub
fn with_js_string_as_utf8 (ctx: CallContext<'_>)
  -> Result<impl NapiValue>
{
    let fst: JsUnknown = ctx.get(0)?;
    let cb: JsFunction = ctx.get(1)?;
    let mut bytes;
    let ptr: *const u8 = match fst.get_type() {
        | Ok(ValueType::Null) => ::core::ptr::null(),
        | Ok(ValueType::String) => {
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
            bytes.as_ptr()
        },
        | _ => return Err(Error::new(
            Status::InvalidArg,
            "First parameter must be `null` or a string".into(),
        )),
    };
    cb.call(None, &[
        ToNapi::to_napi_value(ptr, ctx.env)?.into_unknown(),
    ])
}

#[js_function(1)]
pub
fn char_p_boxed_to_js_string (ctx: CallContext<'_>)
  -> Result<JsString>
{
    let p: *mut crate::c_char = extract_arg(&ctx, 0)?;
    if p.is_null() {
        Err(Error::new(Status::InvalidArg, "Got `NULL`".into()))
    } else {
        let p: crate::prelude::char_p::Box = unsafe {
            ::core::mem::transmute(p)
        };
        ctx.env.create_string(p.to_str())
    }
}

#[macro_export]
macro_rules! node_js_export_ffi_helpers {() => (const _: () = {
    ::safer_ffi::node_js::registering::submit! {
        #![crate = ::safer_ffi::node_js::registering]

        ::safer_ffi::node_js::registering::NapiRegistryEntry::NamedMethod {
            name: "withFfiString",
            method: ::safer_ffi::node_js::ffi_helpers::with_js_string_as_utf8,
        }
    }

    ::safer_ffi::node_js::registering::submit! {
        #![crate = ::safer_ffi::node_js::registering]

        ::safer_ffi::node_js::registering::NapiRegistryEntry::NamedMethod {
            name: "charPBoxedIntoString",
            method: ::safer_ffi::node_js::ffi_helpers::char_p_boxed_to_js_string,
        }
    }
};)} pub use node_js_export_ffi_helpers as register;
