#![cfg_attr(rustfmt, rustfmt::skip)]
use super::{*,
    derive::js_export,
};
use crate::prelude::*;
use ::core::convert::TryFrom;

type JsOf<T> = <crate::layout::CLayoutOf<T> as ReprNapi>::NapiValue;
type JsOfCSliceRef<T> = JsOf<c_slice::Ref<'static, T>>;

/// This module can, oddly enough, end up optimized by the linkage step, especially on macOS (on
/// `--[profile ]release`).
///
/// See:
///   - <https://github.com/getditto/safer_ffi/issues/161>
///   - <https://github.com/dtolnay/inventory/issues/52>
///
/// The most heavyweight but effective workaround is to add `-C link-dead-code` to the `rustflags`,
/// so that `rustc` not invoke the linker with the `--gc-sections` flag from which these problematic
/// elisions stem.
///   - <https://github.com/rust-lang/rust/issues/17141>
///   - Due to worries about the resulting binary size, other venues are explored.
///
/// The aforementioned `inventory` issue also suggests forcing the `--[profile ]release` to use
/// `codegen-units = 1`. This slows down compilation, and, seems quite brittle, so we keep looking.
///
/// The comments over <https://github.com/rust-lang/rust/issues/94348> suggest another trick: to
/// have a non-`static` item be "exported" / visible "outside" (the crate? This remains unclear, but
/// seems plausible).
///   - <https://github.com/rust3ds/pthread-3ds/blob/90bd4d00e371ea5dbd8487c5a63d3d75d667f796/src/lib.rs#L8-L12>
///
/// Experiments show that this does work, here, but only when adding `#[no_mangle]`/`#[export_name]`
/// (probably just because it ensures the seemingly necessary cross-crate visibility).
///   - (`export_name` is used over `no_mangle` to more easily guarantee unicity of the so-exported
///     symbol, should this approach end up needed elsewhere.)
///
/// Notable failed attempt: the official proper tool for this task, the `#[used(linker)]` directive,
/// did not help in this case, alas, no matter how much was tweaked the codegen of `#[js_export]`,
/// of `inventory::submit!`, or of custom hard-coded things.
///   - <https://github.com/dtolnay/inventory/pull/61>
#[export_name = "_ZN9safer_ffi2js11ffi_helpers42_prevent_improper_linker_dead_code_elisionE"]
pub
fn _prevent_improper_linker_dead_code_elision(unreachable: ::core::convert::Infallible)
{
    match unreachable {}
}

#[js_export(js_name = withCString, __skip_napi_import)]
pub
fn with_js_string_as_utf8 (
    fst: JsUnknown,
    cb: JsFunction,
) -> Result<JsUnknown>
{
    use ValueType as Js;

    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let mut bytes;
    let ptr: *const crate::c_char = match fst.get_type() {
        | Ok(Js::Null) => NULL!(),
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
                ).into()),
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
        ).into()),
    };
    cb.call(None, &[
        ReprNapi::to_napi_value(ptr, ctx.env)?.into_unknown(),
    ])
}

#[js_export(js_name = withCBytes, __skip_napi_import)]
pub
fn with_js_buffer_as_slice_uint8_t_ref (
    fst: JsUnknown,
    cb: JsFunction,
) -> Result<JsUnknown>
{
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
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
                        ptr: NULL!(),
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
            ).into())
        },
    }
}

#[js_export(js_name = boxCStringIntoString, __skip_napi_import)]
pub
fn char_p_boxed_to_js_string (
    arg: <*mut crate::c_char as ReprNapi>::NapiValue,
) -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let p: *mut crate::c_char = ReprNapi::from_napi_value(ctx.env, arg)?;
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

#[cfg(not(target_arch = "wasm32"))]
#[js_export(js_name = getDeadlockTimeout, __skip_napi_import)]
pub
fn get_deadlock_timeout_js() -> JsValue
{
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let val: u32 = match get_deadlock_timeout() {
        Some(val) => val.try_into().unwrap(),
        None => 0u32,
    };
    ctx.env.create_uint32(val)
}

#[cfg(not(target_arch = "wasm32"))]
#[js_export(js_name = setDeadlockTimeout, __skip_napi_import)]
pub
fn set_deadlock_timeout_js(
    val: JsNumber,
) -> Result<JsUndefined>
{
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let val: Option<std::num::NonZeroU32> = match val.try_into() {
        // Maps an input 0 to `None`
        Ok(val) => std::num::NonZeroU32::new(val),
        Err(_) => Err(Error::new(
            Status::InvalidArg,
            "Expected a positive number".into(),
        ))?,
    };
    // Safe to unwrap because we should always be able to get the `undefined` object.
    set_deadlock_timeout(val).map(|_| ctx.env.get_undefined().unwrap())
}

