#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(unused_parens)]

use ::safer_ffi::prelude::*;

#[cfg(feature = "nodejs")]
const _: () = {
    ::safer_ffi::node_js::register_exported_functions!();
};

#[ffi_export(node_js)]
fn setup ()
{
    #[cfg(target_arch = "wasm32")] {
        ::console_error_panic_hook::set_once();
    }
}

#[ffi_export(node_js, rename = "add")]
fn add_with_a_weird_rust_name (x: i32, y: i32)
  -> i32
{
    i32::wrapping_add(x, y)
}

#[derive_ReprC(js)]
#[repr(C)]
pub
struct Point {
    x: f64,
    y: f64,
}

#[ffi_export(node_js)]
fn middle_point (
    a: Point,
    b: Point,
) -> Point
{
    Point {
        x: (a.x + b.x) / 2.,
        y: (a.y + b.y) / 2.,
    }
}

#[derive_ReprC]
#[repr(opaque)]
pub
struct Foo { opaque: i32 }

#[ffi_export(node_js)]
fn foo_new ()
  -> repr_c::Box<Foo>
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
fn concat_byte_slices (
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
fn get_hello ()
  -> char_p::Box
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
    thread_local! {
        static CB
            : ::core::cell::Cell<
                Option<(
                    *mut ::std::os::raw::c_void,
                    unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, x: i32) -> u8,
                    unsafe extern "C" fn(data: *mut ::std::os::raw::c_void),
                )>
            >
            = None.into()
        ;
    }
    let ret;
    if let Some((data, cb, release)) = CB.with(|it| it.replace(None)) {
        ret = cb(data, 42);
        release(data);
    } else {
        retain(data);
        CB.with(|it| it.set(Some((data, cb, release))));

        if cfg!(target_arch = "wasm32").not() {
            retain(data);
            ::std::thread::spawn({
                let data = data as usize;
                move || {
                    cb(data as _, 42);
                    release(data as _);
                }
            });
        }

        ret = cb(data, 42);
    }
    release(data);
    ret
}

#[cfg(feature = "nodejs")]
const _: () = {
    use ::safer_ffi::node_js as napi;

    #[cfg(not(target_arch = "wasm32"))]
    const _: () = {
        static LOCKED: ::std::sync::atomic::AtomicBool = {
            ::std::sync::atomic::AtomicBool::new(false)
        };

        #[ffi_export(node_js)]
        fn spinlock_aquire ()
        {
            while LOCKED.swap(true, ::std::sync::atomic::Ordering::Acquire) {}
        }

        #[ffi_export(node_js)]
        fn spinlock_release ()
        {
            LOCKED.store(false, ::std::sync::atomic::Ordering::Release);
        }

        #[napi::derive::js_export]
        fn call_detached (
            arg: (
                <
                    napi::Closure<fn(), napi::SyncKind::Detached> as napi::ReprNapi
                >::NapiValue
            ),
        ) -> napi::Result<napi::JsNumber>
        {
            let ctx = napi::derive::__js_ctx!();
            let cb: napi::Closure<fn(), napi::SyncKind::Detached> =
                napi::ReprNapi::from_napi_value(ctx.env, arg)?
            ;
            let (data_ptr, enqueue_call_fn) = cb.as_raw_parts();
            unsafe { enqueue_call_fn(data_ptr); }
            ctx.env.get_undefined()
        }
    };

    #[napi::derive::js_export(js_name = call_with_42)]
    fn call_with_42_js (
        arg: <napi::Closure<fn(i32) -> u8> as napi::ReprNapi>::NapiValue,
    ) -> napi::Result<napi::JsNumber>
    {
        let ctx = napi::derive::__js_ctx!();
        let mut cb: napi::Closure<fn(i32) -> u8> =
            napi::ReprNapi::from_napi_value(ctx.env, arg)?
        ;
        cb.make_nodejs_wait_for_this_to_be_dropped(true)?;
        let raw_cb = ::std::sync::Arc::new(cb).into_raw_parts();
        let raw_ret = unsafe {
            call_with_42(raw_cb.data, raw_cb.call, raw_cb.release, raw_cb.retain)
        };
        ctx .env
            .create_uint32(raw_ret as _)
    }

    #[napi::derive::js_export]
    fn call_with_str (
        arg: <napi::Closure<fn(char_p::Raw)> as napi::ReprNapi>::NapiValue,
    ) -> napi::Result<napi::JsUndefined>
    {
        let ctx = napi::derive::__js_ctx!();
        let mut cb: napi::Closure<fn(char_p::Raw)> =
            napi::ReprNapi::from_napi_value(ctx.env, arg)?
        ;
        cb.make_nodejs_wait_for_this_to_be_dropped(true)?;
        let (data, call) = cb.as_raw_parts();
        unsafe {
            call(data, c!("Hello, World!").to_str().as_ptr().cast());
        }
        ctx .env
            .get_undefined()
    }
};

