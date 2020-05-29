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
    "staticlib",  # to generate a C _static_ library file:
                  # `target/{debug,release}/libcrate_name.a`
    # and / or:
    "cdylib",     # to generate a C _dynamic_ library file,
                  # `target/{debug,release}/libcrate_name.{so,dylib}`
]
```

### `[dependencies.safer_ffi]`

To get access to `safer_ffi` and its ergonomic attribute macros we add `safer_ffi` as
a dependency, and enable the `proc-macros` feature:

```toml
[dependencies]
safer_ffi = { version = "...", features = ["proc-macros"] }
```

  - or instead simply run `cargo add safer_ffi --features proc-macros` if you
    have [`cargo edit`][cargo-edit].

<details><summary>About the <code>proc-macros</code> feature</summary>

although still a WIP, the author of `safer_ffi` is making an important effort
to make the usage of procedural macros be as optional as possible, so
as to allow downstream users to avoid pulling the very heavyweight
`syn + quote` dependencies, by offering alternative basic macros
alternatives (such as `ReprC!` instead of `#[derive_ReprC]`)

</details>

  - If working in a `no_std` environment, you will need to disable the default
    `std` feature by adding `default-features = false`.

      - if, however, you still have access to an allocator, you can enable the
        `alloc` feature, to get the defintions of `safer_ffi::{Box, String, Vec}`
        _etc._

  - You may also enable the `log` feature so that `safer_ffi` may log `error!`s
    when the semi-checked casts from raw C types into their Rust counterparts
    fail (_e.g._, when receiving a `bool` that is nether `0` nor `1`).

### `[features] c-headers`

Finally, in order to alleviate the compile-time when not generating the headers
(it is customary to bundle pre-generated headers when distributing an
FFI-compatible Rust crate), the runtime C reflection and header generation
machinery (the most heavyweight part of `safer_ffi`) is feature-gated away by
default (behind the `safer_ffi/headers` feature).

However, when [generating the headers][header-generation], such machinery is
needed. Thus the simplest solution is for the FFI crate to have a Cargo feature
(flag) that transitively enables the `safer_ffi/headers` feature. You can name such
feature however you want. In this guide, it is named `c-headers`.

```toml
[features]
c-headers = ["safer_ffi/headers"]
```
