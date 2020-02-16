# `::repr_c`

A crate to expose `#[repr(C)]` (and thus FFI-compatible) types equivalent to
the standard library's, such as:

  - `RefSlice<'lt, T>`, a `&'lt [T]` whose layout in C is guaranteed to be
    `struct { T const * ptr; size_t len; }` (but with `ptr != NULL`: wrap it in an `Option` if you want to support the `NULL` case).

  - `MutSlice<'lt, T>`, a `&'lt mut [T]` whose layout in C is guaranteed to be
    `struct { T * ptr; size_t len; }` (but with `ptr != NULL`: wrap it in an   `Option` if you want to support the `NULL` case).

  - `BoxedSlice<T>`, a `Box<[T]>` whose layout in C is guaranteed to be
    `struct { T * ptr; size_t len; }` (but with `ptr != NULL`: wrap it in an   `Option` if you want to support the `NULL` case).

# ⚠️ WIP ⚠️

This is currently still being developed and in an experimental stage, hence its not being published to crates.io yet.
