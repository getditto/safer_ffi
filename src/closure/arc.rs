use ::core::{
    hint,
    ffi::c_void,
    ptr,
    ops::Not,
};
use ::alloc::sync::Arc;

macro_rules! hack {(
    #[doc = $doc:expr]
    $item:item
) => (
    #[doc = $doc]
    $item
)}

macro_rules! with_tuple {(
    $Arity:ident => (
        $( $Arg0:ident, $($ArgN:ident ,)* )?
    )
) => (::paste::item! {
    hack! {
        #[doc = concat!(
            "`Arc<dyn 'static + Send + Sync + Fn(" $(,
                stringify!($Arg0) $(, ", ", stringify!($ArgN))*
            )?,
            ") -> Ret>`",
        )]
        #[repr(C)]
        pub
        struct [< ArcDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?> {
            /// `Arc<Erased>`
            pub
            env_ptr: ptr::NonNull<c_void>,

            pub
            call:
                unsafe extern "C"
                fn (
                    env_ptr: ptr::NonNull<c_void> $(,
                        $Arg0 $(,
                        $ArgN
                    )*)?
                ) -> Ret
            ,

            pub
            release:
                unsafe extern "C"
                fn (env_ptr: ptr::NonNull<c_void>)
            ,

            pub
            retain:
                unsafe extern "C"
                fn (env_ptr: ptr::NonNull<c_void>)
            ,
        }
    }

    /// `Arc<dyn Send + Sync + ...> : Send`
    unsafe impl<Ret $(, $Arg0 $(, $ArgN)*)?> Send
        for [< ArcDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {}

    /// `Arc<dyn Send + Sync + ...> : Sync`
    unsafe impl<Ret $(, $Arg0 $(, $ArgN)*)?> Sync
        for [< ArcDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {}

    impl<Ret $(, $Arg0 $(, $ArgN)*)?>
        [< ArcDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {
        #[inline]
        pub
        fn new<F> (f: Arc<F>) -> Self
        where
            F : Fn( $($Arg0 $(, $ArgN)*)? ) -> Ret,
            F : Send + Sync + 'static,
        {
            // Safety:
            //   - `F` can be "raw-coerced" to `dyn 'static + Send + Sync + Fn...`
            //     thanks to the generic bounds on F.
            Self {
                env_ptr: ptr::NonNull::from(unsafe { &*Arc::into_raw(f) }).cast(),
                release: {
                    unsafe extern "C"
                    fn release<F> (env_ptr: ptr::NonNull<c_void>)
                    where
                        F : Send + Sync + 'static,
                    {
                        drop::<Arc<F>>(Arc::from_raw(env_ptr.cast().as_ptr()));
                    }
                    release::<F>
                },
                retain: {
                    unsafe extern "C"
                    fn retain<F> (env_ptr: ptr::NonNull<c_void>)
                    where
                        F : Send + Sync + 'static,
                    {
                        use ::core::mem::{forget, ManuallyDrop};

                        let at_arc: &Arc<F> = &*ManuallyDrop::new(Arc::from_raw(
                            env_ptr.cast().as_ptr()
                        ));
                        forget(Arc::clone(at_arc));
                    }
                    retain::<F>

                },
                call: {
                    unsafe extern "C"
                    fn call<F, Ret $(, $Arg0 $(, $ArgN)*)?> (
                        env_ptr: ptr::NonNull<c_void> $(,
                        $Arg0 : $Arg0 $(,
                        $ArgN : $ArgN )*)?
                    ) -> Ret
                    where
                        F : Fn($($Arg0 $(, $ArgN)*)?) -> Ret,
                        F : Send + 'static,
                    {
                        let env_ptr = env_ptr.cast();
                        let f: &F = env_ptr.as_ref();
                        f( $($Arg0 $(, $ArgN)*)? )
                    }
                    call::<F, Ret $(, $Arg0 $(, $ArgN)*)?>
                },
            }
        }
    }

    impl<Ret $(, $Arg0 $(, $ArgN)*)?> Drop
        for [< ArcDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {
        fn drop (self: &'_ mut Self)
        {
            unsafe {
                (self.release)(self.env_ptr)
            }
        }
    }

    impl<Ret $(, $Arg0 $(, $ArgN)*)?> Clone
        for [< ArcDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {
        fn clone (self: &'_ Self)
          -> Self
        {
            unsafe {
                (self.retain)(self.env_ptr)
            }
            Self { .. *self }
        }
    }

    impl<Ret $(, $Arg0 $(, $ArgN)*)?>
        [< ArcDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {
        #[inline]
        pub
        fn call (
            self: &'_ Self $(,
            $Arg0 : $Arg0 $(,
            $ArgN : $ArgN )*)?
        ) -> Ret
        {
            unsafe {
                (self.call)(self.env_ptr, $($Arg0 $(, $ArgN)*)?)
            }
        }
    }
})}

macro_rules! with_tuples {
    () => (
        with_tuple!(_0 => ());
    );

    (
        $Arg0:ident, $($ArgN:ident ,)*
    ) => (
        with_tuple!($Arg0 => (
            $Arg0, $($ArgN ,)*
        ));
        with_tuples!($($ArgN ,)*);
    );
}

with_tuples! {
    _9, _8, _7, _6, _5, _4, _3, _2, _1,
}
