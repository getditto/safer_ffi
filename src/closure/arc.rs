//! `Arc<dyn 'static + Send + Sync + Fn(...) -> _>` but with a `#[repr(C)]`
//! layout (inlined virtual method table).

use ::alloc::sync::Arc;

use_prelude!();

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! with_tuple {(
    $ArcDynFn_N:ident => (
        $( $A_N:ident, $($A_k:ident ,)* )?
    )
) => (
    impl<Ret $(, $A_N $(, $A_k)*)?>
        crate::arc::FitForCArc
    for
        dyn 'static + Send + Sync + Fn($($A_N $(, $A_k)*)?) -> Ret
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        type CArcWrapped = $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>;
    }

    ReprC! {
        @[doc = concat!(
            "`Arc<dyn Send + Sync + Fn(" $(,
                stringify!($A_N) $(, ", ", stringify!($A_k))*
            )?,
            ") -> Ret>`",
        )]
        #[repr(C)]
        pub
        struct $ArcDynFn_N [Ret $(, $A_N $(, $A_k)*)?]
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
            release:
                unsafe extern "C"
                fn (env_ptr: ptr::NonNull<c_void>)
            ,
            retain: Option<
                unsafe extern "C"
                fn (env_ptr: ptr::NonNull<c_void>)
            >,
        }
    }

    const_assert!(
        for[T]
        [T : ?Sized + Send + Sync] => [Arc<T> : Send + Sync]
    );
    /// `Arc<dyn Send + Sync + ...> : Send`
    unsafe impl<Ret $(, $A_N $(, $A_k)*)?> Send
        for $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {}
    /// `Arc<dyn Send + Sync + ...> : Sync`
    unsafe impl<Ret $(, $A_N $(, $A_k)*)?> Sync
        for $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {}

    impl<F, Ret $(, $A_N $(, $A_k)*)?>
        From<Arc<F>>
    for
        $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
        F : Fn( $($A_N $(, $A_k)*)? ) -> Ret,
        F : Send + Sync + 'static,
    {
        #[inline]
        fn from (f: Arc<F>)
          -> Self
        {
            Self::new(f)
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?>
        $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        pub
        fn new<F> (f: Arc<F>)
          -> Self
        where
            F : Fn( $($A_N $(, $A_k)*)? ) -> Ret,
            F : Send + Sync + 'static,
        {
            // Safety: `F` can be "raw-coerced" to `dyn 'static + Send + Fn...`
            // thanks to the generic bounds on F.
            let env_ptr = unsafe {
                ptr::NonNull::new_unchecked(Arc::into_raw(f) as _)
            };
            #[cfg(feature = "alloc-tracking")]
            crate::alloc_tracking::AllocationTracker::track_arc_new(
                env_ptr.cast::<F>().as_ptr()
            );
            Self {
                env_ptr,
                release: {
                    unsafe extern "C"
                    fn release<F> (env_ptr: ptr::NonNull<c_void>)
                    where
                        F : Send + Sync + 'static,
                    {
                        #[cfg(feature = "alloc-tracking")]
                        crate::alloc_tracking::AllocationTracker::track_arc_drop(
                            env_ptr.cast::<F>().as_ptr()
                        );
                        unsafe {
                            drop::<Arc<F>>(Arc::from_raw(env_ptr.cast().as_ptr()));
                        }
                    }
                    release::<F>
                },
                retain: Some({
                    unsafe extern "C"
                    fn retain<F> (env_ptr: ptr::NonNull<c_void>)
                    where
                        F : Send + Sync + 'static,
                    {
                        #[cfg(feature = "alloc-tracking")]
                        crate::alloc_tracking::AllocationTracker::track_arc_clone(
                            env_ptr.cast::<F>().as_ptr()
                        );
                        mem::forget(Arc::<F>::clone(&
                            mem::ManuallyDrop::new(unsafe {
                                Arc::from_raw(env_ptr.cast().as_ptr())
                            })
                        ));
                    }
                    retain::<F>
                }),
                call: {
                    unsafe extern "C"
                    fn call<F, Ret $(, $A_N $(, $A_k)*)?> (
                        env_ptr: ptr::NonNull<c_void> $(,
                        $A_N : $A_N $(,
                        $A_k : $A_k )*)?
                    ) -> Ret
                    where
                        F : Fn($($A_N $(, $A_k)*)?) -> Ret,
                        F : Send + Sync + 'static,
                    {
                        let env_ptr = env_ptr.cast();
                        let f: &F = unsafe { env_ptr.as_ref() };
                        f( $($A_N $(, $A_k)*)? )
                    }
                    call::<F, Ret $(, $A_N $(, $A_k)*)?>
                },
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?> Drop
        for $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        fn drop (self: &'_ mut Self)
        {
            unsafe {
                (self.release)(self.env_ptr)
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?> Clone
        for $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        #[inline]
        fn clone (self: &'_ Self)
          -> Self
        {
            let retain = self.retain.expect(concat!(
                "Cannot `.clone()` a `",
                stringify!($ArcDynFn_N),
                "` whose `.retain` function pointer is `NULL`",
            ));
            unsafe {
                retain(self.env_ptr);
                Self { .. *self }
            }
        }
    }

    impl<Ret $(, $A_N $(, $A_k)*)?>
        $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
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

    impl<Ret $(, $A_N $(, $A_k)*)?> fmt::Debug
        for $ArcDynFn_N <Ret $(, $A_N $(, $A_k)*)?>
    where
        Ret : ReprC, $(
        $A_N : ReprC, $(
        $A_k : ReprC, )*)?
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            fmt .debug_struct(stringify!($ArcDynFn_N))
                .field("env_ptr", &self.env_ptr)
                .field("call", &self.call)
                .field("release", &self.release)
                .field("retain", &self.retain)
                .finish()
        }
    }
)}

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! with_tuples {
    (
        $ArcDynFn0:ident,
    ) => (
        with_tuple!($ArcDynFn0 => ());
    );

    (
        $ArcDynFn0:ident,
        ($ArcDynFn_N:ident, $A_N:ident),
        $(
            ($ArcDynFn_K:ident, $A_K:ident),
        )*
    ) => (
        with_tuple!($ArcDynFn_N => (
            $A_N, $($A_K ,)*
        ));
        with_tuples!(
            $ArcDynFn0,
            $(
                ($ArcDynFn_K, $A_K),
            )*
        );
    );
}

#[cfg(not(docs))]
with_tuples! {
    ArcDynFn0,

    (ArcDynFn9, A9),
    (ArcDynFn8, A8),
    (ArcDynFn7, A7),
    (ArcDynFn6, A6),

    (ArcDynFn5, A5),
    (ArcDynFn4, A4),
    (ArcDynFn3, A3),
    (ArcDynFn2, A2),
    (ArcDynFn1, A1),
}

#[cfg(docs)]
with_tuples! {
    ArcDynFn0,
    (ArcDynFn1, A1),
}

#[cfg(all(test, feature = "alloc-tracking"))]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::alloc_tracking::AllocationTracker;

    #[test]
    #[serial]
    fn test_arc_closure_no_leak() {
        let res = AllocationTracker::run(|| {
            let closure = ArcDynFn0::new(Arc::new(|| {}));
            drop(closure);
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial]
    fn test_arc_closure_with_args_no_leak() {
        let res = AllocationTracker::run(|| {
            let closure = ArcDynFn1::new(Arc::new(|x: i32| x + 1));
            let result = closure.call(41);
            assert_eq!(result, 42);
            drop(closure);
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial]
    fn test_arc_closure_clone_no_leak() {
        let res = AllocationTracker::run(|| {
            let closure1 = ArcDynFn0::new(Arc::new(|| {}));
            let closure2 = closure1.clone();
            let closure3 = closure2.clone();
            drop(closure1);
            drop(closure2);
            drop(closure3);
        });
        assert!(res.is_ok());
    }

    #[test]
    #[serial]
    fn test_arc_closure_leak() {
        let res = AllocationTracker::run(|| {
            let closure = ArcDynFn0::new(Arc::new(|| {}));
            ::core::mem::forget(closure);
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Arc leak"));
    }

    #[test]
    #[serial]
    fn test_arc_closure_clone_leak() {
        let res = AllocationTracker::run(|| {
            let closure1 = ArcDynFn0::new(Arc::new(|| {}));
            let closure2 = closure1.clone();
            drop(closure1);
            // closure2 is forgotten, should detect leak with ref count 1
            ::core::mem::forget(closure2);
        });
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Arc leak"));
    }

    #[test]
    #[serial]
    fn test_arc_closure_with_capture_no_leak() {
        let res = AllocationTracker::run(|| {
            let captured = String::from("hello");
            let closure = ArcDynFn0::new(Arc::new(move || captured.len()));
            let result = closure.call();
            assert_eq!(result, 5);
            drop(closure);
        });
        assert!(res.is_ok());
    }
}
