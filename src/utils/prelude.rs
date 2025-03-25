#![allow(unused_imports)]

pub(crate) use ::core::convert::TryFrom;
pub(crate) use ::core::convert::TryInto;
pub(crate) use ::core::ffi::c_void;
pub(crate) use ::core::fmt;
pub(crate) use ::core::marker::PhantomData;
pub(crate) use ::core::mem;
pub(crate) use ::core::ops::Deref;
pub(crate) use ::core::ops::DerefMut;
pub(crate) use ::core::ops::Not as _;

pub(crate) use crate::c_char;
pub(crate) use crate::layout::macros::*;
pub(crate) use crate::layout::*;
pub(crate) use crate::tuple::*;
pub(crate) use crate::utils::markers::*;

match_cfg! {
    target_arch = "wasm32" => {
        #[allow(bad_style, dead_code)]
        pub(in crate) type size_t = u32;
    },
    _ => {
        pub(in crate) use crate::libc::size_t;
    },
}

cfg_alloc! {
    pub(in crate) use ::alloc::{
        borrow::ToOwned,
        string::ToString,
        vec,
    };
}

pub(crate) mod rust {
    #[apply(cfg_alloc)]
    pub(crate) use ::alloc::boxed::Box;
    #[apply(cfg_alloc)]
    pub(crate) use ::alloc::string::String;
    #[apply(cfg_alloc)]
    pub(crate) use ::alloc::vec::Vec;
}

pub(crate) mod ptr {
    pub(crate) use ::core::ptr::*;

    pub(crate) use crate::ptr::*;
}

#[apply(cfg_std)]
pub(crate) use ::std::io;

pub(crate) use crate::prelude::*;
