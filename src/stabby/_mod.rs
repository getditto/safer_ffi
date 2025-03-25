use ::core::ops::Not;
use ::stabby::IStable;

use crate::layout::__HasNiche__;
use crate::layout::CType;
use crate::layout::ReprC;

/// Coherence wrapper for a blanket [`ReprC`] implementation off
/// a [`stabby::IStable`] one, without running into overlapping impls.
///
/// # A note about SemVer and stability
///
/// There is a blanket impl of [`ReprC`] for this type, bounded by
/// [`::stabby::IStable`]. This, technically, is a public dependency on
/// `::stabby`, whose major versions could then be technically perceived as
/// breaking for this crate too.
///
/// This crate takes the opinionated stance not to be deterred from publicly
/// exposing this wrapper nonetheless, trusting the users to know what they're
/// doing.
///
/// If you are unsure, **do not use this wrapper**.
///
/// [`::safer-ffi`][crate] waves any responsibility w.r.t. SemVer breakage that
/// may otherwise ensue.
///
/// [`ReprC`]: trait@ReprC
#[stabby::stabby]
pub struct Stabbied<T>(pub T);

unsafe impl<T: IStable> ReprC for Stabbied<T>
where
    T::CType: CType,
{
    type CLayout = T::CType;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        unsafe { T::is_invalid(it as *const _ as *const u8) }.not()
    }
}

unsafe impl<T> __HasNiche__ for Stabbied<T>
where
    T: IStable<HasExactlyOneNiche = ::stabby::abi::B1>,
    Self: ReprC,
{
}

mod boxed_impl;
mod fatptr_impl;
mod sync_impl;
