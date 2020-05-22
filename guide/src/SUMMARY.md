# Summary

[Introduction](introduction/_.md)

  - [Motivation](motivation/_.md)

    - [Rust→C FFI using `cbindgen`](motivation/cbindgen.md)

    - [Defined layout for Rust's pervasive types](motivation/repr-c-forall.md)

  - [Prerequisites](prerequisites.md)

  - [Usage](usage/_.md)

  - [`#[ffi_export]`](ffi-export/_.md)

  - [`#[derive_ReprC]`](derive-reprc/_.md)

      - [`#[repr(C)] struct`](derive-reprc/repr-c-struct.md)

      - [`#[repr({int})] enum`](derive-reprc/repr-int-enum.md)

  - [Example: our own `hashmap` in C](hashmap-example/_.md)

  - [Real-world example at Ditto](ditto/_.md)
