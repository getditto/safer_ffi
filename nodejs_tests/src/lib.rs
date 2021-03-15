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

#[ffi_export(node_js)]
fn set_bool (b: Out<'_, bool>)
{
    b.write(true);
}
