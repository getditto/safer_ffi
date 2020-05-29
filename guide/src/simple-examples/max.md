# Simple examples: maximum member of an array

```rust,noplaypen
#![deny(unsafe_code)] /* No `unsafe` needed! */

use ::safer_ffi::prelude::*;

/// Returns a pointer to the maximum element of the slice
/// when it is not empty, and `NULL` otherwise.
#[ffi_export]
fn max<'xs> (xs: c_slice::Ref<'xs, i32>)
  -> Option<&'xs i32>
{
    xs  .as_slice() // : &'xs [i32]
        .iter()
        .max()
}
```

<details><summary>generates</summary>
something along the lines of:

```C
#include <stddef.h>
#include <stdint.h>

typedef struct slice_ref_int32 {
    /* \brief
     * Pointer to the first element (if any).
     *
     * \remark Cannot be NULL.
     */
    int32_t const * ptr;

    /* \brief
     * Number of elements.
     */
    size_t len;
} slice_ref_int32_t;

/* \brief
 * Returns a pointer to the maximum element of the slice
 * when it is not empty, and `NULL` otherwise.
 */
int32_t const * max (
    slice_ref_int32_t xs);
```

</details>
