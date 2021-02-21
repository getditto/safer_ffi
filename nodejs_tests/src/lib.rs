use ::safer_ffi::prelude::*;

::safer_ffi::node_js::register_exported_functions!();

#[ffi_export(node_js)]
fn add (x: i32, y: i32)
  -> i32
{
    i32::wrapping_add(x, y)
}

#[ffi_export(node_js)]
fn sub (x: u8, y: u8)
  -> u8
{
    u8::wrapping_sub(x, y)
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
fn get_hello() -> char_p::Box
{
    char_p::new("Hello, World!")
}
