#![cfg_attr(rustfmt, rustfmt::skip)]
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
#[ffi_export(node_js)]
fn free_char_p (_string: Option<char_p::Box>)
{}

/// Same as `concat`, but with a callback-based API to auto-free the created
/// string.
#[ffi_export]
fn with_concat (
    fst: char_p::Ref<'_>,
    snd: char_p::Ref<'_>,
    /*mut*/ cb: RefDynFnMut1<'_, (), char_p::Raw>,
)
{
    let mut cb = cb;
    let s = concat(fst, snd);
    cb.call(s.as_ref().into());
}

/// Returns a pointer to the maximum integer of the input slice, or `NULL` if
/// it is empty.
#[ffi_export]
fn max<'a> (
    xs: c_slice::Ref<'a, i32>,
) -> Option<&'a i32>
{
    xs  .as_slice()
        .iter()
        .max()
}

mod foo {
    use super::*;

    #[derive_ReprC]
    #[ReprC::opaque("foo")]
    pub
    struct Foo_<Generic> { hidden: Generic }

    type Foo = Foo_<i32>;

    #[ffi_export]
    fn read_foo (foo: &'_ Foo) -> i32
    {
        foo.hidden
    }

    #[ffi_export]
    fn new_foo () -> repr_c::Box<Foo>
    {
        repr_c::Box::new(Foo { hidden: 42 })
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
    fn with_foo (cb: with_ref_cb<Foo>)
    {
        cb(&mut Foo { hidden: 42 });
    }
}

mod bar {
    use super::*;
    #[derive_ReprC]
    #[repr(u8)]
    pub
    enum Bar { A }

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

#[safer_ffi::cfg_headers]
#[test]
fn generate_headers ()
  -> ::std::io::Result<()>
{Ok({
    use ::safer_ffi::headers::Language::*;
    for &(language, ext) in &[(C, "h"), (CSharp, "cs")] {
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
})}

#[ffi_export(async_executor = futures::executor::block_on)]
fn async_get_ft ()
  -> i32
{
    ffi_await!(async { 42 })
}

// #[ffi_export]
// pub const FOO: i32 = 42;

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
