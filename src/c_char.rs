use_prelude!();

#[repr(transparent)]
#[derive(Clone, Copy)]
pub
struct c_char /* = */ (
    pub
    u8,
);

/// Assert that `::libc::c_char` is either `uint8_t` or `int8_t`.
#[cfg(not(target_arch = "wasm32"))] // no libc on WASM
const _: () = {
    trait IsU8OrI8
    {}

    impl IsU8OrI8
        for u8
    {}
    impl IsU8OrI8
        for i8
    {}

    const _: () = {
        fn is_u8_or_i8<T>() where
            T : IsU8OrI8,
        {}
        let _ = is_u8_or_i8::<::libc::c_char>;
    };
};

unsafe
impl CType
    for c_char
{ cfg_headers! {
    fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
      -> R
    {
        ret(&"char")
    }

    fn c_fmt (
        fmt: &'_ mut fmt::Formatter<'_>,
        var_name: &'_ str,
    ) -> fmt::Result
    {
        write!(fmt,
            "char{sep}{}",
            var_name,
            sep = if var_name.is_empty() { "" } else { " " },
        )
    }
}}

from_CType_impl_ReprC! {
    c_char
}
