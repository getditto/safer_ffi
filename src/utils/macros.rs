macro_rules! use_prelude { () => (
    #[allow(unused_imports)]
    use crate::utils::prelude::*;
)}

#[cfg(
    any(docs, feature = "alloc")
)]
macro_rules! cfg_alloc {(
    $($item:item)*
) => (
    $(
        #[cfg_attr(docs, doc(cfg(feature = "alloc")))]
        $item
    )*
)}

#[cfg(not(
    any(docs, feature = "alloc")
))]
macro_rules! cfg_alloc {(
    $($item:item)*
) => (
    // Nothing
)}