#[js_export(js_name = refCStringToString, __skip_napi_import, __skip_napi_import)]
pub
fn char_p_ref_to_js_string (
    arg: <*const crate::c_char as ReprNapi>::NapiValue,
) -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let p: *const crate::c_char = ReprNapi::from_napi_value(ctx.env, arg)?; // extract_arg(&ctx, 0)?;
    if p.is_null() {
        // return Err(Error::new(Status::InvalidArg, "Got `NULL`".into()));
        ctx .env
            .get_null()?
            .into_unknown()
    } else {
        let p: crate::prelude::char_p::Ref<'_> = unsafe {
            crate::layout::from_raw_unchecked(p)
        };
        ctx .env
            .create_string(p.to_str())?
            .into_unknown()
    }
})}

// #[js_function(1)]
// pub
// fn char_p_ref_to_js_string (ctx: CallContext<'_>)
//   -> Result<JsUnknown>
// {Ok({
//     let p: *const crate::c_char = extract_arg(&ctx, 0)?;
//     if p.is_null() {
//         // return Err(Error::new(Status::InvalidArg, "Got `NULL`".into()));
//         ctx .env
//             .get_null()?
//             .into_unknown()
//     } else {
//         let p: crate::prelude::char_p::Ref<'_> = unsafe {
//             crate::layout::from_raw_unchecked(p)
//         };
//         ctx .env
//             .create_string(p.to_str())?
//             .into_unknown()
//     }
// })}

#[js_export(js_name = boxCBytesIntoBuffer, __skip_napi_import)]
pub
fn slice_box_uint8_t_to_js_buffer (
    arg: <crate::slice::slice_boxed_Layout<u8> as ReprNapi>::NapiValue,
) -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let fat = crate::slice::slice_boxed_Layout::<u8>::from_napi_value(ctx.env, arg)?;
    if fat.ptr.is_null() {
        ctx .env
            .get_null()?
            .into_unknown()
    } else {
        let p: crate::prelude::repr_c::Box<[u8]> = unsafe {
            crate::layout::from_raw_unchecked(fat)
        };
        ctx .env
            .create_buffer_copy(p.as_ref().as_slice())?
            .into_unknown()
    }
})}

#[js_export(js_name = refCBytesIntoBuffer, __skip_napi_import)]
pub
fn slice_ref_uint8_t_to_js_buffer (
    arg: JsOfCSliceRef<u8>,
) -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let fat = crate::slice::slice_ref_Layout::<'_, u8>::from_napi_value(ctx.env, arg)?;
    if fat.ptr.is_null() {
        ctx .env
            .get_null()?
            .into_unknown()
    } else {
        let p: crate::prelude::c_slice::Ref<'_, u8> = unsafe {
            crate::layout::from_raw_unchecked(fat)
        };
        ctx .env
            .create_buffer_copy(p.as_slice())?
            .into_unknown()
    }
})}


#[js_export(js_name = withOutBoolean, __skip_napi_import)]
pub
fn with_out_bool (cb: JsFunction)
  -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let mut b = false;
    let out_bool = &mut b;
    cb.call(None, &[
        ReprNapi::to_napi_value(
            unsafe { crate::layout::into_raw(out_bool) },
            ctx.env,
        )?.into_unknown(),
    ])?;
    ReprNapi::to_napi_value(unsafe { crate::layout::into_raw(b) }, ctx.env)?
        .into_unknown()
})}

#[js_export(js_name = withOutU64, __skip_napi_import)]
pub
fn with_out_u64 (cb: JsFunction)
  -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let mut u64 = 0;
    let out_u64 = &mut u64;
    cb.call(None, &[
        ReprNapi::to_napi_value(
            unsafe { crate::layout::into_raw(out_u64) },
            ctx.env,
        )?.into_unknown(),
    ])?;
    ReprNapi::to_napi_value(unsafe { crate::layout::into_raw(u64 as i64) }, ctx.env)?
        .into_unknown()
})}

fn wrap_ptr (env: &'_ Env, p: *mut (), ty: &'_ str)
  -> Result<JsUnknown>
{
    let mut obj = env.create_object()?;
    obj.set_named_property(
        "addr",
        ReprNapi::to_napi_value(p as usize, env)?,
    )?;
    obj.set_named_property(
        "type",
        env.create_string(ty)?,
    )?;
    Ok(obj.into_unknown())
}

