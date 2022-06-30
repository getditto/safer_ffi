{{#include ../links.md}}

# `src/lib.rs`

### Export items to the FFI world

To do this, simply slap the `#[ffi_export]` attribute on any "item" that you
wish to see exported to C.

The only currently supported such "item"s are:

  - **function definitions** (main entry point of the crate!)
  - `const`s
  - type definitions (when not mentioned by an exported function).

<div class="warning">

`static`s are not supported yet. This ought to be fixed soon.

</div>

If using non-primitive non-`safer_ffi`-provided types, then those must be
[`#[derive_ReprC]` annotated][derive_ReprC].

At which point the only thing remaining is to generate the header file.

### Header generation

```rust ,ignore
// with the `safer-ffi/headers` feature enabled:
::safer_ffi::headers::builder()
    .to_file("….h")?
 // .other_optional_adapters()
    .generate()?
```

<div class="warning">

Given how `safer_ffi` implements the C reflection logic as methods within
[a trait][`CType`] related to [`ReprC`], the only way to generate the headers
is to have that `.generate()` call be written _directly_ in the library
(a limitation that comes from the way the machinery currently operates),
`.generate()` cannot be written in a downstream/dependent crate, such as a `bin`
or an `example`.

</div>

On the other hand, it is perfectly possible to have the Rust library export a
function which does this `.generate()` call.

That's why you'll end up with the following pattern:

Define a `cfg`-gated `pub fn` that calls into the
[`safer_ffi::headers::builder()`] to `.generate()` the headers into the given
file(name), or into the given `Write`-able / "write sink":

  - Basic example:

    ```rust ,norun
    //! src/lib.rs
    #[cfg(feature = "headers")]
    pub fn generate_headers() -> ::std::io::Result<()> {
        ::safer_ffi::headers::builder()
            .to_file("filename.h")?
            .generate()
    }
    ```

    ```rust ,norun
    //! examples/generate-headers.rs
    fn main() -> ::std::io::Result<()> {
        ::crate_name::generate_headers()
    }
    ```

  - And run:

    ```bash
    cargo run --example generate-headers --features headers
    ```

    to generate the headers.

  - You may also    want to add:

    ```toml
    # Cargo.toml
    [[example]]
    name = "generate-headers"
    required-features = ["headers"]
    ```

    to your `Cargo.toml` to improve the error messages when the feature is
    missing.

<details>
<summary>More advanced example (runtime-dependent header output)</summary>

```rust ,norun
#[cfg(feature = "headers")]
fn generate_headers() -> ::std::io::Result<()> {
    let builder = ::safer_ffi::headers::builder();
    if let Some(filename) = ::std::env::args_os().nth(1) {
        builder
            .to_file(&filename)?
            .generate()
    } else {
        builder
            .to_writer(::std::io::stdout())
            .generate()
    }
}
```

and run

```bash
cargo run --example generate-headers --features headers -- /path/to/headers.h
```

</details>