#[ffi_export(node_js)]
fn set_bool (b: Out<'_, bool>)
{
    b.write(true);
}

#[ffi_export(node_js)]
fn takes_out_vec (v: &mut Option<repr_c::Vec<u8>>)
{
    *v = Some(vec![42, 27].into());
}

#[ffi_export(node_js)]
fn takes_out_slice (v: &mut Option<c_slice::Box<u8>>)
{
    *v = Some(vec![42, 27].into_boxed_slice().into());
}

#[derive_ReprC(js)]
#[repr(C)]
pub enum MyBool { True, False = 1 }

#[ffi_export(node_js)]
fn boolify (b: MyBool)
  -> bool
{
    matches!(b, MyBool::True)
}

#[derive_ReprC(js)]
#[repr(u8)]
pub enum MyBool2 { True, False = 1 }

#[ffi_export(node_js)]
fn boolify2 (b: MyBool2)
  -> bool
{
    matches!(b, MyBool2::True)
}

#[ffi_export(node_js(async_worker))]
fn long_running ()
  -> i32
{
    if cfg!(not(target_arch = "wasm32")) {
        ::std::thread::sleep(::std::time::Duration::from_millis(250));
    }
    42
}

#[ffi_export(node_js, executor = ::futures::executor::block_on)]
async fn long_running_fut (bytes: c_slice::Ref<'_, u8>)
  -> u8
{
    let wait_time = 300;
    #[cfg(target_arch = "wasm32")]
    let wait_time = 10 * wait_time;
    let arg = bytes[0];
    ffi_await!(async move {
        let _ = sleep(wait_time).await;
        42 + arg
    })
}

#[ffi_export(node_js)]
fn site_id (id: [u8; 8])
  -> char_p::Box
{
    char_p::new(format!("{:02x?}", id))
}

#[ffi_export(node_js)]
fn check_big_int_unsigned (
    value: u64,
    expected: char_p::Ref<'_>,
) -> u64
{
    assert_eq!(value.to_string(), expected.to_str());
    value
}

#[ffi_export(node_js)]
fn check_big_int_signed (
    value: i64,
    expected: char_p::Ref<'_>,
) -> i64
{
    assert_eq!(value.to_string(), expected.to_str());
    value
}

// ---

async
fn sleep (ms: u32)
{
    #[cfg(not(target_arch = "wasm32"))] {
        enum DropMsg {}
        let (tx, rx) = ::futures::channel::oneshot::channel::<DropMsg>();
        ::std::thread::spawn(move || {
            ::std::thread::sleep(
                ::std::time::Duration::from_millis(ms.into())
            );
            drop(tx);
        });
        match rx.await {
            | Ok(unreachable) => match unreachable {},
            | Err(_) => {},
        }
    }

    #[cfg(target_arch = "wasm32")] {
        use ::safer_ffi::node_js::__::*;

        #[wasm_bindgen(inline_js = r#"
            export function sleep(ms) {
                return new Promise(resolve => setTimeout(resolve, ms));
            }
        "#)]
        extern {
            fn sleep (ms: u32)
              -> js_sys::Promise
            ;
        }

        let _ = wasm_bindgen_futures::JsFuture::from(sleep(ms)).await;
    }
}
