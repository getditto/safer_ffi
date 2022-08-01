use super::*;

pub(in super)
fn handle (
    _args: parse::Nothing,
    _input: ItemStatic,
) -> Result<TokenStream2>
{
    #[cfg(not(TODO))]
    todo!("`#[ffi_export]`ing a `static`");
}
