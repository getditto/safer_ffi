inline_mod!(handle_fptr);

fn feed_to_macro_rules (input: TokenStream, name: Ident)
  -> TokenStream
{
    let input = parse_macro_input!(input as DeriveInput);
    if let Some(expansion) = try_handle_fptr(&input) {
        return expansion;
    }
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = input;
    let ret = TokenStream::from(match data {
        | Data::Enum(DataEnum {
            enum_token: ref enum_,
            ref variants,
            ..
        }) => quote! {
            ::safer_ffi::layout::ReprC! {
                #(#attrs)*
                #vis
                #enum_ #ident {
                    #variants
                }
            }
        },
        | Data::Struct(DataStruct {
            struct_token: ref struct_,
            ref fields,
            semi_token: ref maybe_semi_colon,
        }) => {
            let (params, bounds) = generics.my_split();
            quote! {
                ::safer_ffi::layout::#name! {
                    #(#attrs)*
                    #vis
                    #struct_ #ident
                                [#params]
                            where {
                                #(#bounds ,)*
                            }
                        #fields
                    #maybe_semi_colon
                }
            }
        },
        | Data::Union(ref union_) => {
            Error::new_spanned(
                union_.union_token,
                "`union`s are not supported yet."
            ).to_compile_error()
        },
    });
    #[cfg(feature = "verbose-expansions")]
    println!("{}", ret.to_string());
    ret
}

/// Safely implement [`ReprC`]
/// for a `#[repr(C)]` struct **when all its fields are [`ReprC`]**.
///
/// [`ReprC`]: /safer_ffi/layout/trait.ReprC.html
///
/// # Examples
///
/// ### Simple `struct`
///
/// ```rust
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(C)]
/// struct Instant {
///     seconds: u64,
///     nanos: u32,
/// }
/// ```
///
///   - corresponding to the following C definition:
///
///     ```C
///     typedef struct {
///         uint64_t seconds;
///         uint32_t nanos;
///     } Instant_t;
///     ```
///
/// ### Field-less `enum`
///
/// ```rust
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(u8)]
/// enum Status {
///     Ok = 0,
///     Busy,
///     NotInTheMood,
///     OnStrike,
///     OhNo,
/// }
/// ```
///
///   - corresponding to the following C definition:
///
///     ```C
///     typedef uint8_t Status_t; enum {
///         STATUS_OK = 0,
///         STATUS_BUSY,
///         STATUS_NOT_IN_THE_MOOD,
///         STATUS_ON_STRIKE,
///         STATUS_OH_NO,
///     }
///     ```
///
/// ### Generic `struct`
///
/// In that case, it is required that the struct's generic types carry a
/// `: ReprC` bound each:
///
/// ```rust
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(C)]
/// struct Point<Coordinate : ReprC> {
///     x: Coordinate,
///     y: Coordinate,
/// }
/// ```
///
/// Each monomorphization leads to its own C definition:
///
///   - **`Point<i32>`**
///
///     ```C
///     typedef struct {
///         int32_t x;
///         int32_t y;
///     } Point_int32_t;
///     ```
///
///   - **`Point<f64>`**
///
///     ```C
///     typedef struct {
///         double x;
///         double y;
///     } Point_double_t;
///     ```
#[cfg(feature = "proc_macros")]
#[proc_macro_attribute] pub
fn derive_ReprC (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    if let Some(tt) = TokenStream2::from(attrs).into_iter().next() {
        return Error::new_spanned(tt,
            "Unexpected parameter",
        ).to_compile_error().into();
    }
    feed_to_macro_rules(input, parse_quote!(ReprC))
}

#[proc_macro_attribute] pub
fn derive_CType (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return compile_error("Unexpected parameter", unexpected_tt.span());
    }
    feed_to_macro_rules(input, parse_quote!(CType))
}
