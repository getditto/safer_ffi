macro_rules! use_prelude { () => (
    #[allow(unused_imports)]
    use crate::utils::prelude::*;
)}

#[cfg(
    any(all(docs, feature = "nightly"), feature = "alloc")
)]
macro_rules! cfg_alloc {(
    $($item:item)*
) => (
    $(
        #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
        $item
    )*
)}

#[cfg(not(
    any(all(docs, feature = "nightly"), feature = "alloc")
))]
macro_rules! cfg_alloc {(
    $($item:item)*
) => (
    // Nothing
)}

macro_rules! const_assert {
    (
        for [$($generics:tt)*]
        [$($($pre:tt)+)?] => [$($post:tt)*]
    ) => (
        const _: () = {
            fn check<$($generics)*> ()
            where
                $($($pre)+)?
            {
                fn const_assert<$($generics)*> ()
                where
                    $($($pre)* ,)?
                    $($post)*
                {}
                let _ = const_assert::<$($generics)*>;
            }
        };
    );

    (
        $cond:expr
    ) => (
        const _: [(); 1] = [(); {
            const COND: bool = $cond;
            COND
        } as usize];
    );
}
