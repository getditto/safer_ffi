# Simple examples: `string_concat`

```rust,noplaypen
#![deny(unsafe_code)] /* No `unsafe` needed! */

use ::safer_ffi::prelude::*;

/// Concatenate two input UTF-8 (_e.g._, ASCII) strings.
///
/// \remark The returned string must be freed with `rust_free_string`
#[ffi_export]
fn concat (fst: char_p::Ref<'_>, snd: char_p::Ref<'_>)
  -> char_p::Box
{
   let fst = fst.to_str(); // : &'_ str
   let snd = snd.to_str(); // : &'_ str
   format!("{}{}", fst, snd) // -------+
      .try_into() //                   |
      .unwrap() // <- no inner nulls --+
}

/// Frees a Rust-allocated string.
#[ffi_export]
fn rust_free_string (string: char_p::Box)
{
    drop(string)
}
```

<details><summary>generates</summary>

```C
/* \brief
 * Concatenate two input UTF-8 (_e.g._, ASCII) strings.
 *
 * \remark The returned string must be freed with `rust_free_string`.
 */
char * concat (
    char const * fst,
    char const * snd);

/* \brief
 * Frees a Rust-allocated string.
 */
void free_string (
    char * string);
```

</details>
