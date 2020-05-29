{{#include ../links.md}}

# `src/lib.rs`

### Export items to the FFI world

To do this, simply slap the `#[ffi_export]` attribute on any "item" that you
wish to see exported to C.

<div class="warning">

The only currently supported such "item"s are function definitions: `const` and
`static`s are not supported yet. This ought to be fixed soon.

</div>

If using non-primitive non-`safer_ffi`-provided types, then those must be
[`#[derive_ReprC]` annotated][derive_ReprC].

At which point the only thing remaining is to generate the header file.

### Header generation

Given how `safer_ffi` implements the C reflection logic as methods within
[a trait][`CType`] related to [`ReprC`], the only way to generate the headers
is to be a "Rust downstream" user of the library, within the same compilation
unit / crate (a limitation that comes from the way the machinery currently
operates). That is, a **unit test**.

So you need to define a `cfg`-gated unit test that calls into the
[`safer_ffi::headers::builder()`] to `.generate()` the headers into the given
file(name), or into the given `Write`-able / "write sink":

  - Basic example:

    ```rust,noplaypen
    #[::safer_ffi::cfg_headers]
    #[test]
    fn generate_headers () -> ::std::io::Result<()>
    {
        ::safer_ffi::headers::builder()
            .to_file("filename.h")?
            .generate()
    }
    ```

  - And run:

    ```bash
    cargo test --features c-headers -- generate_headers --nocapture
    ```

    to generate the headers.

<details>
<summary>More advanced example (runtime-dependent header output)</summary>

```rust,noplaypen
#[::safer_ffi::cfg_headers]
#[test]
fn generate_headers () -> ::std::io::Result<()>
{
    let builder = ::safer_ffi::headers::builder();
    if let Ok(filename) = ::std::env::var("HEADERS_FILE") {
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
HEADERS_FILE=/path/to/headers.h \
cargo test --features c-headers -- generate_headers --nocapture
```

</details>
