<span style="text-align: center;">

![safer-ffi-banner](
https://github.com/getditto/safer_ffi/blob/banner/guide/assets/safer_ffi.jpg?raw=true)

[![CI](
https://github.com/getditto/safer_ffi/workflows/CI/badge.svg?branch=master)](
https://github.com/getditto/safer_ffi/actions)
[![guide](https://img.shields.io/badge/guide-mdbook-blue)](
https://getditto.github.io/safer_ffi)
[![docs-rs](https://docs.rs/safer-ffi/badge.svg)](
https://getditto.github.io/safer_ffi/rustdoc/safer_ffi)
[![crates-io](https://img.shields.io/crates/v/safer-ffi.svg)](
https://crates.io/crates/safer-ffi)
[![repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/getditto/safer_ffi)

</span>

# What is `safer_ffi`?

`safer_ffi` is a framework that helps you write foreign function interfaces (FFI) without polluting your Rust code with `unsafe { ... }` code blocks while making functions far easier to read and maintain.

> <strong style="font-size: x-large;">[📚 Read The User Guide 📚][user guide]</strong>

[user guide]: https://getditto.github.io/safer_ffi

## Prerequisites

Minimum Supported Rust Version: `1.66.1`

# Quickstart

<details open><summary>Click to hide</summary>

#### Small self-contained demo

You may try working with the `examples/point` example embedded in the repo:

```bash
git clone https://github.com/getditto/safer_ffi && cd safer_ffi
(cd examples/point && make)
```

Otherwise, to start using `::safer_ffi`, follow the following steps:

### Crate layout

#### Step 1: `Cargo.toml`

Edit your `Cargo.toml` like so:

```toml
[package]
name = "crate_name"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = [
    "staticlib",  # Ensure it gets compiled as a (static) C library
  # "cdylib",     # If you want a shared/dynamic C library (advanced)
    "lib",        # For `generate-headers` and other downstream rust dependents
                  # such as integration `tests/`, doctests, and `examples/`
]

[[bin]]
name = "generate-headers"
required-features = ["headers"]  # Do not build unless generating headers.

[dependencies]
# Use `cargo add` or `cargo search` to find the latest values of x.y.z.
# For instance:
#   cargo add safer-ffi
safer-ffi.version = "x.y.z"
safer-ffi.features = [] # you may add some later on.

[features]
# If you want to generate the headers, use a feature-gate
# to opt into doing so:
headers = ["safer-ffi/headers"]
```

  - Where `"x.y.z"` ought to be replaced by the last released version, which you
    can find by running `cargo search safer-ffi`.

  - See the [dedicated chapter on `Cargo.toml`][cargo-toml] for more info.

#### Step 2: `src/lib.rs`

Then, to export a Rust function to FFI, add the
[`#[derive_ReprC]`][derive_ReprC] and [`#[ffi_export]`][ffi_export] attributes
like so:

```rust ,no_run
use ::safer_ffi::prelude::*;

/// A `struct` usable from both Rust and C
#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: f64,
    y: f64,
}

/* Export a Rust function to the C world. */
/// Returns the middle point of `[a, b]`.
#[ffi_export]
fn mid_point(a: &Point, b: &Point) -> Point {
    Point {
        x: (a.x + b.x) / 2.,
        y: (a.y + b.y) / 2.,
    }
}

/// Pretty-prints a point using Rust's formatting logic.
#[ffi_export]
fn print_point(point: &Point) {
    println!("{:?}", point);
}

// The following function is only necessary for the header generation.
#[cfg(feature = "headers")] // c.f. the `Cargo.toml` section
pub fn generate_headers() -> ::std::io::Result<()> {
    ::safer_ffi::headers::builder()
        .to_file("rust_points.h")?
        .generate()
}
```

  - See [the dedicated chapter on `src/lib.rs`][lib-rs] for more info.

#### Step 3: `src/bin/generate-headers.rs`

```rust ,ignore
fn main() -> ::std::io::Result<()> {
    ::crate_name::generate_headers()
}
```

### Compilation & header generation

```bash
# Compile the C library (in `target/{debug,release}/libcrate_name.ext`)
cargo build # --release

# Generate the C header
cargo run --features headers --bin generate-headers
```

  - See [the dedicated chapter on header generation][header-generation] for
    more info.

<details><summary>Generated C header (<code>rust_points.h</code>)</summary>

```C
/*! \file */
/*******************************************
 *                                         *
 *  File auto-generated by `::safer_ffi`.  *
 *                                         *
 *  Do not manually edit this file.        *
 *                                         *
 *******************************************/

#ifndef __RUST_CRATE_NAME__
#define __RUST_CRATE_NAME__
#ifdef __cplusplus
extern "C" {
#endif


#include <stddef.h>
#include <stdint.h>

/** \brief
 *  A `struct` usable from both Rust and C
 */
typedef struct Point {
    /** <No documentation available> */
    double x;

    /** <No documentation available> */
    double y;
} Point_t;

/** \brief
 *  Returns the middle point of `[a, b]`.
 */
Point_t
mid_point (
    Point_t const * a,
    Point_t const * b);

/** \brief
 *  Pretty-prints a point using Rust's formatting logic.
 */
void
print_point (
    Point_t const * point);


#ifdef __cplusplus
} /* extern \"C\" */
#endif

#endif /* __RUST_CRATE_NAME__ */
```

___

</details>

## Testing it from C

Here is a basic example to showcase FFI calling into our exported Rust
functions:

### `main.c`

```C
#include <stdlib.h>

#include "rust_points.h"

int
main (int argc, char const * const argv[])
{
    Point_t a = { .x = 84, .y = 45 };
    Point_t b = { .x = 0, .y = 39 };
    Point_t m = mid_point(&a, &b);
    print_point(&m);
    return EXIT_SUCCESS;
}
```

### Compilation command

```bash
cc -o main{,.c} -L target/debug -l crate_name -l{pthread,dl,m}

# Now feel free to run the compiled binary
./main
```

  - <details><summary>Note regarding the extra <code>-l…</code> flags.</summary>

    Those vary based on the version of the Rust standard library being used, and
    the system being used to compile it. In order to reliably know which ones to
    use, `rustc` itself ought to be queried for it.

    Simple command:

    ```bash
    rustc --crate-type=staticlib --print=native-static-libs -</dev/null
    ```

    this yields, _to the stderr_, output along the lines of:

    ```text
    note: Link against the following native artifacts when linking against this static library. The order and any duplication can be significant on some platforms.

    note: native-static-libs: -lSystem -lresolv -lc -lm -liconv
    ```

    Using something like `sed -nE 's/^note: native-static-libs: (.*)/\1/p'` is
    thus a convenient way to extract these flags:

    ```bash
    rustc --crate-type=staticlib --print=native-static-libs -</dev/null \
        2>&1 | sed -nE 's/^note: native-static-libs: (.*)/\1/p'
    ```

    Ideally, you would not query for this information _in a vacuum_ (_e.g._,
    `/dev/null` file being used as input Rust code just above), and rather,
    would apply it for your actual code being compiled:

    ```bash
    cargo rustc -q -- --print=native-static-libs \
        2>&1 | sed -nE 's/^note: native-static-libs: (.*)/\1/p'
    ```

    And if you really wanted to polish things further, you could use the
    JSON-formatted compiler output (this, for instance, avoids having to
    redirect `stderr`). But then you'd have to use a JSON parser, such as `jq`:

    ```bash
    RUST_STDLIB_DEPS=$(set -eo pipefail && \
        cargo rustc \
            --message-format=json \
            -- --print=native-static-libs \
        | jq -r '
            select (.reason == "compiler-message")
            | .message.message
        ' | sed -nE 's/^native-static-libs: (.*)/\1/p' \
    )
    ```

    and then use:

    ```bash
    cc -o main{,.c} -L target/debug -l crate_name ${RUST_STDLIB_DEPS}
    ```

    </details>

which does output:

```text
Point { x: 42.0, y: 42.0 }
```

🚀🚀

[callbacks]: https://getditto.github.io/safer_ffi/callbacks/_.html
[cargo-toml]: https://getditto.github.io/safer_ffi/usage/cargo-toml.html
[ffi_export]: https://getditto.github.io/safer_ffi/ffi-export/_.html
[header-generation]: https://getditto.github.io/safer_ffi/usage/lib-rs.html#header-generation
[derive_ReprC]: https://getditto.github.io/safer_ffi/derive-reprc/_.html
[lib-rs]: https://getditto.github.io/safer_ffi/usage/lib-rs.html

</details>

## Development

<details><summary>Click to see</summary>

To test the code with a certain amount of FFI integration baked into the tests (since `safer-ffi`,
alone, only _exports_ APIs to the FFI, so doesn't come with FFI callsites on its own), the
`ffi_tests/` project directory is used to test against C, C#, and Lua callsites.

You can run these tests directly by doing:

```bash
make -C ffi_tests
```

or by adding `--features ffi-tests` to the `cargo test` command.

### Formatting

Code is formatted using the "`{MSRV}-nightly` toolchain". That is, formatting uses some `unstable`
features of `rustfmt`, so we use a Versioned Nightly™ approach. For the sake of version consistency,
we stick to that of our MSRV.

But since it needs to be a genuine `nightly` toolchain, we are forced to pick an actual `nightly`
toolchain, only one whose date matches the birth of the corresponding `MSRV`-stable toolchain.

  - See [`./scripts/formatting/rust-toolchain.toml`](./scripts/formatting/rust-toolchain.toml).
  - See also, w.r.t. versions and dates: <https://releases.rs>

To format the code, you have three options:

  - `./scripts/format.sh`

  - `cargo fmt-nightly` (which is defined as an alias of the previous bullet)

  - `cargo fmt` but only after having run `./scripts/formatting/setup_cargo_fmt_override.sh` at
    least once.

    This does mutate a bit your `rustup` setup, but in an unobservable way (but for allowing the
    matching stable toolchain to use `unstable` features when running `cargo fmt`, the very point
    of this maneuver).

    This is probably the preferred approach for those with IDEs or whatnot which automagically runs
    `cargo fmt`/`rustfmt` on save.

### Dependencies

  - #### Lua dependencies

    For running the C# FFI integration tests please install `dotnet` (v8.0) dependency:

    See <https://aka.ms/dotnet-download> for guidance about this.

  - #### Lua dependencies

    For running Lua FFI integration tests please install `luajit` dependency:

    MacOS:
    ```bash
    brew install luajit
    ```

    Ubuntu/Debian:
    ```bash
    sudo apt-get install -y luajit
    ```

### Various integration test suites

safer-ffi includes three different tests suites that can be run.

```bash
# In the project root:
cargo test

# FFI tests

make -C ffi_tests

# JavaScript tests

make -C js_tests

# Running the JS tests also gives you instructions for running browser tests.
# Run this command in the `js_tests` directory, open a browser and navigate to
# http://localhost:13337/
wasm-pack build --target web && python3 -m http.server 13337

```

</details>
