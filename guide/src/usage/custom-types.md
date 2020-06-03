{{#include ../links.md }}

# Custom Types

Custom types are also supported, as long as they:

  - have a defined C layout;

  - have a [`#[derive_ReprC]`][derive_ReprC] attribute.

### Usage with structs

```rust,noplaypen
#[derive_ReprC] // <- `::safer_ffi`'s attribute
#[repr(C)]      // <- defined C layout is mandatory!
struct Point {
    x: i32,
    y: i32,
}
```

  - See [the dedicated chapter on structs][derive_ReprC-struct] for more info.

### Usage with enums

```rust,noplaypen
#[derive_ReprC] // <- `::safer_ffi`'s attribute
#[repr(u8)]     // <- explicit integer `repr` is mandatory!
pub enum Direction {
    Up = 1,
    Down = -1,
}
```

  - See [the dedicated chapter on enums][derive_ReprC-enum] for more info.
