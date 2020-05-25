# Summary

  - [Introduction](introduction/_.md)

      - [Getting Started](introduction/getting_started.md)

  - [Motivation: safer types across FFI](motivation/_.md)

    - [The limits of traditional FFI](motivation/traditional-ffi.md)

    - [Defined layout for Rust's pervasive types](motivation/repr-c-forall.md)

  - [Simple examples](simple-examples/_.md)

    - [`string_concat`](simple-examples/string_concat.md)

    - [Maximum member of an array](simple-examples/max.md)

  - [Detailed usage](usage/_.md)

    - [`Cargo.toml`](usage/cargo-toml.md)

    - [`src/lib.rs` and header generation](usage/lib-rs.md)

  - [`ReprC` and `#[derive_ReprC]`](derive-reprc/_.md)

      - [On a `struct`](derive-reprc/struct.md)

      - [On an `enum`](derive-reprc/enum.md)

  - [`#[ffi_export]`](ffi-export/_.md)

      - [Auto-generated checks](ffi-export/sanity-checks.md)

      - [Attributes](ffi-export/attributes.md)

[Example: Real-world use case at Ditto](example-ditto/_.md)

[Example: our own `hashmap` in C](example-hashmap/_.md)

[Appendix: FFI and C compilation](appendix/c-compilation.md)

[Appendix: how does `repr_c` work](appendix/how-does-repr_c-work.md)
