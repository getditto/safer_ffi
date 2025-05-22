#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
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
pub struct SomeStruct {
    a: Option<repr_c::Box<[u8]>>,
}

#[derive_ReprC]
#[repr(opaque)]
pub struct SomeOpaqueStruct {
    _a: Option<repr_c::Box<[u8]>>,
}

#[derive_ReprC(rename = "dittoffi_result")]
#[repr(C, js)]
pub struct FfiResult<Ok: ReprC> {
    /// Non-`NULL` pointer to opaque object on error, `NULL` otherwise.
    pub error: Option<repr_c::Box<SomeStruct>>,

    /// When no error occurred, the success value payload can be retrieved here.
    ///
    /// Otherwise, the value is to be ignored.
    pub success: Option<repr_c::Box<SomeStruct>>,
    foo: Ok::CLayout,
}

#[derive_ReprC(rename = "RenamedStruct")]
#[repr(transparent)]
pub struct TransparentStruct {
    pub i: BasicEnum
}

#[ffi_export]
/// Some comment
/// Some comment 2
/// Some comment 3
pub unsafe fn free_vec (
    _optional_parameter: Option<repr_c::Box<i32>>,
    _required_parameter: repr_c::Box<i32>,
    _foo: FfiResult<repr_c::Box<i32>>,
    // _optional_foo: FfiResult<Option<repr_c::Box<i32>>>,
    _bar: repr_c::Box<SomeOpaqueStruct>,
    _be: BasicEnum,
    _ed: EnumWithExplicitDiscriminant,
    _static_array: [u64; 16],
    _dynamic_array: repr_c::Box<[u64]>,
    _vector: repr_c::Vec<u64>,
    _transparent: TransparentStruct,
    _string: char_p::Ref<'_>,
    _raw_pointer: *mut i32,
) {
}

#[ffi_export]
/// Some comment
/// Some comment 2
/// Some comment 3
pub const SOME_CONSTANT: u32 = 4 * 1024;

#[ffi_export(untyped)]
pub const SOME_STRING: &str = "SOME_STRING";

#[ffi_export(untyped)]
pub const SOME_INT: u32 = 1;

#[ffi_export(untyped)]
pub const SOME_DOUBLE: f64 = 1.0;

#[derive_ReprC]
#[repr(u8)]
/// Some comment
/// Some comment 2
/// Some comment 3
pub enum BasicEnum {
    True,
    False,
}

#[derive_ReprC]
#[repr(C)]
pub enum EnumWithExplicitDiscriminant {
    // Some comment
    // Some comment 2
    True = 42,
    False = 43,
}

#[cfg(feature = "headers")]
#[test]
fn test_kotlin () -> io::Result<()> {Ok({
    use ::safer_ffi::headers::Language::*;

    safer_ffi::headers::builder()
        .with_language(Metadata)
        .to_writer(&mut io::stderr())
        .generate()?
})}
