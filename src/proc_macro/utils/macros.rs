#![cfg_attr(rustfmt, rustfmt::skip)]

macro_rules! spanned {( $span:expr $(,)? ) => (
    ::proc_macro2::Ident::new("__", $span)
)} pub(in crate) use spanned;

macro_rules! bail {
    (
        $err_msg:expr $(,)?
    ) => (
        $crate::utils::bail! {
            $err_msg => $crate::utils::spanned!(::proc_macro2::Span::call_site())
        }
    );

    (
        $err_msg:expr => $spanned:expr $(,)?
    ) => (
        return ::syn::Result::Err(::syn::Error::new_spanned(
            &$spanned,
            $err_msg,
        ))
    );
} pub(in crate) use bail;

macro_rules! unwrap {( $proc_macro_result:expr $(,)? ) => (
    $proc_macro_result
        .unwrap_or_else(|mut err| {
            let mut iter_errors =
                err .into_iter()
                    .map(|err| Error::new_spanned(
                        err.to_compile_error(),
                        format_args!(
                            "`#[::safer_ffi::{}]`: {}",
                            $crate::utils::function_name!(),
                            err,
                        ),
                    ))
            ;
            err = iter_errors.next().unwrap();
            iter_errors.for_each(|cur| err.combine(cur));
            err.to_compile_error()
        })
        .into()
)} pub(in crate) use unwrap;

pub(in crate)
fn type_name_of_val<T> (_: T)
  -> &'static str
{
    ::core::any::type_name::<T>()
}

macro_rules! function_name {() => ({
    let mut name = $crate::utils::type_name_of_val({ fn f () {} f });
    name = &name[.. name.len() - "::f".len()].trim_end_matches("::{{closure}}");
    if let ::core::option::Option::Some(i) = name.rfind(':') {
        name = &name[i + 1..];
    }
    name
})} pub(in crate) use function_name;

macro_rules! let_quote {(
    use $($contents:tt)*
) => (
    __let_quote! {
        [
            []
            []
        ]
        $($contents)*
    }
)} pub(in crate) use let_quote;

macro_rules! __let_quote {
    (
        [
            $fst:tt
            $snd:tt
            $($deeper:tt)*
        ]
        {
            $($inner:tt)*
        } $(,
            $($rest:tt)*
        )? $(;)?
    ) => (
        __let_quote! {
            [
                $fst // duplicate fst
                $fst
                $snd
                $($deeper)*
            ]
            $($inner)*
        }
        __let_quote! {
            [
                $snd // replace fst with duplicate of snd
                $snd
                $($deeper)*
            ]
            $($($rest)*)?
        }
    );

    (
        [
            [$($path:tt)*] // fst
            $snd:tt
            $($deeper:tt)*
        ]
        $last_segment:ident $(as $rename:ident)? $(,
        $($rest:tt)* )? $(;)?
    ) => (
        let quoted = crate::utils::LazyQuote(
            || ::quote::quote_spanned!(::proc_macro2::Span::mixed_site()=>
                $($path)* $last_segment
            ),
            None.into(),
        );
        #[allow(nonstandard_style, unused_variables)]
        #[cfg(all(
            $($rename = "__if_provided",
                any(),
            )?
        ))]
        let $last_segment @ _ = quoted;
    $(
        #[allow(nonstandard_style)]
        let $rename @ _ = quoted;
    )?
        __let_quote! {
            [
                $snd // replace fst with duplicate of snd
                $snd
                $($deeper)*
            ]
            $($($rest)*)?
        }
    );

    (
        [
            [$($path:tt)*]
            $($deeper:tt)*
        ]
        $mid_segment:tt
        $($rest:tt)*
    ) => (
        __let_quote! {
            [
                [$($path)* $mid_segment]
                $($deeper)*
            ]
            $($rest)*
        }
    );

    (
        $path:tt
        /* nothing left */
    ) => ();
} pub(in crate) use __let_quote;

macro_rules! match_ {(
    ( $($input:tt)* ) $rules:tt
) => (
    macro_rules! __recurse__ $rules
    __recurse__! { $($input)* }
)} pub(in crate) use match_;

macro_rules! dbg_parse_quote {(
    $($code:tt)*
) => (
    (|| {
        fn type_of_some<T> (_: Option<T>)
          -> &'static str
        {
            ::core::any::type_name::<T>()
        }

        let target_ty = None; if false { return target_ty.unwrap(); }
        let code = ::quote::quote!( $($code)* );
        eprintln!(
            "[{}:{}:{}:parse_quote!]\n  - ty: `{ty}`\n  - code: `{code}`",
            file!(), line!(), column!(),
            ty = type_of_some(target_ty),
        );
        ::syn::parse2(code).unwrap()
    })()
)} pub(in crate) use dbg_parse_quote;

macro_rules! Quote {( $T:ty $(,)? ) => (
    ::proc_macro2::TokenStream
)} pub(in crate) use Quote;

macro_rules! Expr {( $T:ty $(,)? ) => (
    ::syn::Expr
)} pub(in crate) use Expr;
