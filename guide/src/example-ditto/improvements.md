{{#include ../links.md}}

<span class = "warning">

TODO: better reorganize this section

</span>

# How `::safer_ffi` improved our FFI

[![Code before and code after using safer_ffi][before-and-after]][before-and-after]

### Interesting stat: evolution of the number of `unsafe` blocks

#### Old

> 282

#### New

> 48

That is, **a whopping 83% decrease!**

  - <small>(by the way, we should be able to get rid of these remaining `unsafe`
    blocks once support for lifetimes in [callback][callbacks] signatures is
    added).</small>

## Returning a Vec when doing FFI

Many people that start doing Rust FFI stumble upon the same question:

> How can I return a Vec when doing Rust FFI?

And the answer is now pretty simple: use `::safer_ffi`'s [`repr_c::Vec`]!

### Real-life example

Take, for instance, our implementation of a function serializing one of our
objects to the CBOR format, thus returning a heap-allocated slice of bytes.

<div class="code_blocks_wrap" style = "display: flex; max-width: 100%;">
<style scoped>
    .code_blocks_wrap pre {
        white-space: pre-wrap;       /* css-3 */
        white-space: -moz-pre-wrap;  /* Mozilla, since 1999 */
        white-space: -pre-wrap;      /* Opera 4-6 */
        white-space: -o-pre-wrap;    /* Opera 7 */
        word-wrap: break-word;       /* Internet Explorer 5.5+ */
        margin: 5px;
    }
</style>
<div style = "flex: 1;">

<h4 style = "text-align: center;">Old</h4>

```rust,noplaypen
#[require_unsafe_in_body]
#[no_mangle]
pub
unsafe extern "C"
fn ditto_document_cbor (
    document: *const Document,
    out_cbor_len: *mut usize,
) -> *const c_uchar
{
    let value =
        unsafe { &*document }
            .to_value()
    ;
    let cbor_bytes: Box<[u8]> =
        ::serde_cbor::to_vec(&value)
            .unwrap()
            .into_boxed_slice()
    ;
    unsafe {
        *out_cbor_len = cbor_bytes.len();
    }
    Box::into_raw(cbor_bytes)
        as *const _
}
```

</div>

<div style = "flex: 1;">

<h4 style = "text-align: center;">New</h4>

```rust,noplaypen
#[ffi_export]
pub
fn ditto_document_cbor (
    document: &'_ Document,
) -> c_slice::Box<u8>
{
    let value = document.to_value();
    let cbor_bytes: Box<[u8]> =
        ::serde_cbor::to_vec(&value)
            .unwrap()
            .into_boxed_slice()
    ;
    cbor_bytes
        .into()
}
```

</div></div>

  - Notice how
      - an `unsafe fn`,
      - with _two_ `unsafe { ... }` blocks in it,
      - and an unannotated pointer cast,

    are just _gone_.

  - Also notice how the returned slice had one field returned as an out
    parameter, and another one as a return value (which, in the long run, is
    error-prone: we may return the pointer and forget to update the `len`
    field).

    Now these two are _bundled together_.

<details>

<summary>
Aside: <code>Box&lt;[_]&gt;</code> vs. <code>Vec&lt;_&gt;</code>
</summary>

In the example above, a micro-optimisation has been performed whereby the
obtained `Vec<u8>` is converted to a `Box<[u8]>`, thereby removing any unused
extra capacity and thus getting rid of the `capacity: usize` field that was used
to track it.

However, we can keep the Rust code simpler by skipping that step: with
`::safer_ffi`, you can directly return a Vec from FFI!

Simply use `repr_c::Vec` as the type used in the function signature and, when
needed, convert into and from a (Rust) `Vec` with `.into()`:

```rust,noplaypen
/// Document's CBOR
///
/// The returned value must be freed with `ditto_free_cbor`.
#[ffi_export]
fn ditto_document_cbor (
    document: &'_ Document,
) -> repr_c::Vec<u8>
{
    let vec: Vec<u8> =
        ::serde_cbor::to_vec(&document.to_value())
            .unwrap()
    ;
    vec.into()
}

#[ffi_export]
fn ditto_free_cbor (vec: repr_c::Vec<u8>)
{
    drop(vec);
}
```

<details><summary>Generated C header</summary>

```C
/** \brief
 *  Same as [`Vec<T>`][`rust::Vec`], but with guaranteed `#[repr(C)]` layout
 */
typedef struct {
    uint8_t * ptr;

    size_t len;

    size_t cap;
} Vec_uint8_t;

/** \brief
 *  Document's CBOR
 *
 *  The returned value must be freed with `ditto_free_cbor`.
 */
Vec_uint8_t ditto_document_cbor (
    Document_t const * document);

void ditto_free_cbor (
    Vec_uint8_t vec);
```

</details>

</details>

## Clarifying function signatures

As mentioned [previously][traditional-ffi-limits], traditional FFI leads to
(ab)using flat pointers everywhere, and the ownership _vs._ borrowing semantics,
as well as the nullability semantics are then lost, making a function's
signature almost impossible to read unless one has access to the body of the
function to know how such pointers are used, or unless there is an extensive
documentation that keeps up to date with such information.

And remember: documentation can become stale and fall out of sync, but the code
itself cannot!

### Real-life example

Indeed, compare the two following function signatures:

#### Old


```rust,noplaypen
/// Creates a new document from CBOR
///
/// It will allocate a new document and set `document` pointer to it. It will
/// later need to be released with `::ditto_document_free`.
///
/// The input `cbor` must be a valid CBOR.
///
/// Return codes:
///
/// * `0` -- success
/// * `1` -- invalid CBOR
/// * `2` -- cbor is not an object
/// * `3` -- ID string is empty
#[no_mangle] pub unsafe extern "C"
fn ditto_document_new_cbor (
    cbor_ptr: *const u8,
    cbor_len: usize,
    id: *const c_char,
    site_id: c_uint,
    document: *mut *mut Document,
) -> c_int
```

#### New

```rust,noplaypen
/// Creates a new document from CBOR
///
/// It will allocate a new document and set `document` pointer to it. It will
/// later need to be released with `::ditto_document_free`.
///
/// The input `cbor` must be a valid CBOR.
///
/// Return codes:
///
/// * `0` -- success
/// * `1` -- invalid CBOR
/// * `2` -- cbor is not an object
/// * `3` -- ID string is empty
#[ffi_export]
fn ditto_document_new_cbor (
    cbor: c_slice::Ref<'_, u8>,
    id: Option<char_p::Ref<'_>>,
    site_id: c_uint,
    document: Out<'_, Option<repr_c::Box<Document>>>,
) -> c_int
```

  - _c.f._ [the documentation of `Out`](
    https://docs.rs/uninit/0.3.0/uninit/out_ref/struct.Out.html#method.write)
    (write-only) references.

Thanks to the rich semantics of Rust types, the latter signature makes it
blatanly obvious that:

  - `id` can be `NULL`;

  - `document` is an out parameter, which may be used to write a `NULL` where
    it points to, and when a non-`NULL` pointer is written, then such pointer
    **carries ownership** over the pointed-to `Document`.

#### Future

Although not yet implemented, once more complex types such as `Result` get
[`ReprC`] equivalents, it will be possible to further simplify the above
function signature to:

```rust,noplaypen
#[ffi_export]
fn ditto_document_new_cbor (
    cbor: c_slice::Ref<'_, u8>,
    id: Option<char_p::Ref<'_>>,
    site_id: c_uint,
) -> repr_c::Result<repr_c::Box<Document>>
```
