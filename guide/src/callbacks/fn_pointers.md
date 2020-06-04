{{#include ../links.md}}

# Function pointers

These are the most simple callback objects. They consist of a single (function)
pointer, that is, the address of the (beginning of the) code that is to be
called.

  - In Rust, this is the family of `fn(...) -> _` types.

    To avoid (very bad) bugs when mixing calling conventions (ABIs), Rust
    includes the ABI within the type of the function pointer, granting
    additional type-level safety.
    When dealing with C, the calling convention that matches C's is
    almost always `extern "C"` and is _never_ `extern "Rust"`.

    <span class = "warning">

    When unspecified, the calling convention defaults to `extern "Rust"`, which
    is different from `extern "C"`!

    </span>

    This is why all function pointers involved in FFI need to be `extern "C"`.
    Forgetting to annotate it results in code that triggers _Undefined
    Behavior_ (and
    [traditional FFI fails to guard against it][traditional-ffi-limits])⚠️

  - In C, these are written as `ret_t (*name)(function_args)`, where `name` is
    the name of a variable or parameter that has the function pointer type,
    or the name of the type being type-aliased to the function pointer type.

## Examples

|                             Rust                             |                      C                     |
|:------------------------------------------------------------:|:------------------------------------------:|
|                     `cb: extern "C" fn()`                    |             `void (*cb)(void)`             |
|          `f: extern "C" fn(arg1_t, arg2_t) -> ret_t`         |        `ret_t (*f)(arg1_t, arg2_t)`        |
|          `transmute::<_, extern "C" fn(arg_t) -> ret_t>(f)`          |              `(ret_t (*)(arg_t)) (f)`              |
| `type cb_t = extern "C" fn(arg_t) -> ret_t;`<br/>`let f: cb_t = ...;`<br/>`transmute::<_, cb_t>(f)` | `typedef ret_t (*cb_t)(arg_t);`<br/>`cb_t f = ...;`<br/>`(cb_t) (f)` |

So, for instance,

```rust,noplaypen
#[ffi_export]
fn call (
    ctx: *mut c_void,
    cb: unsafe extern "C" fn(ctx: *mut c_void),
)
```

becomes

```C
void call (
    void * ctx,
    void (*cb)(void * ctx)
);
```

### Nullable function pointers

<span class = "warning">

A Rust `fn` pointer _cannot_ possibly be NULL!

</span>

This means that when `NULL`-able function pointers are involved, **forgetting to
`Option`-wrap them can lead to _Undefined Behavior_**. Luckily, this is something
that is easily caught by [`::safer_ffi`'s sanity checks][sanity checks].
