//! Closures with a `#[repr(C)]` layout (inlined vtable),
//! up to 9 function arguments.
//!
//! Simplified for lighter documentation, but the actual `struct` definitions
//! and impls range up to `...DynFn...9`.
//!
//! ## Examples
//!
//! ### FFI-safe `Box<dyn FnMut()>`
//!
/*!  - ```rust
    use ::safer_ffi::prelude::*;

    let mut captured = String::from("…");
    let ffi_safe: repr_c::Box<dyn Send + FnMut()> =
        Box::new(move || {
            captured.push('!');
            println!("{}", captured);
        })
        .into()
    ;

    fn assert_ffi_safe (_: &impl ReprC)
    {}
    assert_ffi_safe(&ffi_safe);
    ``` */
//!
//! ### FFI-safe `Arc<dyn Fn()>`
//!
/*!  - ```rust
    use ::{
        safer_ffi::{
            prelude::*,
        },
        std::{
            sync::Arc,
        },
    };

    let captured = String::from("…");
    let ffi_safe: repr_c::Arc<dyn Send + Sync + Fn()> =
        Arc::new(move || {
            println!("{}", captured);
        })
        .into()
    ;

    fn assert_ffi_safe (_: &impl ReprC)
    {}
    assert_ffi_safe(&ffi_safe);
    ``` */
//!

cfg_alloc! {
    pub mod arc;
    pub mod boxed;
    #[doc(no_inline)]
    pub use self::{
        arc::{ArcDynFn0, ArcDynFn1},
        boxed::{BoxDynFnMut0, BoxDynFnMut1},
    };
    #[cfg(not(docs))]
    #[doc(no_inline)]
    pub use self::{
        arc::{
            ArcDynFn2, ArcDynFn3, ArcDynFn4, ArcDynFn5,
            ArcDynFn6, ArcDynFn7, ArcDynFn8, ArcDynFn9,
        },
        boxed::{
            BoxDynFnMut2, BoxDynFnMut3, BoxDynFnMut4, BoxDynFnMut5,
            BoxDynFnMut6, BoxDynFnMut7, BoxDynFnMut8, BoxDynFnMut9,
        },
    };
}

pub mod borrowed;

#[doc(no_inline)]
pub use borrowed::RefDynFnMut0;
#[doc(no_inline)]
pub use borrowed::RefDynFnMut1;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut2;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut3;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut4;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut5;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut6;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut7;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut8;
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::RefDynFnMut9;
