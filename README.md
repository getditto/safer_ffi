# `::repr_c`

# ⚠️ WIP ⚠️

This is currently still being developed and at an experimental stage, hence its not being published to crates.io yet.

## Quickstart

### Setup

Edit your `Cargo.toml` like so:

### Code example

```rust
//! `src/lib.rs`
use ::repr_c::prelude::*;

#[derive_ReprC] // <- `::repr_c`'s attribute
#[repr(C)]      // <- defined C layout is mandatory!
#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[ffi_export]
fn mid_point (
    left: Point,
    right: Point,
) -> Point
{
   Point {
      x: (left.x + right.x) / 2,
      y: (left.y + right.y) / 2,
   }
}

#[ffi_export]
fn print_point (point: &Point)
{
    println!("{:?}", point);
}
```

### Compilation & header generation

```shell
# Compile the C library
cargo build...
# Generate the C header
cargo tes...
```

<details><summary>Generated C header</summary>

```C
#ifndef ...
```

<details>

### Testing it

```C
#include "rust_points.h"

int main (void)
{
    Point a = { .x = 84, .y = 45 };
    Point b = { .x = 0, .y = 39 };
    Point m = mid_point(a, b);
    print_point(&m);
}
```

then run

```bash
cc main.c ./target/debug/libexample.a -o main && ./main
```

which outputs:

```text
Point {
    x: 42.,
    y: 42.,
}
```
