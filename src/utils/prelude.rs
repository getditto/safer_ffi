#![allow(unused_imports)]

pub(in crate) use crate::{
    layout::macros::*,
    c_char,
    layout::*,
    tuple::*,
    utils::markers::*,
};

pub(in crate) use ::core::{
    convert::{TryFrom, TryInto},
    ffi::c_void,
    fmt,
    marker::PhantomData,
    mem,
    ops::{
        Deref, DerefMut,
        Not as _,
    },
};

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

pub(in crate)
mod rust {
    #[apply(cfg_alloc)]
    pub(in crate) use ::alloc::{
        boxed::Box,
        string::String,
        vec::Vec,
    };
}

pub(in crate)
mod ptr {
    pub(in crate) use ::core::ptr::*;
    pub(in crate) use crate::ptr::*;
}

#[apply(cfg_std)]
pub(in crate) use ::std::{
    io,
};

pub(in crate)
use crate::prelude::*;
