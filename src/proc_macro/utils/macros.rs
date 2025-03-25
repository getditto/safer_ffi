pub(crate) fn reified_span(span: impl Into<Option<::proc_macro2::Span>>) -> impl ::quote::ToTokens {
    ::proc_macro2::Ident::new(
        "__",
        span.into().unwrap_or_else(::proc_macro2::Span::call_site),
    )
}

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! bail {
    (
        $err_msg:expr $(,)?
    ) => (
        $crate::utils::bail! {
            $err_msg => $crate::utils::reified_span(::core::prelude::v1::None)
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
}
pub(crate) use bail;

#[cfg_attr(rustfmt, rustfmt::skip)]
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
)}
pub(crate) use unwrap;

pub(crate) fn type_name_of_val<T>(_: T) -> &'static str {
    ::core::any::type_name::<T>()
}

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! function_name {() => ({
    let mut name = $crate::utils::type_name_of_val({ fn f () {} f });
    name = &name[.. name.len() - "::f".len()].trim_end_matches("::{{closure}}");
    if let ::core::option::Option::Some(i) = name.rfind(':') {
        name = &name[i + 1..];
    }
    name
})}
pub(crate) use function_name;

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! let_quote {(
    $( #[rustfmt::skip] )?
    use $($contents:tt)*
) => (
    __let_quote! {
        [
            []
            []
        ]
        $($contents)*
    }
)}
pub(crate) use let_quote;

#[cfg_attr(rustfmt, rustfmt::skip)]
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
        let ref $last_segment @ _ = quoted;
    $(
        #[allow(nonstandard_style)]
        let ref $rename @ _ = quoted;
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
}
pub(crate) use __let_quote;

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! match_ {(
    ( $($input:tt)* ) $rules:tt
) => (
    #[cfg_attr(rustfmt, rustfmt::skip)]
    macro_rules! __recurse__ $rules
    __recurse__! { $($input)* }
)}
pub(crate) use match_;

#[cfg_attr(rustfmt, rustfmt::skip)]
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
)}
pub(crate) use dbg_parse_quote;

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! Quote {( $T:ty $(,)? ) => (
    ::proc_macro2::TokenStream
)}
pub(crate) use Quote;

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! Expr {( $T:ty $(,)? ) => (
    ::syn::Expr
)}
pub(crate) use Expr;

// Like `quote_spanned!` (defaulting to mixed_site), but allowing for
// `#{ … }` interpolations.
#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! squote {
    // ()
    (
        (out:
            $($out:tt)*
        )
        (in:
            [
                ( $($paren:tt)* )
                $($rest:tt)*
            ]
            $($in:tt)*
        )
    ) => (squote! {
        (out:
            [parenthesized: ]
            $($out)*
        )
        (in:
            [ $($paren)* ]
            [ $($rest)* ]
            $($in)*
        )
    });
    (
        (out:
            [parenthesized: $($paren:tt)*]
            [ $($acc:tt)* ]
            $($out:tt)*
        )
        (in:
            [
                /* exhausted layer */
            ]
            $($in:tt)*
        )
    ) => (squote! {
        (out:
            [
                $($acc)*
                ( $($paren)* )
            ]
            $($out)*
        )
        (in:
            $($in)*
        )
    });

    // {}
    (
        (out:
            $($out:tt)*
        )
        (in:
            [
                { $($brace:tt)* }
                $($rest:tt)*
            ]
            $($in:tt)*
        )
    ) => (squote! {
        (out:
            [braced: ]
            $($out)*
        )
        (in:
            [ $($brace)* ]
            [ $($rest)* ]
            $($in)*
        )
    });
    (
        (out:
            [braced: $($brace:tt)*]
            [ $($acc:tt)* ]
            $($out:tt)*
        )
        (in:
            [
                /* exhausted layer */
            ]
            $($in:tt)*
        )
    ) => (squote! {
        (out:
            [
                $($acc)*
                { $($brace)* }
            ]
            $($out)*
        )
        (in:
            $($in)*
        )
    });

    // []
    (
        (out:
            $($out:tt)*
        )
        (in:
            [
                [ $($bracket:tt)* ]
                $($rest:tt)*
            ]
            $($in:tt)*
        )
    ) => (squote! {
        (out:
            [bracketed: ]
            $($out)*
        )
        (in:
            [ $($bracket)* ]
            [ $($rest)* ]
            $($in)*
        )
    });
    (
        (out:
            [bracketed: $($bracket:tt)*]
            [ $($acc:tt)* ]
            $($out:tt)*
        )
        (in:
            [
                /* exhausted layer */
            ]
            $($in:tt)*
        )
    ) => (squote! {
        (out:
            [
                $($acc)*
                [ $($bracket)* ]
            ]
            $($out)*
        )
        (in:
            $($in)*
        )
    });

    // #{…}
    (
        (out:
            [ $($acc:tt)* ]
            $($out:tt)*
        )
        (in:
            [
                #{ $e:expr }
                $($rest:tt)*
            ]
            $($in:tt)*
        )
    ) => (
        match &$e { interpolated => {
            squote! {
                (out:
                    [
                        $($acc)*
                        #interpolated
                    ]
                    $($out)*
                )
                (in:
                    [ $($rest)* ]
                    $($in)*
                )
            }
        }}
    );

    // $otherwise:tt
    (
        (out:
            [ $($acc:tt)* ]
            $($out:tt)*
        )
        (in:
            [
                $otherwise:tt
                $($rest:tt)*
            ]
            $($in:tt)*
        )
    ) => (squote! {
        (out:
            [
                $($acc)*
                $otherwise
            ]
            $($out)*
        )
        (in:
            [ $($rest)* ]
            $($in)*
        )
    });

    // end
    (
        (out:
            [ $($out:tt)* ]
            /* no more acc layers */
        )
        (in:
            [ /* exhausted layer */ ]
            /* no more input layers */
        )
    ) => (
        ::quote::quote_spanned! {
            $($out)*
        }
    );

    // entry-point (with provided span)
    (
        @$span:expr=>
        $($input:tt)*
    ) => (
        squote! {
            (out:
                [$span=> ]
            )
            (in:
                [
                    $($input)*
                ]
            )
        }
    );

    // entry-point (no span provided)
    (
        $($input:tt)*
    ) => (
        squote! { @::proc_macro2::Span::mixed_site()=>
            $($input)*
        }
    )
}
pub(crate) use squote;
