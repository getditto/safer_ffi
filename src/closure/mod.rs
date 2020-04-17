//! Closures with a `#[repr(C)]` layout (inlined vtable),
//! up to 9 function arguments.
//!
//! Simplified for lighter documentation, but the actual `struct` definitions
//! and impls range up to `...DynFn...9`.

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
pub use borrowed::{RefDynFnMut0, RefDynFnMut1};
#[cfg(not(docs))]
#[doc(no_inline)]
pub use borrowed::{
    RefDynFnMut2, RefDynFnMut3, RefDynFnMut4, RefDynFnMut5,
    RefDynFnMut6, RefDynFnMut7, RefDynFnMut8, RefDynFnMut9,
};
