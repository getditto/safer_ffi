{{#include ../links.md}}

# `Cargo.toml`

### `[lib] crate-type`

So, to ensure we compile a [static or dynamic library][c-compilation]
containing the definitions of our `#[ffi_export]` functions (+ any code they
transitively depend on), we need to tell to `cargo` that our crate is of that
type:

```toml
# Cargo.toml

[lib]
crate-type = [
    "staticlib",  # Ensure it gets compiled as a (static) C library
                  # `target/{debug,release}/libcrate_name.a`
    # and/or:
    "cdylib",     # If you want a shared/dynamic C library (advanced)
                  # `target/{debug,release}/libcrate_name.{so,dylib}`

    "lib",        # For downstream Rust dependents: `examples/`, `tests/` etc.
]
```

### `[dependencies.safer_ffi]`

To get access to `safer_ffi` and its ergonomic attribute macros we add `safer_ffi` as
a dependency, and enable the `proc_macros` feature:

```toml
[dependencies]
safer-ffi.version = "x.y.z"
```

  - Where `"x.y.z"` ought to be replaced by the last released version, which you
    can find by running `cargo search safer-ffi` or `cargo add safer-ffi`

  - If working in a `no_std` environment, you will need to disable the default
    `std` feature by adding `default-features = false`.

    ```toml
    [dependencies]
    safer-ffi.version = "x.y.z"
    safer-ffi.default-features = false  # <- Add this!
    ```


      - if, however, you still have access to an allocator, you can enable the
        `alloc` feature, to get the defintions of `safer_ffi::{Box, String, Vec}`
        _etc._

        ```toml
        [dependencies]
        safer-ffi.version = "x.y.z"
        safer-ffi.default-features = false
        safer-ffi.features = [
            "alloc",  # <- Add this!
        ]
        ```

  - You may also enable the `log` feature so that `safer_ffi` may log `error!`s
    when the semi-checked casts from raw C types into their Rust counterparts
    fail (_e.g._, when receiving a `bool` that is nether `0` nor `1`).

    ```toml
    [dependencies]
    safer-ffi.version = "x.y.z"
    safer-ffi.features = [
        "log",  # <- Add this!
    ]
    ```

### `[features] headers`

Finally, in order to alleviate the compile-time when not generating the headers
(it is customary to bundle pre-generated headers when distributing an
FFI-compatible Rust crate), the runtime C reflection and header generation
machinery (the most heavyweight part of `safer_ffi`) is feature-gated away by
default (behind the `safer_ffi/headers` feature).

However, when [generating the headers][header-generation], such machinery is
needed. Thus the simplest solution is for the FFI crate to have a Cargo feature
(flag) that transitively enables the `safer_ffi/headers` feature. You can name
such feature however you want. In this guide, it is named `headers`.

```toml
[features]
headers = ["safer_ffi/headers"]
```
