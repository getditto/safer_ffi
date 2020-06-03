{{#include ../links.md}}

# Why use `safer_ffi`?

Traditionally, to generate FFI from Rust to C developers would use `#[no_mangle]` and [`cbindgen`] like so:

```rust,noplaypen
#[repr(C)]
pub
struct Point {
    x: i32,
    y: i32,
}

#[no_mangle] pub extern "C"
fn origin () -> Point
{
    Point { x: 0, y: 0 }
}
```

And this is already quite good! For simple FFI projects (_e.g._, exporting just
one or two Rust functions to C), this pattern works wonderfully. So kudos to
[`cbindgen`] authors for such a convenient, customizable and easy to use tool!

But it turns out that this can struggle with more complex scenarios. My
company, [![Ditto][Ditto-logo]][Ditto], extensively uses FFI with Rust
and has run into the limitations outlined below.

[Learn more about Ditto's experience with FFI and Rust.](../ditto/_.md)

## `safer_ffi` features that traditional FFI struggles to support

  - _These have been tested with `cbindgen v0.14.2`._

### Support for complex types and respective layout or ABI semantics

Traditionally, if one were to write the following FFI definition:

```rust,noplaypen
#[no_mangle] pub extern "C"
fn my_free (ptr: Box<i32>)
{
    drop(ptr)
}
```

they would get:

```C
typedef struct Box_i32 Box_i32;

void my_free(Box_i32 ptr);
```

which does not even compile.

This means that the moment [you want to use types to express properties
and invariants][parse-dont-validate], you quickly stumble upon this limitation.
This is why, traditional Rust→C FFI code uses "flat" raw pointers. **This
results in `unsafe` implementations which are more error-prone**.

`::safer_ffi` solves this issue by using more evolved types:

{{#include ../repr_c-types.md}}

<details><summary>Example</summary>

```rust,noplaypen
#[ffi_export]
fn my_free (ptr: repr_c::Box<i32>)
{
    drop(ptr)
}
```

correctly generates

```C
void my_free(int32_t * ptr);
```

</details>

For instance, what better way to guard against `NULL` pointer dereferences than
to express nullability (or lack thereof) with `Option<_>`-wrapped pointer
types?

<details><summary>Example</summary>

```rust,noplaypen
#[ffi_export]
fn my_free_supports_null (ptr: Option<repr_c::Box<i32>>)
{
    drop(ptr)
}
```

</details>

### Consistent support for macro-generated definitions

Since `safer_ffi` is integrated within the compiler, it supports macros expanding
to `#[ffi_export]` function definitions or `#[derive_ReprC]` type definitions.

<details><summary>Example</summary>

To make the following code work (_w.r.t._ auto-generated headers):

```rust,noplaypen
macro_rules! adders {(
    $(
        $T:ty => $add_T:ident,
    )*
) => (
    $(
        #[no_mangle] pub extern "C"
        fn $add_T (x: $T, y: $T) -> $T
        {
            x.wrapping_add(y)
        }
    )*
)}

adders! {
    u8  => add_uint8,
    i8  => add_int8,
    u16 => add_uint16,
    i16 => add_int16,
    u32 => add_uint32,
    i32 => add_int32,
    u64 => add_uint64,
    i64 => add_int64,
}
```

one only has to:

```diff
-       #[no_mangle] pub extern "C"
+       #[ffi_export]
```

</details>

### Support for shadowed paths

Since `safer_ffi` is integrated withing the compiler, the types the code refers to
are unambiguously understood by both `#[derive_ReprC]` and `#[ffi_export]`.

<details><summary>Example</summary>

The following examples confuses traditional FFI:

```rust,noplaypen
/// Let's imagine that we have a custom `Option` type with a defined C layout.
/// We are opting out of a niche layout optimization.
/// (https://rust-lang.github.io/unsafe-code-guidelines/glossary.html#niche)
#[repr(C)]
pub
struct Option<T> {
    is_some: bool,
    value: ::core::mem::MaybeUninit<T>,
}

mod ffi_functions {
    use super::*; // <- This is what `cbindgen` currently struggles with

    #[no_mangle] pub extern "C"
    fn with_my_option (my_opt: Option<&'_ i32>) -> i8
    {
        if my_opt.is_some {
            let value: &'_ i32 = unsafe { my_opt.value.assume_init() };
            println!("{}", *value);
            0
        } else {
            -1
        }
    }
}
```

Indeed, it generates:

```C
void with_my_option(const int32_t *_it);
```

Which corresponds to the signature of a function using the standard `Option`
type: ⚠️ an incorrect function signature, with no warnings whatsoever ⚠️

</details>

This is another instance where

```diff
-   #[no_mangle] pub extern "C"
+   #[ffi_export]
```

saves the day 🙂
