#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(unused)]

use ::safer_ffi::prelude::*;

/// Concatenate the two input strings into a new one.
///
/// The returned string must be freed using `free_char_p`.
#[ffi_export]
fn concat (
    fst: char_p::Ref<'_>,
    snd: char_p::Ref<'_>,
) -> char_p::Box
{
    format!("{}{}", fst.to_str(), snd.to_str())
        .try_into()
        .unwrap()
}

/// Frees a string created by `concat`.
#[ffi_export(js)]
fn free_char_p (_string: Option<char_p::Box>)
{}

#[ffi_export]
fn returns_a_fn_ptr ()
  -> extern "C" fn(u8) -> u16
{
    extern "C"
    fn f (n: u8)
      -> u16
    {
        (n as u16) << 8
    }

    f
}

#[ffi_export]
fn call_in_the_background (
    f: repr_c::Arc<dyn Send + Sync + Fn()>,
)
{
    let f2 = f.clone();
    ::std::thread::spawn(move || {
        f2.call()
    });
    drop(f);
}

/// https://github.com/getditto/safer_ffi/issues/45
#[ffi_export]
fn _issue_45<'a, 'b> (_: i32)
  -> i32
where
    'a : 'b,
{
    unimplemented!();
}

/// Same as `concat`, but with a callback-based API to auto-free the created
/// string.
#[ffi_export]
fn with_concat (
    fst: char_p::Ref<'_>,
    snd: char_p::Ref<'_>,
    mut cb: ::safer_ffi::closure::RefDynFnMut1<'_, (), char_p::Raw>,
)
{
    let concat = concat(fst, snd);
    cb.call(concat.as_ref().into());
}

/// Returns a pointer to the maximum integer of the input slice, or `NULL` if
/// it is empty.
#[ffi_export(rename = "max")]
fn max_but_with_a_weird_rust_name<'a> (
    xs: c_slice::Ref<'a, i32>,
) -> Option<&'a i32>
{
    xs  .as_slice()
        .iter()
        .max()
}

mod foo {
    use super::*;

    #[derive_ReprC(rename = "foo")]
    #[repr(opaque)]
    pub
    struct Foo_<Generic> {
        hidden: Generic,
    }

    type Foo = Foo_<i32>;

    #[ffi_export]
    fn read_foo (foo: &'_ Foo) -> i32
    {
        foo.hidden
    }

    #[ffi_export]
    fn new_foo () -> repr_c::Box<Foo>
    {
        Box::new(Foo { hidden: 42 })
            .into()
    }

    #[ffi_export]
    fn free_foo (foo: Option<repr_c::Box<Foo>>)
    {
        drop(foo)
    }

    #[derive_ReprC]
    #[repr(transparent)]
    pub
    struct with_ref_cb<Arg : 'static + ReprC> /* = */ (
        pub
        extern "C" fn(&mut Arg)
    );

    #[ffi_export]
    fn with_foo (cb: with_ref_cb<Foo>) -> bool
    {
        cb(&mut Foo { hidden: 42 });
        true
    }
}

mod bar {
    use super::*;

    #[derive_ReprC]
    #[repr(i8)]
    pub
    enum Bar {
        A = 43,
        B = (Bar::A as i8 - 1),
    }

    #[ffi_export]
    fn check_bar (_bar: Bar)
    {}
}

#[allow(nonstandard_style)]
mod baz {
    use super::*;

    /// This is a `#[repr(C)]` enum, which leads to a classic enum def.
    #[derive_ReprC]
    #[repr(C)]
    pub
    enum SomeReprCEnum {
        /// This is some variant.
        SomeVariant,
    }

    #[ffi_export]
    fn check_SomeReprCEnum (_baz: SomeReprCEnum)
    {}
}

#[ffi_export]
#[derive_ReprC]
#[repr(u8)]
pub enum Wow {
    Leroy,
    Jenkins,
}

/// Hello, `World`!
#[ffi_export]
#[derive_ReprC(rename = "triforce")]
#[repr(u8)]
pub enum Triforce {
    Din = 3,
    Farore = Triforce::Din as u8 - 2,
    Naryu,
}

#[derive_ReprC]
#[repr(transparent)]
pub struct MyPtr {
    foo: ::core::ptr::NonNull<()>,
    bar: (),
}

#[derive_ReprC(rename = "my_renamed_ptr")]
#[repr(transparent)]
pub struct MyRenamedPtr {
    foo: ::core::ptr::NonNull<()>,
    bar: (),
}

