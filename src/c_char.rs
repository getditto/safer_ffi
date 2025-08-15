use_prelude!();

/// A `ReprC` _standalone_ type with the same layout and ABI as
/// [`::libc::c_char`][crate::libc::c_char].
///
/// By _standalone_, the idea is that this is defined as a (`transparent`) _newtype_ `struct`,
/// rather than as a _`type` alias_, which is error-prone and yields less-portable headers (since
/// the header generation will resolve the type alias and emit, for instance, `int8_t`, ⚠️).
///
/// By using this type, you guarantee that the C `char` type be used in the headers.
#[cfg_attr(feature = "stabby", stabby::stabby)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct c_char(pub u8);

/// Assert that `crate::libc::c_char` is either `uint8_t` or `int8_t`.
const _: () = {
    trait IsU8OrI8 {}

    impl IsU8OrI8 for u8 {}
    impl IsU8OrI8 for i8 {}

    const _: () = {
        fn is_u8_or_i8<T>()
        where
            T: IsU8OrI8,
        {
        }
        let _ = is_u8_or_i8::<crate::libc::c_char>;
    };
};

unsafe impl CType for c_char {
    type OPAQUE_KIND = OpaqueKind::Concrete;
    __cfg_headers__! {
        fn short_name() -> String {
            "char".into()
        }

        fn define_self__impl(
            _language: &dyn HeaderLanguage,
            _definer: &mut dyn Definer,
        ) -> io::Result<()>
        {
            Ok(())
        }

        fn metadata_type_usage() -> String {
            r#""kind": "char""#.into()
        }

        fn render(
            out: &mut dyn io::Write,
            language: &dyn HeaderLanguage,
        ) -> io::Result<()>
        {
            language.emit_primitive_ty(
                out,
                primitives::Primitive::CChar,
            )
        }
    }
}

from_CType_impl_ReprC! {
    c_char
}
