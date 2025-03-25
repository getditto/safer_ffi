use super::*;

mod const_;
mod fn_;
mod static_;
mod type_;

#[allow(unused_macros)]
#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! emit {( $($tt:tt)* ) => ( $($tt)* )}

pub(super) fn ffi_export(
    args: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2> {
    use ::proc_macro2::*;

    match parse2::<Item>(input)? {
        | Item::Struct(struct_) => {
            type_::handle(parse2(args)?, &struct_.ident, &struct_.generics, &struct_)
        },
        | Item::Enum(enum_) => type_::handle(parse2(args)?, &enum_.ident, &enum_.generics, &enum_),
        | Item::Fn(fn_) => fn_::handle(parse2(args)?, fn_),
        | Item::Const(const_) => const_::handle(parse2(args)?, const_),
        | Item::Static(static_) => static_::handle(parse2(args)?, static_),
        | _otherwise => bail!("unsupported item type"),
    }
}