#[js_export(js_name = withOutBoxCBytes, __skip_napi_import)]
pub
fn with_out_byte_slice (cb: JsFunction)
  -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let ty = &"slice_boxed_uint8_t";
    let mut v = crate::slice::slice_ref_Layout::<()> {
        ptr: NULL!(),
        len: 0,
        _lt: unsafe { ::core::mem::transmute(()) },
    };
    let out_v = &mut v;
    cb.call(None, &[
        wrap_ptr(ctx.env, <*mut _>::cast(out_v), &format!("{} *", ty))?
    ])?;
    let mut v_js = ctx.env.create_object()?;
    v_js.set_named_property(
        "ptr", wrap_ptr(ctx.env, v.ptr as _, "uint8_t *")?,
    )?;
    v_js.set_named_property(
        "len", ReprNapi::to_napi_value(v.len as usize, ctx.env)?,
    )?;
    v_js.into_unknown()
})}

#[js_export(js_name = withOutVecOfPtrs, __skip_napi_import)]
pub
fn with_out_vec_of_ptrs (
    vec_ty: JsString,
    ty: JsString,
    cb: JsFunction,
) -> Result<JsUnknown>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let ref vec_ty: String = vec_ty.into_utf8()?.into_owned()?;
    let ref ty: String = ty.into_utf8()?.into_owned()?;
    let mut v = crate::vec::Vec_Layout::<()> {
        ptr: NULL!(),
        len: 0,
        cap: 0,
    };
    let out_v = &mut v;
    cb.call(None, &[
        wrap_ptr(
            ctx.env,
            <*mut _>::cast(out_v),
            &format!("{} *", vec_ty),
        )?
    ])?;
    let mut v_js = ctx.env.create_object()?;
    v_js.set_named_property(
        "ptr", wrap_ptr(ctx.env, v.ptr.cast(), &format!("{} *", ty))?,
    )?;
    v_js.set_named_property(
        "len", ReprNapi::to_napi_value(v.len as usize, ctx.env)?,
    )?;
    v_js.set_named_property(
        "cap", ReprNapi::to_napi_value(v.cap as usize, ctx.env)?,
    )?;
    v_js.into_unknown()
})}

#[js_export(js_name = cStringVecToStringArray, __skip_napi_import)]
pub
fn vec_char_ptr_to_js_string_array (
    arg: <crate::vec::Vec_Layout::<char_p::Box> as ReprNapi>::NapiValue,
) -> Result<JsObject>
{Ok({
    let ctx = ::safer_ffi::js::derive::__js_ctx!();
    let arg = crate::vec::Vec_Layout::from_napi_value(ctx.env, arg)?;
    let v: repr_c::Vec<char_p::Box> = unsafe { crate::layout::from_raw_unchecked(arg) };
    let v: Vec<char_p::Box> = v.into();
    let ret = ctx.env.create_array()?;
    let push = {
        let ret = &ret;
        let push_method = ret.get_named_property::<JsFunction>("push")?;
        move |elem| push_method.call(
            Some(ret),
            &[elem],
        )
    };
    for p in v {
        push(
            ctx .env
                .create_string(p.to_str())?
                .into_unknown()
        )?;
    }
    ret
})}

// match_! {(
//     "withCString": with_js_string_as_utf8,
//     "boxCStringIntoString": char_p_boxed_to_js_string,
//     "refCStringToString": char_p_ref_to_js_string,
//     "withCBytes": with_js_buffer_as_slice_uint8_t_ref,
//     "boxCBytesIntoBuffer": slice_box_uint8_t_to_js_buffer,
//     "withOutBoolean": with_out_bool,
//     "withOutU64": with_out_u64,
//     "withOutBoxCBytes": with_out_byte_slice,
//     "withOutVecOfPtrs": with_out_vec_of_ptrs,
//     "cStringVecToStringArray": vec_char_ptr_to_js_string_array,
// ) {[ $( $name:literal : $fun:ident ),* $(,)? ] => (
//     #[macro_export]
//     macro_rules! js_export_ffi_helpers {() => (const _: () = {
//         $(
//             $crate::js::registering::submit! {
//                 #![crate = $crate::js::registering]

//                 $crate::js::registering::NapiRegistryEntry::NamedMethod {
//                     name: $name,
//                     method: $crate::js::ffi_helpers::$fun,
//                 }
//             }
//         )*
//     };)}
// )}}
// pub use js_export_ffi_helpers as register;
