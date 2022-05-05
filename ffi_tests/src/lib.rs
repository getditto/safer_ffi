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
    format!("{}{}\0", fst.to_str(), snd.to_str())
        .try_into()
        .unwrap()
}

/// Frees a string created by `concat`.
#[ffi_export]
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
    let concat = concat(fst, snd);
    cb.call(concat.as_ref().into());
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
    struct Foo { hidden: i32 }

    #[ffi_export]
    fn new_foo () -> repr_c::Box<Foo>
    {
        repr_c::Box::new(Foo { hidden: 42 })
    }

    #[ffi_export]
    fn read_foo (foo: &'_ Foo) -> i32
    {
        foo.hidden
    }

    #[ffi_export]
    fn free_foo (foo: Option<repr_c::Box<Foo>>)
    {
        drop(foo)
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

#[ffi_export]
#[derive_ReprC]
#[repr(u8)]
pub enum Wow {
    Leroy,
    Jenkins,
}

#[ffi_export]
#[derive_ReprC]
#[repr(u8)]
pub enum Triforce {
    Din = 3,
    Farore = 1,
    Naryu,
}

#[safer_ffi::cfg_headers]
#[test]
fn generate_headers ()
  -> ::std::io::Result<()>
{
    let builder = ::safer_ffi::headers::builder();
    if ::std::env::var("HEADERS_TO_STDOUT").ok().map_or(false, |it| it == "1") {
        builder
            .to_writer(::std::io::stdout())
            .generate()
    } else {
        builder
            .to_file(&"generated.h".to_string())?
            .generate()
    }
}
