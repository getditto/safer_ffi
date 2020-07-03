use ::safer_ffi::prelude::*;

#[derive_ReprC]
#[repr(C)]
pub
struct Point {
    x: f32,
    y: f32,
}

#[ffi_export]
fn add (x: i32, y: i32)
  -> i32
{
    x.wrapping_add(y)
}

#[ffi_export]
fn new_Point (p: Out<'_, Point>)
  -> bool
{
    dbg!();
    p.write(Point {
        x: 42., y: 27.,
    });
    true
}

#[ffi_export]
fn concat (
    s1: char_p::Ref<'_>,
    s2: char_p::Ref<'_>,
) -> Option<char_p::Box>
{
    format!("{}{}\0", s1.to_str(), s2.to_str())
        .try_into()
        .ok()
}

#[ffi_export]
unsafe
fn with_concat (
    s1: char_p::Ref<'_>,
    s2: char_p::Ref<'_>,
    cb: RefDynFnMut1<'_, (), Option<char_p::Raw>>,
)
{
    let concat = concat(s1, s2);
    let () = {cb}.call(concat.as_ref().map(|it| it.as_ref().into()));
    drop(concat);
}

#[ffi_export]
fn free_string (string: char_p::Box)
{
    drop(string);
}

#[ffi_export]
fn __safer_ffi_helper_free_string (string: Option<char_p::Box>)
{
    drop(string);
}

#[ffi_export]
fn names () -> Option<repr_c::Vec<repr_c::String>>
{
    Some(::safer_ffi::c_vec![
        "Hello,".to_string().into(),
        "World!".to_string().into(),
    ])
}

#[ffi_export]
fn free_names (names: Option<repr_c::Vec<repr_c::String>>)
{
    drop(names);
}
