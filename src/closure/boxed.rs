//! Simplified for lighter documentation, but the actual `struct` definitions
//! and impls range from `BoxDynFn0` up to `BoxDynFn6`.

use ::core::{
    hint,
    ffi::c_void,
    ptr,
    ops::Not,
};
use ::alloc::boxed::Box;

use_prelude!();

macro_rules! hack {(
    #[doc = $doc:expr]
    $item:item
) => (
    #[doc = $doc]
    $item
)}

macro_rules! with_tuple {(
    $BoxDynFn_N:ident => (
        $( $A_N:ident, $($A_k:ident ,)* )?
    )
) => (
    derive_ReprC! {
        @[doc = concat!(
            "`Box<dyn 'static + Send + Fn(" $(,
                stringify!($A_N) $(, ", ", stringify!($A_k))*
            )?,
            ") -> Ret>`",
        )]
        #[repr(C)]
        pub
        struct $BoxDynFn_N [Ret $(, $A_N $(, $A_k)*)?]
        where {
            Ret : ReprC, $(
            $A_N : ReprC, $(
            $A_k : ReprC, )*)?
        }
        {
            env_ptr: ptr::NonNull<c_void>,
            call:
                unsafe extern "C"
                fn (
                    env_ptr: ptr::NonNull<c_void> $(,
                        $A_N $(,
                        $A_k
                    )*)?
                ) -> Ret
            ,
            free:
                unsafe extern "C"
                fn (env_ptr: ptr::NonNull<c_void>)
            ,
        }
    }

    /// `Box<dyn Send + ...> : Send`
    unsafe impl<Ret $(, $A_N $(, $A_k)*)?> Send
        for $BoxDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {}

    impl<Ret $(, $A_N $(, $A_k)*)?>
        $BoxDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        pub
        fn new<F> (f: Box<F>) -> Self
        where
            F : Fn( $($A_N $(, $A_k)*)? ) -> Ret,
            F : Send + 'static,
        {
            // Safety: `F` can be "raw-coerced" to `dyn 'static + Send + Fn...`
            // thanks to the generic bounds on F.
            Self {
                env_ptr: ptr::NonNull::from(Box::leak(f)).cast(),
                free: {
                    unsafe extern "C"
                    fn free<F> (env_ptr: ptr::NonNull<c_void>)
                    where
                        F : Send + 'static,
                    {
                        drop::<Box<F>>(Box::from_raw(env_ptr.cast().as_ptr()));
                    }
                    free::<F>
                },
                call: {
                    unsafe extern "C"
                    fn call<F, Ret $(, $A_N $(, $A_k)*)?> (
                        env_ptr: ptr::NonNull<c_void> $(,
                        $A_N : $A_N $(,
                        $A_k : $A_k )*)?
                    ) -> Ret
                    where
                        F : Fn($($A_N $(, $A_k)*)?) -> Ret,
                        F : Send + 'static,
                    {
                        let env_ptr = env_ptr.cast();
                        let f: &F = env_ptr.as_ref();
                        f( $($A_N $(, $A_k)*)? )
                    }
                    call::<F, Ret $(, $A_N $(, $A_k)*)?>
                },
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?> Drop
        for $BoxDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        fn drop (self: &'_ mut Self)
        {
            unsafe {
                (self.free)(self.env_ptr)
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?> ::core::fmt::Debug
        for $BoxDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut ::core::fmt::Formatter<'_>)
          -> ::core::fmt::Result
        {
            <str as ::core::fmt::Display>::fmt(stringify!($BoxDynFn_N), fmt)
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?>
        $BoxDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        pub
        fn call (
            self: &'_ Self $(,
            $A_N : $A_N $(,
            $A_k : $A_k )*)?
        ) -> Ret
        {
            unsafe {
                (self.call)(self.env_ptr, $($A_N $(, $A_k)*)?)
            }
        }
    }
)}

macro_rules! with_tuples {
    (
        $BoxDynFn0:ident,
    ) => (
        with_tuple!($BoxDynFn0 => ());
    );

    (
        $BoxDynFn0:ident,
        ($BoxDynFn_N:ident, $A_N:ident),
        $(
            ($BoxDynFn_K:ident, $A_K:ident),
        )*
    ) => (
        with_tuple!($BoxDynFn_N => (
            $A_N, $($A_K ,)*
        ));
        with_tuples!(
            $BoxDynFn0,
            $(
                ($BoxDynFn_K, $A_K),
            )*
        );
    );
}

#[cfg(not(docs))]
with_tuples! {
    BoxDynFn0,

    (BoxDynFn9, A9),
    (BoxDynFn8, A8),
    (BoxDynFn7, A7),
    (BoxDynFn6, A6),

    (BoxDynFn5, A5),
    (BoxDynFn4, A4),
    (BoxDynFn3, A3),
    (BoxDynFn2, A2),
    (BoxDynFn1, A1),
}

#[cfg(docs)]
with_tuples! {
    BoxDynFn0,
    (BoxDynFn1, A1),
}
