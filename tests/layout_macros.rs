#![feature(hash_set_entry)]
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

#[test]
fn validity ()
{
    // `Foo_Layout` is `<Foo as ReprC>::CLayout`
    assert!(
        Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: slice_ptr_Layout { ptr: 4 as _, len: 0 } }
        )
    );
    assert!(
        Foo::is_valid(&
            Foo_Layout { b: 43_u8.into(), field: slice_ptr_Layout { ptr: 4 as _, len: 0 } }
        )
    );

    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 0.into(), field: slice_ptr_Layout { ptr: 4 as _, len: 0 } }
        ))
    );
    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: slice_ptr_Layout { ptr: 0 as _, len: 0 } }
        ))
    );
    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: slice_ptr_Layout { ptr: 3 as _, len: 0 } }
        ))
    );
}

#[derive_ReprC]
#[repr(C)]
pub
struct Crazy {
    a: extern "C" fn (extern "C" fn(::repr_c::char_p::Ref_), Tuple2<[Foo<'static>; 12], ::repr_c::Box<MyBool>>),
    closure: RefDynFnMut2<'static, (), i32, usize>,
}

/// Concatenate two strings
#[ffi_export]
fn concat (
    fst: char_p_ref<'_>,
    snd: char_p_ref<'_>,
) -> char_p_boxed
{
    format!("{}{}\0", fst.to_str(), snd.to_str())
        .try_into()
        .unwrap()
}

/// Some docstring
#[ffi_export]
pub fn max (
    ints: slice_ref<'_, i32>
) -> Option<&'_ i32>
{
    ints.as_slice().iter().max()
}

/// Returns a owned copy of the input array, with its elements sorted.
#[ffi_export]
pub fn clone_sorted (
    ints: slice_ref<'_, i32>
) -> repr_c::Vec<i32>
{
    let mut ints = ints.as_slice().to_vec();
    ints.sort_unstable();
    ints.into()
}

/// Frees the input `Vec`.
#[ffi_export]
pub fn free_vec (
    _: repr_c::Vec<i32>,
)
{}

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
        .try_for_each(|::repr_c::TypeDef(define)| define(definer))
        ?
    ;

    // where
    struct MyDefiner<'out> {
        out: &'out mut dyn io::Write,
        defines: Set<String>,
    }
    impl ::repr_c::layout::Definer
        for MyDefiner<'_>
    {
        fn insert (self: &'_ mut Self, name: &'_ str)
          -> bool
        {
            let mut inserted = false;
            self.defines.get_or_insert_with(name, |name| {
                inserted = true;
                name.to_owned()
            });
            inserted
        }

        fn out (self: &'_ mut Self)
          -> &'_ mut dyn io::Write
        {
            &mut self.out
        }
    }
})}
