{{#include ../links.md}}

# Deriving `ReprC` for custom structs


## Usage

```rust,noplaypen
use ::safer_ffi::prelude::*;

#[derive_ReprC] // <- `::safer_ffi`'s attribute
#[repr(C)]      // <- defined C layout is mandatory!
pub
struct Point {
    x: i32,
    y: i32,
}

#[ffi_export]
fn get_origin ()
  -> Point
{
    Point { x: 0, y: 0 }
}
```

<details><summary>Generated C header</summary>

```c
typedef struct Point {
    int32_t x;
    int32_t y;
} Point_t;

Point_t get_origin (void);
```

</details>

### Usage with Generic Structs

`#[derive_ReprC]` supports generic structs:

```rust,noplaypen
use ::safer_ffi::prelude::*;

/// The struct can be generic...
#[derive_ReprC]
#[repr(C)]
pub
struct Point<Coordinate> {
    x: Coordinate,
    y: Coordinate,
}

/// ... but its usage within an `#[ffi_export]`-ed function must
/// no longer be generic (it must have been instanced with a concrete type)
#[ffi_export]
fn get_origin ()
  -> Point<i32>
{
    Point { x: 0, y: 0 }
}
```

<details><summary>Generated C header</summary>

Each monomorphization leads to its own C definition:

  - **`Point<i32>`**

    ```C
    typedef struct {
        int32_t x;
        int32_t y;
    } Point_int32_t;
    ```

  - **`Point<f64>`**

    ```C
    typedef struct {
        double x;
        double y;
    } Point_double_t;
    ```
</details>

## Requirements

  - All the fields must be [`ReprC`] or generic.

      - In the generic case, the struct is [`ReprC`] only when it is instanced
        with concrete [`ReprC`] types.

  - The struct must be non-empty (because ANSI C does not support empty structs)

## Going further

<details><summary>Transparent newtype wrapper</summary>

```rust,noplaypen
use ::safer_ffi::{prelude::*, ptr};

/// A `Box`-like owned pointer type, but which can be freed using `free()`.
#[derive_ReprC]
#[repr(transparent)]
pub struct Malloc<T>(ptr::NonNullOwned<T>);

impl<T> Malloc<T> {
    pub fn new(value: T) -> Option<Self> {
        /* Uses `posix_memalign()` to handle the allocation */
    }
}
```

This pattern allows you to define a new type with thus specific Rust semantics
attached to it (_e.g._, specific constructor, destructor and methods) while
hiding all that to the C side:

  - in the C world, `Malloc<T>` will be referred to in the same way that
    `ptr::NonNullOwned<T>` is, _i.e._, as a (non-nullable) `*mut T`.

<details><summary>Example</summary>

```rust,noplaypen
#[ffi_export]
fn new_int (x: i32)
  -> Option<Malloc<i32>>
{
    Malloc::new(x)
}
```

would then generate:

```C
int32_t * new_int (
    int32_t x);
```

</details>

</details>
