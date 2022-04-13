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
            $spanned,
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
