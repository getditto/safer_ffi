#![cfg_attr(rustfmt, rustfmt::skip)]
use_prelude!();

/// A `ReprC` _standalone_ type with the same layout and ABI as
/// [`::libc::c_char`][crate::libc::c_char].
///
/// By _standalone_, the idea is that this is defined as a (`transparent`) _newtype_ `struct`,
/// rather than as a _`type` alias_, which is error-prone and yields less-portable headers (since
/// the header generation will resolve the type alias and emit, for instance, `int8_t`, ⚠️).
///
/// By using this type, you guarantee that the C `char` type be used in the headers.
#[repr(transparent)]
#[derive(
    Debug,
    Clone, Copy,
    Default,
    PartialOrd, Ord,
    PartialEq, Eq,
    Hash,
)]
pub
struct c_char /* = */ (
    pub
    u8,
);

/// Assert that `crate::libc::c_char` is either `uint8_t` or `int8_t`.
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
        let _ = is_u8_or_i8::<crate::libc::c_char>;
    };
};

unsafe
impl LegacyCType
    for c_char
{ __cfg_headers__! {
    fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt.write_str("char")
    }

    fn c_var_fmt (
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

    fn c_define_self (
        _: &'_ mut dyn crate::headers::Definer,
    ) -> io::Result<()>
    {
        Ok(())
    }

    __cfg_csharp__! {
        fn csharp_define_self (
            _: &'_ mut dyn crate::headers::Definer,
        ) -> io::Result<()>
        {
            Ok(())
        }

        fn csharp_ty ()
          -> rust::String
        {
            "byte".into()
        }
    }
} type OPAQUE_KIND = crate::layout::OpaqueKind::Concrete; }

from_CType_impl_ReprC! {
    c_char
}
