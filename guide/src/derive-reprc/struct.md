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

### Requirements

  - All the fields must be [`ReprC`] or generic.

      - In the generic case, the struct is [`ReprC`] only when instanced with
        concrete [`ReprC`] types.

  - The struct must be non-empty (because ANSI C does not support empty structs)

## Opaque types (_forward declarations_)

Sometimes you may be dealing with a complex Rust type and you don't want to go
through the hassle of recusrively changing each field to make it [`ReprC`].

In that case, the type can be defined as an _opaque_ object _w.r.t._ the C API,
which will make it usable by C but only through a layer of pointer indirection
and function abstraction:

```rust,noplaypen
#[derive_ReprC]
#[ReprC::opaque] // <-- instead of `#[repr(C)]`
pub
struct ComplicatedStruct {
    path: PathBuf,
    cb: Rc<dyn 'static + Fn(&'_ Path)>,
    x: i32,
}
```

<span class = "warning">

Only braced struct definitions are currently supported. Opaque tuple structs and
`enum`s ought to supported soon.

</span>

<details><summary>Example</summary>

```rust,noplaypen
use ::std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use ::safer_ffi::prelude::*;

#[derive_ReprC]
#[ReprC::opaque]
pub
struct ComplicatedStruct {
    path: PathBuf,
    cb: Rc<dyn 'static + Fn(&'_ Path)>,
    x: i32,
}

#[ffi_export]
fn create ()
  -> repr_c::Box<ComplicatedStruct>
{
    Box::new(ComplicatedStruct {
        path: "/tmp".into(),
        cb: Rc::new(|path| println!("path = `{}`", path.to_string_lossy())),
        x: 42,
    }).into()
}

#[ffi_export]
fn call_and_get_x (it: &'_ ComplicatedStruct)
  -> i32
{
    (it.cb)(&it.path);
    it.x
}

#[ffi_export]
fn destroy (it: repr_c::Box<ComplicatedStruct>)
{
    drop(it)
}
```

<details><summary>Generated C header</summary>

```C
/* Forward declaration */
typedef struct ComplicatedStruct ComplicatedStruct_t;

ComplicatedStruct_t * create (void);

int32_t call_and_get_x (
    ComplicatedStruct_t const * it);

void destroy (
    ComplicatedStruct_t * it);
```

</details>

<br/>

<details><summary>Testing it from C</summary>

```C
#include <assert.h>
#include <stdlib.h>

#include "dem_header.h"

int main (
    int argc,
    char const * const argv[])
{
    ComplicatedStruct_t * it = create();
    assert(call_and_get_x(it) == 42); // Prints 'path = `/tmp`'
    destroy(it);
    return EXIT_SUCCESS;
}
```

</details>

</details>

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
