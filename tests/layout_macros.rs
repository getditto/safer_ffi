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
use ::repr_c::{
    char_p::*,
    closure::*,
    ffi_export,
    layout::{
        CType,
        ReprC,
        derive_ReprC,
    },
    slice::*,
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
    field: slice_ref<'a, u32>,
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
        extern "C" fn(::repr_c::char_p::Ref_),
        Tuple2<
            [Foo<'static>; 12],
            Option<::repr_c::Box<Option<MyBool>>>
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
            fst: char_p_ref<'_>,
            snd: char_p_ref<'_>,
        ) -> char_p_boxed
        {
            format!("{}{}\0", fst.to_str(), snd.to_str())
                .try_into()
                .unwrap()
        }
    };
    unsafe {
        extern "C" {
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
    ints: slice_ref<'a, i32>
) -> Option<&'a i32>
{
    ints.as_slice().iter().max()
}

extern "C" {
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

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn test_max_invalid ()
{
    unsafe {
        ffi_max(i32_slice { ptr: 0 as _, len: 0 });
    }
}

#[ffi_export]
/// Returns an owned copy of the input array, with its elements sorted.
pub fn clone_sorted (
    ints: slice_ref<'_, i32>
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
            fst: char_p_ref<'_>,
            snd: char_p_ref<'_>,
            cb: RefDynFnMut1<(), char_p_raw>,
        )
        {
            let concat = &*format!("{}{}\0", fst.to_str(), snd.to_str());
            let char_p_concat: char_p_ref<'_> = concat.try_into().unwrap();
            {cb}.call(char_p_concat.into())
        }
    };
    unsafe {
        extern "C" {
            fn with_concat (
                fst: char_p_ref<'_>,
                snd: char_p_ref<'_>,
                cb: RefDynFnMut1<(), char_p_raw>,
            );
        }
        let mut called = false;
        with_concat(
            "Hello, \0".try_into().unwrap(),
            "World!\0".try_into().unwrap(),
            RefDynFnMut1::new(&mut |concat: char_p_raw| {
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
            MyBool_Layout(42)
        )
    );
    assert!(
        MyBool::is_valid(&
            MyBool_Layout(43)
        )
    );

    assert!(bool::not(
        MyBool::is_valid(&
            MyBool_Layout(44)
        )
    ));

    // Some(MyBool::True)
    assert!(
        <Option<MyBool>>::is_valid(&
            MyBool_Layout(42)
        )
    );
    // Some(MyBool::False)
    assert!(
        <Option<MyBool>>::is_valid(&
            MyBool_Layout(43)
        )
    );
    // None
    assert!(
        <Option<MyBool>>::is_valid(&
            MyBool_Layout(unsafe { ::core::mem::transmute(None::<MyBool>) })
        )
    );
}

#[cfg(feature = "headers")]
#[test]
fn generate_headers ()
  -> ::std::io::Result<()>
{Ok({
    let ref mut definer = MyDefiner {
        out: &mut ::std::io::stderr(),
        defines: Default::default(),
    };
    ::repr_c::inventory::iter
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .try_for_each(|::repr_c::FfiExport(define)| define(definer))
        ?
    ;

    // where
    struct MyDefiner<'out> {
        out: &'out mut dyn io::Write,
        defines: Set<String>,
    }
    impl ::repr_c::headers::Definer
        for MyDefiner<'_>
    {
        fn insert (self: &'_ mut Self, name: &'_ str)
          -> bool
        {
            self.defines
                .insert(name.to_owned())
        }

        fn out (self: &'_ mut Self)
          -> &'_ mut dyn io::Write
        {
            &mut self.out
        }
    }
})}
