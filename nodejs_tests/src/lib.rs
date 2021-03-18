use ::safer_ffi::prelude::*;

#[cfg(feature = "nodejs")]
const _: () = {
    ::safer_ffi::node_js::register_exported_functions!();
    ::safer_ffi::node_js::ffi_helpers::register!();
};

#[ffi_export(node_js)]
fn add (x: i32, y: i32)
  -> i32
{
    i32::wrapping_add(x, y)
}

#[derive_ReprC]
#[ReprC::opaque]
pub
struct Foo { opaque: i32 }

#[ffi_export(node_js)]
fn foo_new () -> repr_c::Box<Foo>
{
    Box::new(Foo { opaque: 42 })
        .into()
}

#[ffi_export(node_js)]
fn foo_read (foo: &'_ Foo)
  -> i32
{
    foo.opaque
}

#[ffi_export(node_js)]
fn foo_free (_p: Option<repr_c::Box<Foo>>)
{}

#[ffi_export(node_js)]
fn print (s: char_p::Ref<'_>)
{
    println!("{}", s);
}

#[ffi_export(node_js)]
fn concat (s1: char_p::Ref<'_>, s2: char_p::Ref<'_>)
  -> char_p::Box
{
    format!("{}{}", s1, s2)
        .try_into()
        .unwrap()
}

#[ffi_export(node_js)]
fn concat_bytes (
    xs1: Option<c_slice::Ref<'_, u8>>,
    xs2: Option<c_slice::Ref<'_, u8>>,
) -> Option<c_slice::Box<u8>>
{Some({
    [xs1?.as_slice(), xs2?.as_slice()]
        .concat()
        .into_boxed_slice()
        .into()
})}

#[ffi_export(node_js)]
fn get_hello() -> char_p::Box
{
    char_p::new("Hello, World!")
}

#[ffi_export]
unsafe
fn call_with_42 (
    data: *mut ::std::os::raw::c_void,
    cb: unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, x: i32) -> u8,
    release: unsafe extern "C" fn(data: *mut ::std::os::raw::c_void),
    retain: unsafe extern "C" fn(data: *mut ::std::os::raw::c_void),
) -> u8
{
    retain(data);
    ::std::thread::spawn({
        let data = data as usize;
        move || {
            dbg!(cb(data as _, 42));
            release(data as _);
        }
    });

    let ret = dbg!(cb(data, 42));
    release(data);
    ret
}

#[cfg(feature = "nodejs")]
const _: () = {
    use ::safer_ffi::node_js as napi;

    #[napi::js_function(1)]
    fn call_with_42_js (ctx: napi::CallContext<'_>)
      -> napi::Result<napi::JsNumber>
    {
        let cb: napi::Closure<fn(i32) -> u8> =
            napi::extract_arg(&ctx, 0)?
        ;
        let raw_cb = ::std::sync::Arc::new(cb).into_raw_parts();
        let raw_ret = unsafe {
            call_with_42(raw_cb.data, raw_cb.call, raw_cb.release, raw_cb.retain)
        };
        ctx.env
            .create_uint32(raw_ret as _)
        // ctx.env.get_undefined()
    }

    napi::registering::submit! {
        #![crate = napi::registering]

        napi::registering::NapiRegistryEntry::NamedMethod {
            name: "call_with_42",
            method: call_with_42_js,
        }
    }
};

#[ffi_export(node_js)]
fn set_bool (b: Out<'_, bool>)
{
    b.write(true);
}
