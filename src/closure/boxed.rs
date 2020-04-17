//! `Box<dyn 'static + Send + FnMut(...) -> _>` but with a `#[repr(C)]`
//! layout (inlined virtual method table).

use_prelude!();
use ::alloc::boxed::Box;

macro_rules! with_tuple {(
    $BoxDynFnMut_N:ident => (
        $( $A_N:ident, $($A_k:ident ,)* )?
    )
) => (
    ReprC! {
        @[doc = concat!(
            "`Box<dyn 'static + Send + FnMut(" $(,
                stringify!($A_N) $(, ", ", stringify!($A_k))*
            )?,
            ") -> Ret>`",
        )]
        #[repr(C)]
        pub
        struct $BoxDynFnMut_N [Ret $(, $A_N $(, $A_k)*)?]
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
    unsafe
        impl<Ret $(, $A_N $(, $A_k)*)?> Send
            for $BoxDynFnMut_N <Ret $(, $A_N $(, $A_k)*)?>
        where
            Ret : ReprC, $(
            $A_N : ReprC, $(
            $A_k : ReprC, )*)?
        {}

    impl<Ret $(, $A_N $(, $A_k)*)?>
        $BoxDynFnMut_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        pub
        fn new<F> (f: rust::Box<F>) -> Self
        where
            F : FnMut( $($A_N $(, $A_k)*)? ) -> Ret,
            F : Send + 'static,
        {
            // Safety: `F` can be "raw-coerced" to `dyn 'static + Send + FnMut...`
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
                        F : FnMut($($A_N $(, $A_k)*)?) -> Ret,
                        F : Send + 'static,
                    {
                        let mut env_ptr = env_ptr.cast();
                        let f: &mut F = env_ptr.as_mut();
                        f( $($A_N $(, $A_k)*)? )
                    }
                    call::<F, Ret $(, $A_N $(, $A_k)*)?>
                },
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?> Drop
        for $BoxDynFnMut_N <Ret $(, $A_N $(, $A_k)*)?>
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

    impl<Ret $(, $A_N $(, $A_k)*)?>
        $BoxDynFnMut_N <Ret $(, $A_N $(, $A_k)*)?>
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
        for $BoxDynFnMut_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            fmt .debug_struct(stringify!($BoxDynFnMut_N))
                .field("env_ptr", &self.env_ptr)
                .field("call", &self.call)
                .field("free", &self.free)
                .finish()
        }
    }
)}

macro_rules! with_tuples {
    (
        $BoxDynFnMut0:ident,
    ) => (
        with_tuple!($BoxDynFnMut0 => ());
    );

    (
        $BoxDynFnMut0:ident,
        ($BoxDynFnMut_N:ident, $A_N:ident),
        $(
            ($BoxDynFnMut_K:ident, $A_K:ident),
        )*
    ) => (
        with_tuple!($BoxDynFnMut_N => (
            $A_N, $($A_K ,)*
        ));
        with_tuples!(
            $BoxDynFnMut0,
            $(
                ($BoxDynFnMut_K, $A_K),
            )*
        );
    );
}

#[cfg(not(docs))]
with_tuples! {
    BoxDynFnMut0,

    (BoxDynFnMut9, A9),
    (BoxDynFnMut8, A8),
    (BoxDynFnMut7, A7),
    (BoxDynFnMut6, A6),

    (BoxDynFnMut5, A5),
    (BoxDynFnMut4, A4),
    (BoxDynFnMut3, A3),
    (BoxDynFnMut2, A2),
    (BoxDynFnMut1, A1),
}

#[cfg(docs)]
with_tuples! {
    BoxDynFnMut0,
    (BoxDynFnMut1, A1),
}
