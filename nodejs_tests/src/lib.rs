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
    cb: unsafe extern "C" fn (data: *mut ::std::os::raw::c_void, x: i32),
)
{
    cb(data, 42)
}

#[cfg(feature = "nodejs")]
const _: () = {
    use ::safer_ffi::node_js as napi;

    #[::safer_ffi::node_js::js_function(1)]
    fn call_with_42_js (ctx: ::safer_ffi::node_js::CallContext<'_>)
      -> ::safer_ffi::node_js::Result<::safer_ffi::node_js::JsUndefined>
    {
        let cb: ::safer_ffi::node_js::Closure<fn(i32)> =
            ::safer_ffi::node_js::extract_arg(&ctx, 0)?
        ;
        let data = &cb as *const _ as *mut _;
        let trampoline
            : (
                unsafe extern "C"
                fn (data: *mut ::std::os::raw::c_void, x: i32) -> _
            ) = {
                ::safer_ffi::node_js::Closure::<fn(i32)>::cb
            }
        ;
        unsafe {
            call_with_42(data, ::core::mem::transmute(trampoline))
        };
        ctx.env.get_undefined()
    }

    ::safer_ffi::node_js::registering::submit! {
        #![crate = ::safer_ffi::node_js::registering]

        ::safer_ffi::node_js::registering::NapiRegistryEntry::NamedMethod {
            name: "call_with_42",
            method: call_with_42_js,
        }
    }

    // #[ffi_export(node_js)]
    // fn call_with_42 (
    //     cb: RefMutDynFnMut<'_, c_int>,
    // )
    //     // data: *mut c_void,
    //     // cb: Option<
    //     //     unsafe extern "C"
    //     //     fn (data: *mut c_void, arg: c_int),
    //     // >,
    // {
    //     let mut cb = cb;
    //     cb.call(42);
    //     // if let Some(cb) = cb {
    //     //     unsafe {
    //     //         cb(data, 42);
    //     //     }
    //     // }
    // }

    // #[js_function(1)]
    // fn call_with_42 (ctx: CallContext<'_>)
    // {
    //     let cb: JsFunction = ctx.get(0)?;
    //     call_with_42_real(
    //         <*const _>::cast::<c_void>(&cb),
    //         Some({
    //             unsafe extern "C"
    //             fn cb (void * data, arg: c_int)
    //             {
    //                 let data: *const JsFunction = data.cast();
    //                 let cb: &JsFunction = data.as_ref().unwrap();
    //                 data.call(None, &[
    //                     arg.into_repr_napi(…)
    //                 ])
    //             }
    //             cb
    //         }),
    //     )
    //     fn trampolone

    // }
};
