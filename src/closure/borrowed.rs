#![cfg_attr(rustfmt, rustfmt::skip)]
//! `&'lt mut (dyn 'lt + Send + FnMut(...) -> _>` but with a `#[repr(C)]`
//! layout (env ptr + function ptr).

use_prelude!();

macro_rules! with_tuple {(
    $RefDynFnMut_N:ident => (
        $( $A_N:ident, $($A_k:ident ,)* )?
    )
) => (
    ReprC! {
        @[doc = concat!(
            "`&'lt mut (dyn 'lt + Send + FnMut(" $(,
                stringify!($A_N) $(, ", ", stringify!($A_k))*
            )?,
            ") -> Ret)`",
        )]
        #[repr(C)]
        pub
        struct $RefDynFnMut_N ['lt, Ret $(, $A_N $(, $A_k)*)?]
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
            _lt: PhantomData<&'lt ()>,
        }
    }

    /// `&'_ mut (dyn Send + ...) : Send`
    unsafe impl<Ret $(, $A_N $(, $A_k)*)?> Send
        for $RefDynFnMut_N <'_, Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {}

    impl<'lt, F, Ret $(, $A_N $(, $A_k)*)?>
        From<&'lt mut F>
    for
        $RefDynFnMut_N<'lt, Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
        F : Fn( $($A_N $(, $A_k)*)? ) -> Ret,
        F : Send + 'static,
    {
        #[inline]
        fn from (f: &'lt mut F)
          -> Self
        {
            Self::new(f)
        }
    }

    impl<'lt, Ret $(, $A_N $(, $A_k)*)?>
        $RefDynFnMut_N <'lt, Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        pub
        fn new<F> (f: &'lt mut F) -> Self
        where
            F : FnMut( $($A_N $(, $A_k)*)? ) -> Ret,
            F : 'lt + Send,
        {
            // Safety: `F` can be "raw-coerced" to `dyn 'lt + Send + FnMut...`
            // thanks to the generic bounds on F.
            Self {
                env_ptr: ptr::NonNull::from(f).cast(),
                call: {
                    unsafe extern "C"
                    fn call<F, Ret $(, $A_N $(, $A_k)*)?> (
                        env_ptr: ptr::NonNull<c_void> $(,
                        $A_N : $A_N $(,
                        $A_k : $A_k )*)?
                    ) -> Ret
                    where
                        F : FnMut($($A_N $(, $A_k)*)?) -> Ret,
                        F : Send,
                    {
                        let mut env_ptr = env_ptr.cast();
                        let f: &'_ mut F = env_ptr.as_mut();
                        f( $($A_N $(, $A_k)*)? )
                    }
                    call::<F, Ret $(, $A_N $(, $A_k)*)?>
                },
                _lt: PhantomData,
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?>
        $RefDynFnMut_N <'_, Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        pub
        fn call (
            self: &'_ mut Self $(,
            $A_N : $A_N $(,
            $A_k : $A_k )*)?
        ) -> Ret
        {
            unsafe {
                (self.call)(self.env_ptr, $($A_N $(, $A_k)*)?)
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?> fmt::Debug
        for $RefDynFnMut_N <'_, Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            fmt .debug_struct(stringify!($RefDynFnMut_N))
                .field("env_ptr", &self.env_ptr)
                .field("call", &self.call)
                .finish()
        }
    }
)}

macro_rules! with_tuples {
    (
        $RefDynFnMut0:ident,
    ) => (
        with_tuple!($RefDynFnMut0 => ());
    );

    (
        $RefDynFnMut0:ident,
        ($RefDynFnMut_N:ident, $A_N:ident),
        $(
            ($RefDynFnMut_K:ident, $A_K:ident),
        )*
    ) => (
        with_tuple!($RefDynFnMut_N => (
            $A_N, $($A_K ,)*
        ));
        with_tuples!(
            $RefDynFnMut0,
            $(
                ($RefDynFnMut_K, $A_K),
            )*
        );
    );
}

#[cfg(not(docs))]
with_tuples! {
    RefDynFnMut0,

    (RefDynFnMut9, A9),
    (RefDynFnMut8, A8),
    (RefDynFnMut7, A7),
    (RefDynFnMut6, A6),

    (RefDynFnMut5, A5),
    (RefDynFnMut4, A4),
    (RefDynFnMut3, A3),
    (RefDynFnMut2, A2),
    (RefDynFnMut1, A1),
}

#[cfg(docs)]
with_tuples! {
    RefDynFnMut0,
    (RefDynFnMut1, A1),
}
