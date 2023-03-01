# Summary

  - [Introduction](introduction/_.md)

      - [Quickstart](introduction/quickstart.md)

  - [Detailed usage](usage/_.md)

    - [&lt;code&gt;Cargo.toml&lt;/code&gt;](usage/cargo-toml.md)

    - [&lt;code&gt;src/lib.rs&lt;/code&gt; and header generation](usage/lib-rs.md)

    - [Custom types](usage/custom-types.md)

  - [Motivation: safer types across FFI](motivation/_.md)

    - [The limits of traditional FFI](motivation/traditional-ffi.md)

    - [Defined layout for Rust's pervasive types](motivation/repr-c-forall.md)

  - [Simple examples](simple-examples/_.md)

    - [&lt;code&gt;string_concat&lt;/code&gt;](simple-examples/string_concat.md)

    - [Maximum member of an array](simple-examples/max.md)

  - [&lt;code&gt;ReprC&lt;/code&gt; and &lt;code&gt;#[derive_ReprC]&lt;/code&gt;](derive-reprc/_.md)

      - [On a &lt;code&gt;struct&lt;/code&gt;](derive-reprc/struct.md)

      - [On an &lt;code&gt;enum&lt;/code&gt;](derive-reprc/enum.md)

  - [&lt;code&gt;#[ffi_export]&lt;/code&gt;](ffi-export/_.md)

      - [Auto-generated checks](ffi-export/sanity-checks.md)

      - [Attributes](ffi-export/attributes.md)

  - [Callbacks](callbacks/_.md)

      - [Function pointers](callbacks/fn_pointers.md)

      - [Closures](callbacks/closures.md)

  - [&lt;code&gt;dyn Trait&lt;/code&gt;s / Virtual objects](dyn_traits/_.md)

      - [&lt;code&gt;VirtualPtr&amp;lt;dyn Trait&amp;gt;&lt;/code&gt;](dyn_traits/virtual_ptr.md)

      - [&lt;code&gt;#[derive_ReprC(dyn, …)]&lt;/code&gt;](dyn_traits/derive_reprc_dyn.md)

      - [Example: FFI-safe &lt;code&gt;Future&lt;/code&gt;s and executors](dyn_traits/futures.md)

  - [Example: Real-world use case at Ditto](example-ditto/_.md)

  - [Example: our own &lt;code&gt;hashmap&lt;/code&gt; in C](example-hashmap/_.md)

[Appendix: FFI and C compilation](appendix/c-compilation.md)

[Appendix: how does &lt;code&gt;safer_ffi&lt;/code&gt; work](appendix/how-does-safer_ffi-work.md)
