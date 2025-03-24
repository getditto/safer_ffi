use super::*;

pub(in crate)
fn c_str (
    input: TokenStream2,
) -> Result<TokenStream2>
{
    let input: LitStr =
        if let Some(it) = parse2(input)? {
            it
        } else {
            return Ok(::quote::quote!(
                ::safer_ffi::char_p::char_p_ref::EMPTY
            ));
        }
    ;
    let bytes = input.value();
    let mut bytes = bytes.as_bytes();
    let mut v;
    match bytes.iter().position(|&b| b == b'\0') {
        | None => {
            v = Vec::with_capacity(bytes.len() + 1);
            v.extend_from_slice(bytes);
            v.push(b'\0');
            bytes = &v[..];
        },
        | Some(n) if n == bytes.len() - 1 => {},
        | Some(bad_idx) => {
            return Err(Error::new_spanned(input, &format!(
                "Error, encountered inner nul byte at position {}", bad_idx,
            )));
        },
    }
    let byte_str = LitByteStr::new(bytes, input.span());
    Ok(::quote::quote!(
        unsafe {
            const STATIC_BYTES: &'static [u8] = #byte_str;
            ::safer_ffi::char_p::char_p_ref::from_ptr_unchecked(
                ::safer_ffi::ptr::NonNull::new_unchecked(STATIC_BYTES.as_ptr() as _)
            )
        }
    ))
}
