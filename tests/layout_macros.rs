#![allow(clippy::all)]
#![allow(unused_imports)]

#[macro_use]
extern crate macro_rules_attribute;

use ::std::{
    collections::HashSet as Set,
    convert::TryInto,
    io,
    ptr,
    ops::Not as _,
};
use ::safer_ffi::{
    closure::*,
    prelude::*,
    layout::{
        CType,
        ReprC,
        derive_ReprC,
    },
    tuple::Tuple2,
};

#[derive_ReprC]
#[repr(C)]
pub
struct Tuple1<T> {
    _0: T,
}

#[derive_ReprC]
#[repr(u8)]
#[derive(Debug)]
/// Some docstring
pub
enum MyBool {
    False = 42,
    True, // = 43
}

#[derive_ReprC]
#[repr(C)]
/// Some docstring
pub
struct Foo<'a> {
    b: MyBool,
    field: c_slice::Ref<'a, u32>,
}

#[repr(C)]
struct i32_slice {
    ptr: *const i32,
    len: usize,
}

#[test]
fn validity ()
{ unsafe {
    // `Foo_Layout` is `<Foo as ReprC>::CLayout`
    assert!(
        Foo::is_valid(&
            ::core::mem::transmute([
                42_u8,
                    /*pad*/ 0,0,0,0,0,0,0,
                0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        )
    );
    assert!(
        Foo::is_valid(&
            ::core::mem::transmute([
                43_u8,
                    /*pad*/ 0,0,0,0,0,0,0,
                0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        )
    );

    assert!(
        bool::not(Foo::is_valid(&
            ::core::mem::transmute([
                0_u8,
                    /*pad*/ 0,0,0,0,0,0,0,
                0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        ))
    );
    assert!(
        bool::not(Foo::is_valid(&
            ::core::mem::transmute([
                42_u8,
                    /*pad*/ 0,0,0,0,0,0,0,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        ))
    );
    assert!(
        bool::not(Foo::is_valid(&
            ::core::mem::transmute([
                42_u8,
                    /*pad*/ 0,0,0,0,0,0,0,
                0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
        ))
    );
}}

#[derive_ReprC]
#[repr(C)]
pub
struct Crazy {
    a: extern "C" fn (
        extern "C" fn(char_p::Raw) -> bool,
        Tuple2<
            [Foo<'static>; 12],
            Option<repr_c::Box<MyBool>>
        >,
    ),
    closure: RefDynFnMut2<'static, (), i32, usize>,
}

#[test]
fn test_concat ()
{
    use ::std::{os::raw::c_char, slice};

    let () = {
        #[ffi_export]
        /// Concatenate two strings
        fn concat (
            fst: char_p::Ref<'_>,
            snd: char_p::Ref<'_>,
        ) -> char_p::Box
        {
            format!("{}{}\0", fst.to_str(), snd.to_str())
                .try_into()
                .unwrap()
        }
    };
    unsafe {
        unsafe extern "C" {
            fn concat (
                fst: *const c_char,
                snd: *const c_char,
            ) -> *mut c_char;
        }
        let it = concat(
            b"Hello, \0".as_ptr().cast(),
            b"World!\0".as_ptr().cast(),
        );
        let bytes = ::std::ffi::CStr::from_ptr(it).to_bytes_with_nul();
        assert_eq!(
            bytes,
            b"Hello, World!\0",
        );
        let len = bytes.len();
        drop::<Box<[u8]>>(Box::from_raw(slice::from_raw_parts_mut(
            it.cast(),
            len,
        )));
    }
}

#[ffi_export]
/// Some docstring
pub fn max<'a> (
    ints: c_slice::Ref<'a, i32>
) -> Option<&'a i32>
{
    ints.as_slice().iter().max()
}

unsafe extern "C" {
    #[link_name = "max"]
    fn ffi_max (
        ints: i32_slice,
    ) -> *const i32;
}

#[test]
fn test_max ()
{
    unsafe {
        let empty = i32_slice { ptr: 4 as _, len: 0 };
        assert!(ffi_max(empty).is_null());
        let xs = &[-8, -2, -4][..];
        assert_eq!(
            max(xs.into()),
            ffi_max(i32_slice { ptr: xs.as_ptr(), len: xs.len() }).as_ref(),
        );
    }
}

// #[cfg(debug_assertions)]
// #[test]
// #[should_panic] /* Currently abort guard prevents it */
// fn test_max_invalid ()
// {
//     unsafe {
//         ffi_max(i32_slice { ptr: 0 as _, len: 0 });
//     }
// }

#[ffi_export]
/// Returns an owned copy of the input array, with its elements sorted.
pub fn clone_sorted (
    ints: c_slice::Ref<'_, i32>
) -> repr_c::Vec<i32>
{
    let mut ints = ints.as_slice().to_vec();
    ints.sort_unstable();
    ints.into()
}

#[ffi_export]
/// Frees the input `Vec`.
pub fn free_vec (
    _vec: repr_c::Vec<i32>,
)
{}

#[test]
fn test_with_concat ()
{
    use ::std::sync::Arc;
    let () = {
        #[ffi_export]
        fn with_concat (
            fst: char_p::Ref<'_>,
            snd: char_p::Ref<'_>,
            cb: RefDynFnMut1<'_, (), char_p::Raw>,
        )
        {
            let concat = &*format!("{}{}\0", fst.to_str(), snd.to_str());
            let char_p_concat: char_p::Ref<'_> = concat.try_into().unwrap();
            {cb}.call(char_p_concat.into())
        }
    };
    unsafe {
        unsafe extern "C" {
            fn with_concat (
                fst: char_p::Ref<'_>,
                snd: char_p::Ref<'_>,
                cb: RefDynFnMut1<(), char_p::Raw>,
            );
        }
        let mut called = false;
        with_concat(
            "Hello, \0".try_into().unwrap(),
            "World!\0".try_into().unwrap(),
            RefDynFnMut1::new(&mut |concat: char_p::Raw| {
                called = true;
                assert_eq!(
                    concat.as_ref().to_str(),
                    "Hello, World!",
                );
            }),
        );
        assert!(called);
    }
}

#[test]
fn test_niche ()
{
    #[derive_ReprC]
    #[repr(i8)]
    enum MyBool {
        True = 42,
        False = 43,
    }

    assert!(
        MyBool::is_valid(&
            MyBool_Layout {
                discriminant: 42,
            }
        )
    );
    assert!(
        MyBool::is_valid(&
            MyBool_Layout {
                discriminant: 43,
            }
        )
    );

    assert!(bool::not(
        MyBool::is_valid(&
            MyBool_Layout {
                discriminant: 44,
            }
        )
    ));
}

#[test]
fn test_c_str_macro ()
{
    let mut it: char_p::Ref<'static> = c!();
    assert_eq!(it.to_str(), "");
    it = c!("Hello, World!");
    assert_eq!(it.to_str(), "Hello, World!");
}

#[cfg(feature = "headers")]
#[test]
fn generate_headers ()
  -> ::std::io::Result<()>
{Ok({
    use ::safer_ffi::headers::Language::*;
    let mut out = vec![];
    for &language
        in  &[
                C,
                CSharp,
                Lua,
            ]
    {
        ::safer_ffi::headers::builder()
            .with_language(language)
            .to_writer(&mut out)
            .generate()?
    }
    eprintln!("{}", String::from_utf8_lossy(&out));
})}
