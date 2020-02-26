use ::core::{
    hint,
    ffi::c_void,
    ptr,
    ops::Not,
};
use ::alloc::boxed::Box;

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
            "`Box<dyn 'static + Send + Fn(" $(,
                stringify!($Arg0) $(, ", ", stringify!($ArgN))*
            )?,
            ") -> Ret>`",
        )]
        #[repr(C)]
        pub
        struct [< BoxDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?> {
            /// `Box<Erased>`
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
            free:
                unsafe extern "C"
                fn (env_ptr: ptr::NonNull<c_void>)
            ,
        }
    }

    /// `Box<dyn Send + ...> : Send`
    unsafe impl<Ret $(, $Arg0 $(, $ArgN)*)?> Send
        for [< BoxDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {}

    impl<Ret $(, $Arg0 $(, $ArgN)*)?>
        [< BoxDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {
        #[inline]
        pub
        fn new<F> (f: Box<F>) -> Self
        where
            F : Fn( $($Arg0 $(, $ArgN)*)? ) -> Ret,
            F : Send + 'static,
        {
            // Safety: `F` can be "raw-coerced" to `dyn 'static + Send + Fn...`
            // thanks to the generic bounds on F.
            Self {
                ptr: ptr::NonNull::from(Box::leak(f)).cast(),
                free: {
                    unsafe extern "C"
                    fn free<F> (ptr: ptr::NonNull<c_void>)
                    where
                        F : Send + 'static,
                    {
                        drop::<Box<F>>(Box::from_raw(ptr.cast().as_ptr()));
                    }
                    free::<F>
                },
                call: {
                    unsafe extern "C"
                    fn call<F, Ret $(, $Arg0 $(, $ArgN)*)?> (
                        ptr: ptr::NonNull<c_void> $(,
                        $Arg0 : $Arg0 $(,
                        $ArgN : $ArgN )*)?
                    ) -> Ret
                    where
                        F : Fn($($Arg0 $(, $ArgN)*)?) -> Ret,
                        F : Send + 'static,
                    {
                        let ptr = ptr.cast();
                        let f: &F = ptr.as_ref();
                        f( $($Arg0 $(, $ArgN)*)? )
                    }
                    call::<F, Ret $(, $Arg0 $(, $ArgN)*)?>
                },
            }
        }
    }

    impl<Ret $(, $Arg0 $(, $ArgN)*)?> Drop
        for [< BoxDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
    {
        fn drop (self: &'_ mut Self)
        {
            unsafe {
                (self.free)(self.ptr)
            }
        }
    }

    impl<Ret $(, $Arg0 $(, $ArgN)*)?>
        [< BoxDynFn $Arity >] <Ret $(, $Arg0 $(, $ArgN)*)?>
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
                (self.call)(self.ptr, $($Arg0 $(, $ArgN)*)?)
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
