use_prelude!();

mod void {
    #[derive(Clone, Copy)]
    pub
    struct Void {
        _0: (),
    }
    pub const Void: Void = Void { _0: () };
}
pub use void::Void;

unsafe
impl CType for Void {
    #[cfg(feature = "headers")]
    fn with_short_name<R> (
        ret: impl FnOnce(&'_ dyn fmt::Display) -> R,
    ) -> R
    {
        ret(&"void")
    }

    #[cfg(feature = "headers")]
    fn c_fmt (
        fmt: &'_ mut fmt::Formatter<'_>,
        var_name: &'_ str,
    ) -> fmt::Result
    {
        fmt.write_str("void")
    }
}
from_CType_impl_ReprC! { Void }

unsafe
impl ReprC for ::core::ffi::c_void {
    type CLayout = Void;

    fn is_valid (it: &'_ Void)
      -> bool
    {
        panic!("Trying to construct a `c_void` is a logic error");
    }
}

derive_ReprC! {
    #[repr(C)]
    pub
    struct Tuple1[T0] {
        pub _0: T0,
    }
}

derive_ReprC! {
    #[repr(C)]
    pub
    struct Tuple2[T0, T1] {
        pub _0: T0,
        pub _1: T1,
    }
}

derive_ReprC! {
    #[repr(C)]
    pub
    struct Tuple3[T0, T1, T2] {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
    }
}

derive_ReprC! {
    #[repr(C)]
    pub
    struct Tuple4[T0, T1, T2, T3] {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
    }
}
derive_ReprC! {
    #[repr(C)]
    pub
    struct Tuple5[T0, T1, T2, T3, T4] {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
        pub _4: T4,
    }
}
derive_ReprC! {
    #[repr(C)]
    pub
    struct Tuple6[T0, T1, T2, T3, T4, T5] {
        pub _0: T0,
        pub _1: T1,
        pub _2: T2,
        pub _3: T3,
        pub _4: T4,
        pub _5: T5,
    }
}
