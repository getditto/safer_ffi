#![allow(unused_imports)]

pub(in crate) use crate::{
    __cfg_headers__,
    c_char,
    layout::*,
    tuple::*,
    utils::markers::*,
};
cfg_alloc! {
    pub(in crate) use crate::{
        Box,
        String,
        Vec,
    };
}
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
#[cfg(not(target_arch = "wasm32"))]
pub(in crate) use ::libc::size_t;

#[cfg(target_arch = "wasm32")]
#[allow(bad_style)]
pub(in crate) type size_t = u32;

pub(in crate)
mod rust {
    cfg_alloc! {
        pub(in crate) use ::alloc::{
            boxed::Box,
            string::String,
            vec::Vec,
        };
    }
}

pub(in crate)
mod ptr {
    pub(in crate) use ::core::ptr::*;
    pub(in crate) use crate::ptr::*;
}

cfg_std! {
    pub(in crate) use ::std::{
        io,
    };
}

pub(in crate)
use crate::prelude::*;
