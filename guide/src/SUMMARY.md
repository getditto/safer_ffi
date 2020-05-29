# Summary

[Introduction](introduction/_.md)

  - [Motivation](motivation/_.md)

    - [Rust→C FFI using `cbindgen`](motivation/cbindgen.md)

    - [Defined layout for Rust's pervasive types](motivation/repr-c-forall.md)

  - [Prerequisites](prerequisites.md)

  - [Usage](usage/_.md)

  - [`#[ffi_export]`](ffi-export/_.md)

  - [`#[derive_ReprC]`](derive-reprc/_.md)

      - [On a `struct`](derive-reprc/struct.md)

      - [On an `enum`](derive-reprc/enum.md)

  - [Example: Real-world use case at Ditto](example-ditto/_.md)

  - [Example: our own `hashmap` in C](example-hashmap/_.md)