#[ffi_export]
fn my_renamed_ptr_api() -> MyRenamedPtr {
    MyRenamedPtr {
        foo: ::core::ptr::NonNull::new(0xbad000 as _).unwrap(),
        bar: ()
    }
}

macro_rules! docs {() => (
    "Hello, `World`!"
)}

#[ffi_export]
#[doc = docs!()]
#[derive_ReprC(rename = "next_generation")]
#[repr(C)]
pub struct Next {
    /// I test some `gen`-eration.
    gen: bar::Bar,
    /// with function pointers and everything!
    cb: extern "C" fn(bool) -> Option<MyPtr>,
}

#[ffi_export]
#[derive_ReprC]
#[repr(C)]
pub struct AnUnusedStruct {
    are_you_still_there: Wow,
}

#[ffi_export]
#[derive_ReprC]
#[repr(C)]
pub struct ArraysStruct {
    floats: [f32; 3],
    sizes: [u64; 5],
    dim_2: [[u8; 1]; 2],
    dim_3: [[[u8; 1]; 2]; 3],
}

#[derive_ReprC]
#[repr(C)]
pub struct ConstGenericStruct<const M: usize, T: ReprC> {
    data: [T; M]
}

#[ffi_export]
#[derive_ReprC]
#[repr(C)]
pub struct SpecificConstGenericContainer {
    field1: ConstGenericStruct<1, u8>,
    field2: ConstGenericStruct<2, u8>,
    field3: ConstGenericStruct<3, u16>,
}

#[safer_ffi::cfg_headers]
#[test]
fn generate_headers ()
  -> ::std::io::Result<()>
{
    use ::safer_ffi::headers::Language::*;

    let languages = &[
        (C, "h"),
        (CSharp, "cs"),
        (Lua, "lua"),
        (Python, "cffi"),
    ];

    for &(language, ext) in languages {
        let builder =
            ::safer_ffi::headers::builder()
                .with_language(language)
        ;
        if  ::std::env::var("HEADERS_TO_STDOUT")
                .ok()
                .map_or(false, |it| it == "1")
        {
            builder
                .to_writer(::std::io::stdout())
                .generate()?
        } else {
            builder
                .to_file(&format!("generated.{}", ext))?
                .generate()?
        }
    }
    Ok(())
}

#[ffi_export(executor = futures::executor::block_on)]
async fn async_get_ft ()
  -> i32
{
    ffi_await!(async { 42 })
}

#[ffi_export]
pub const FOO: i32 = 42;

#[ffi_export(untyped)]
pub const SOME_NAME: &str = "hello there";

#[ffi_export]
fn _some_opaque_std_lib_type() -> repr_c::Box<String> {
    Box::new(String::from("…")).into()
}

#[derive_ReprC]
#[repr(opaque)]
enum Enum { A(String) }

#[ffi_export]
fn _my_enum_is_opaque() -> repr_c::Box<Enum> {
    Box::new(Enum::A("…".into())).into()
}

mod futures {
    pub
    mod executor {
        use ::std::{
            future::Future,
            pin::Pin,
            sync::Arc,
            task::{Context, Poll, Wake},
            thread::{self, Thread},
        };

        struct ThreadUnparker /* = */ (Thread);
        impl Wake for ThreadUnparker {
            fn wake (self: Arc<Self>) { self.0.unpark(); }
        }

        pub
        fn block_on<T> (ref mut fut: impl Future<Output = T>)
          -> T
        {
            let ref mut fut = unsafe { Pin::new_unchecked(fut) };
            let ref waker = Arc::new(ThreadUnparker(thread::current())).into();
            let ref mut cx = Context::from_waker(waker);
            loop {
                match fut.as_mut().poll(cx) {
                    Poll::Ready(res) => break res,
                    Poll::Pending => thread::park(),
                }
            }
        }
    }
}

mod executors {
    use {
        ::safer_ffi::{
            futures::FfiFutureExecutor,
            prelude::*,
        },
    };

    ::safer_ffi::ffi_export_future_helpers!();

    #[ffi_export]
    fn test_spawner (
        executor: VirtualPtr<dyn 'static + FfiFutureExecutor>,
    ) -> i32
    {
        let x: i32 = executor.block_on(async {
            let x: i32 =
                executor.spawn(async {
                    async {}.await;
                    42
                })
                .await
            ;
            x
        });
        x
    }
}
