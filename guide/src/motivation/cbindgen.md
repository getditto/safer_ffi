{{#include ../links.md}}

# Why not `cbindgen`?

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

But it turns out that [`cbindgen`] can struggle with more complex scenarios. My, [Ditto](https://www.ditto.live) extensively uses FFI with Rust and regularly run into `cbindgen` limitations outlined below. 

[Learn more about Ditto's experience with FFI and Rust.](../ditto/_.md)


## Limitations of `cbindgen`

_These pertain to (`cbindgen v0.14.2`)._

### No support for complex types and respective layout or ABI semantics 

The following code:

```rust,noplaypen
#[no_mangle] pub extern "C"
fn my_free (ptr: Box<i32>)
{
    drop(ptr)
}
```

generates:

```C
typedef struct Box_i32 Box_i32;

void my_free(Box_i32 ptr);
```

but what we expect is:

```C
void my_free(int32_t * ptr);
```


This means that the moment [you want to use types to express properties
and invariants](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/
), you quickly stumble upon this limitation. This means 
that all Rust→C FFI code uses "flat" raw
pointers. The side effect results in `unsafe` implementations
which are more error-prone. How can we be sure that all the pointer
dereferences have been guarded against `NULL` cases unless we wrap them in`Option<_>`?

### No consistent support for complex procedural macros:

The moment the signatures get more involved like with the usage of procedural macros, `cbingen` will either fail to generate FFI.

The following:

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

⚠️ has no output with [`cbindgen`]. ⚠️

### No warnings or errors while generating incorrect FFI:

In this example, [`cbindgen`] fails to generate correct FFI without any warnings or errors:

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
        
generates the following with no warnings or errors:

```C
void with_my_option(const int32_t *_it);
```

⚠️ Unfortunately, this generated code is an incorrect function signature. ⚠️
