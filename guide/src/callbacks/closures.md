{{#include ../links.md}}

# Adding state: Closures

Since bare function pointers cannot carry any non-global instance-specific
state, their usability is quite limited. For a callback-based API to be good,
it must be able to support some associated state.

## Stateful callbacks in C

In C, the idiomatic way to achieve this is to carry an extra `void *` parameter
(traditionally called `data`, `ctx`, `env` or `payload`), and have the function pointer
receive it as one of its parameters:

<details><summary>Example</summary>

```C
#include <assert.h>
#include <stdlib.h>

void call_n_times (
    size_t repeat_count,
    void (*cb)(void * cb_ctx),
    void * cb_ctx)
{
    for (size_t i = 0; i < repeat_count; ++i) {
        (*cb)(cb_ctx);
    }
}

void my_cb (
    void * cb_ctx);

int main (void)
{
    int counter = 0; // state to be shared
    int * at_counter = &counter; // pointer to the state
    void * cb_ctx = (void *) at_counter; // type-erased
    call_n_times(
        42,
        my_cb,
        cb_ctx)
    ;
    assert(counter == 42);
    return EXIT_SUCCESS;
}
// where
void my_cb (
    void * cb_ctx)
{
    int * at_counter = (int *) cb_ctx; // undo type erasure
    *at_counter += 1; // access state through dereference
}
```

</details>

This pattern is so pervasive that the natural thing to do is to bundle those
two fields (data pointer, and function pointer) within a `struct`:

```C
typedef struct MyCallback {
    void (*cb)(void * ctx);
    void * ctx;
} MyCallback_t;
```

<details><summary>Example</summary>

```C
#include <assert.h>
#include <stdlib.h>

typedef struct MyCallback {
    void * ctx;
    void (*fptr)(void * ctx);
} MyCallback_t;

void call_n_times (
    size_t repeat_count,
    MyCallback_t cb)
{
    for (size_t i = 0; i < repeat_count; ++i) {
        (*cb.fptr)(cb.ctx);
    }
}

void my_cb (
    void * cb_ctx);

int main (void)
{
    int counter = 0;
    call_n_times(
        42,
        (MyCallback_t) {
            .fptr = my_cb,
            .ctx = (void *) &counter,
        })
    ;
    assert(counter == 42);
    return EXIT_SUCCESS;
}
// where
void my_cb (
    void * cb_ctx)
{
    int * at_counter = (int *) cb_ctx;
    *at_counter += 1;
}
```

</details>

## Back to Rust

In Rust, the situation is quite more subtle, since the properties of the
closure are not wave-handed like they are in C. Instead, there are very
rigurous things to take into account:

  - #### `'static`

    Can the environment be held arbitrarily long, or is there some call frame /
    scope / lifetime it cannot outlive?

  - #### `Send`

    Can the environment be accessed (non-concurrently) from another thread?

      - For the sake of sanity, non-`Send` closures are not supported.

  - #### `Fn` _vs._ `FnMut`

    Both involve a callable API, but `FnMut` involves non-concurrent access
    whereas `Fn` allows concurrent access (_e.g._, closure then has to be
    reentrant-safe and, when `Sync`, thread-safe too).

  - #### `Sync` (+ `Fn`)

    Is the closure thread-safe / can it be called in parallel?

> To get a better understanding of the `Fn*` traits and the `move? |...| ...`
> closure sugar in Rust I highly recommend reading the
> [_Closures: Magic functions_ blog post][closures magic functions].

___

Such struct definitions are available, in a generic fashion, in `::safer_ffi`,
under the [`::safer_ffi::closure` module][`::safer_ffi::closure`].

<details class="warning">

<summary>Disclaimer about callbacks using lifetimes</summary>

Function signatures involving lifetimes are not supported yet (and will
probably never be, due to a limitation of Rust's typesystem). Using a
transparent newtype around concrete closure signatures would circumvent that
genericity limitation, and the crate's author intends to release a macro that
would automate that step. In the meantime, you will have to use raw pointers or
the `Raw` variants of the [provided C types][repr-c-forall] (_e.g._,
`c_slice::Raw`, `char_p::Raw`).

</details>

For instance, `MyCallback_t` above is equivalent to using, within Rust, the
[`RefDynFnMut0`]`<'_, ()>` type, a [`ReprC`] version of
`&'_ mut (dyn Send + FnMut())`:

<details><summary>C layout</summary>

```C
typedef struct {
    // Cannot be NULL
    void * env_ptr;
    // Cannot be NULL
    void (*call)(void * env_ptr);
} RefDynFnMut0_void_t;
```

</details>

### Borrowed closures

More generally, when having to deal with a
<span title="no destructor or ressources to release whatsoever, but instead a scope / lifetime that must not be outlived. This is the most common situation in the C world"><u>borrowed</u><sup>?</sup></span>
stateful callback
having `N` inputs of type `A1, A2, ..., An`, and a return type of `Ret`, _i.e._,
a `&'_ mut (dyn Send + FnMut(A1, ..., An) -> Ret)`, then the [`ReprC`]
equivalent to use is:

> [`RefDynFnMutN`]`<'_, Ret, A1, ..., An>`

  - C layout:

    ```C
    typedef struct {
        // Cannot be NULL
        void * env_ptr; // &'_ mut TypeErased
        // Cannot be NULL
        Ret_t (*call)(void * env_ptr,
            A1_t arg_1,
            A2_t arg2,
            ...,
            An_t arg_n);
    } RefDynFnMutN_Ret_A1_A2_..._An_t;
    ```

</details>


<details><summary>Example: <code>call_n_times</code> in Rust</summary>

The previously shown API:

```C
typedef struct MyCallback {
    void * ctx;
    void (*fptr)(void * ctx);
} MyCallback_t;

void call_n_times (
    size_t repeat_count,
    MyCallback_t cb)
{
    for (size_t i = 0; i < repeat_count; ++i) {
        (*cb.fptr)(cb.ctx);
    }
}
```

can be trivially implemented in Rust with the following code:

```rust,noplaypen
use ::safer_ffi::prelude::*;

#[ffi_export]
fn call_n_times (
    repeat_count: usize,
    cb: RefDynFnMut0<'_, ()>,
)
{
    // A current limitation of the `#[ffi_export]` is that it does not support
    // any non-identifier patterns such as `mut cb`.
    // We thus need to rebind it at the beginning of the function's body.
    // This ought to be fixed very soon.
    let mut cb = cb;
    for _ in 0 .. repeat_count {
        cb.call();
    }
}
```

<details><summary>Bonus: calling it from Rust</summary>

Although most FFI functions are only to be called by C, sometimes we wish to
call them from Rust too (_e.g._, when wanting to test them). In that case,
know that the `...DynFn...N<...>` family of [`ReprC`] closures all come with:

  - constructors supporting the equivalent Rust types (before type erasure!);

  - as well as as `.call(...)` method as showcased just above;

  - when dealing with owned variants, the Rust types implement `Drop` (so that
    offering a function to free a closure is as simple as exporting a function
    that simply `drop`s its input);

  - and finally, when dealing with `ArcDynFnN<...>`, it also implements `Clone`,
    although it will `panic!` if the `.retain` function pointer happens to be
    `NULL`.

```rust,noplaypen
let mut count = 0;
call_n_times(42, RefDynFnMut0::new(&mut || { count += 1; }));
assert_eq!(count, 42);
```

</details>

</details>

### Owned closures

When, instead, the closure may be held arbitrarily long (_e.g._, in another
thread), and may have some destructor logic, _i.e._, when dealing with a
heap-allocation-agnostic generalization of:

```rust,noplaypen
Box<dyn 'static + Send + FnMut(A1, ..., An) -> Ret>
```

then, the [`ReprC`] equivalent type to use is:

> [`BoxDynFnMutN`]`<Ret, A1, ..., An>`

<details><summary>C layout</summary>

```C
typedef struct {
    // Cannot be NULL
    void * env_ptr; // Box<TypeErased>
    // Cannot be NULL
    Ret_t (*call)(void * env_ptr,
        A1_t arg_1,
        A2_t arg2,
        ...,
        An_t arg_n);
    // Cannot be NULL
    void (*free)(void * env_ptr);
} BoxDynFnMutN_Ret_A1_A2_..._An_t;
```

</details>

### Ref-counted thread-safe closures

And, finally, when, on top of the previous considerations, the closure may have
multiple owners (requiring ref-counting) and/or may be called by concurrent
(`Fn` instead of `FnMut`) and even _parallel_ (added `Sync` bound) code, _i.e._,
when dealing with a heap-allocation-agnostic generalization of:

```rust,noplaypen
Arc<dyn 'static + Send + Sync + Fn(A1, ..., An) -> Ret>
```

then, the [`ReprC`] equivalent type to use is:

> [`ArcDynFnN`]`<Ret, A1, ..., An>`

<details><summary>C layout</summary>

```C
typedef struct {
    // Cannot be NULL
    void * env_ptr; // Arc<TypeErased>
    // Cannot be NULL
    Ret_t (*call)(void * env_ptr,
        A1_t arg_1,
        A2_t arg2,
        ...,
        An_t arg_n);
    // Cannot be NULL
    void (*release)(void * env_ptr);
    // May be NULL
    void (*retain)(void * env_ptr);
} ArcDynFnN_Ret_A1_A2_..._An_t;
```

  - Note how an `ArcDynFnN... *` can be casted to a `BoxDynFnMutN... *` (same
    prefix), and how the latter can be converted to the former by having
    `.retain = NULL`.
</details>
