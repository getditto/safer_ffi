{{#include ../links.md}}

# `VirtualPtr<dyn Trait>`

[`VirtualPtr`] is the key **pointer type** enabling all the FFI-safe `dyn`-support machinery in `safer-ffi`.

  - In order to better convey its purpose and semantics, other names considered for this type (besides a `VPtr` shorthand) have been:

    - `DynPtr<dyn Trait>`
    - `DynBox<dyn Trait>`
    - `VirtualBox<dyn Trait>` / `VBox` (this one has been _very_ strongly considered)

Indeed, this type embodies **owning pointer** semantics, much like `Box` does.

But it does so with a twist, hence the dedicated special name: **the owning mode is, itself, virtual/`dyn`**!

  - As will be seen in the remainder of this post, this aspect of `VirtualPtr` is gonna be the key element to allow **full type unification across even _different pointer types_!**

    For instance, consider:

    ```rust ,ignore
    fn together<'r>(
        a: Box<impl 'r + Trait>,
        b: Rc<impl 'r + Trait>,
        c: &'r impl Trait,
    ) -> [???; 3] // 🤔🤔🤔
    {
        [a.into(), b.into(), c.into()]
    }
    ```

    With `VirtualPtr`, we can fully type-erase and thus type-unify all these three types into a common one:

    ```rust ,ignore
    # (
    ) -> [VirtualPtr<dyn 'r + Trait>; 3] // 💡💡💡
    ```

### This allows a unified type able to cover all of `Box<dyn Trait>`, `{A,}Rc<dyn Trait>`, `&[mut] dyn Trait` under one same umbrella

> _One type to unify them all,_
>
> _One type to coërce them,_
>
> _One type to bring them all_
>
> _and in the erasure bind them._

![One VirtualPtr to rule them all][one-virtualptr-to-rule-them-all]

## Constructing a `VirtualPtr` from Rust

That is, whilst **a `Box<impl Trait>` can[^repr_c_trait] be "coërced" `.into()` a `VirtualPtr<dyn Trait>`**, `Box` will oftentimes not be the sole pointer/indirection with that capability. Indeed, there will often be other similar "coërcions" from a `&impl Trait`, a `&mut impl Trait`, a `Rc<impl Trait>`, or a `Arc<impl Trait + Send + Sync>`!

[^repr_c_trait]: provided that `dyn Trait` be a `ReprCTrait`, _i.e._, that the `Trait` definition have been `#[derive_ReprC(dyn)]`-annotated.

Here is the complete list of possible conversion at the moment:

 1. #### Given `<T> where T : 'T + Trait`,

 1. With `Trait` "being `ReprC`" / FFI-safe (_i.e._, `dyn Trait : ReprCTrait`)

| **`From<...>`** | **`.into()`**                         | **Notes** for `Trait` |
|---------------|---------------------------------------|---------------------------------------------------------------------------------|
| `Box<T>`      | `VirtualPtr<dyn 'T + Trait>`               | • (requires `T : Clone` when `Clone`-annotated)                                            |
| `&T`       | `VirtualPtr<dyn '_ + Trait>`          | • cannot have `&mut self` methods                                                        |
| `&mut T`   | `VirtualPtr<dyn '_ + Trait>`          | • cannot be `Clone`-annotated                                                         |
| `Rc<T>`       | `VirtualPtr<dyn 'T + Trait>`               | • must be `Clone`-annotated<br>• cannot have `&mut self` methods                                 |
| `Arc<T>`      | `VirtualPtr<dyn 'T + Trait + Send + Sync>` | • must be `Clone`-annotated<br>• cannot have `&mut self` methods<br>• requires `T : Send + Sync`          |

  - Where "`Clone`-annotated" refers to the `#[derive_Repr(dyn, Clone)]` case.

#### Remarks

  - Whenever `T : 'static`, we can pick `'T = 'static`, so that `dyn 'T + Trait` may be more succintly written as `dyn Trait`.

  - If the trait has methods with a `Pin`ned `self` receiver, then the `From<…>`-column needs to be `Pin`-wrapped.

  - **`+ Send` and/or `+ Sync` can always be added** inside a `VirtualPtr`, in which case `T : Send` and/or `T : Sync` (respectively) will be required.

      - The only exception here is `Rc`, since `Rc<dyn Trait + Send + Sync>` _& co._ are oxymorons which have been deemed not to deserve the necessary codegen (if multiple ownership and `Send + Sync` is required, use `Arc`, otherwise, use `Rc`).

    **Tip**: Since `+ Send + Sync` is so pervasive(ly recommended for one's sanity) when doing FFI, these can be added as super-traits of our `Trait`, so that they be implied in both `T : Trait` and `dyn Trait`, thereby alleviating the syntax without compromising the thread-safety:

    ```rust ,ignore
    #[derive_ReprC(dyn, /* Clone */)]
    trait Trait : Send + Sync {
    ```

      - But be aware that, even with such a super trait annotation, `dyn Trait` and `dyn Trait + Send + Sync` will remain being distinct types  as far as Rust is concerned! ⚠️

### Its FFI-layout: constructing and using `VirtualPtr` from the FFI

Given some:

```rust ,ignore
#[derive_ReprC(dyn, /* Clone */)]
trait Trait {
    fn get(&self, _: bool) -> i32;
    fn set(&mut self, _: i32);
    fn method3(&… self, _: Arg1, _: Arg2, …) -> Ret;
    …
}
```

  - (with `Arg1 : ReprC<CLayout = CArg1>`, _etc._)

A `VirtualPtr<dyn Trait>` will be laid out as the following:

```rust ,ignore
type ErasedPtr = ptr::NonNull<ty::Erased>; /* modulo const/mut */

#[repr(C)]
struct VirtualPtr<dyn Trait> {
    ptr: ErasedPtr,
    // Note: it is *inlined* / *no* pointer indirection!
    vtable: {
        // the `drop` / `free`ing function.
        release_vptr: unsafe extern fn(ErasedPtr),

        /* if `Clone`-annotated:
        retain_vptr: unsafe extern fn(ErasedPtr) -> VirtualPtr<dyn Trait>, */

        /* and the FFI-safe virtual methods of the trait: */
        get: unsafe extern fn(ErasedPtr, _: CLayoutOf<bool>) -> i32,
        set: unsafe extern fn(ErasedPtr, _: i32),
        method3: unsafe extern fn(ErasedPtr, _: CArg1, _: CArg2, …) -> CRet,
        …
    },
}
```

## A fully virtual owning mode

Remember the sentence above?

> But it does so with a twist, hence the dedicated special name: **the owning mode is, itself, virtual/`dyn`**!

What this means is that **_all of the destructor_ is virtual / `dyn`amically-dispatched**, for instance (and ditto for `.clone()`ing, when applicable).

### Non-fully-virtual examples

To better understand this nuance, consider the opposite (types which are not _fully_ virtual / `dyn`amically dispatched, such as `Box<dyn …>`): what happens when you drop a `Box<dyn Trait>` _vs._ dropping a `Rc<dyn Trait>`?

  - #### when you drop a `Box<dyn Trait>`:

     1. It _virtually/`dyn`_-amically queries the `Layout` knowledge of that `dyn Trait` type-erased data;
     1. It _virtually/`dyn`_-amically drops the `dyn Trait` pointee _in place_;
     1. It then calls `dealloc` (`free`) of the backing storage using the aforementioned data `Layout` (as the layout of the whole allocation, since a `Box<T>` allocates exactly as much memory as needed to hold a `T`)

    This last step is thus _statically_ dispatched, thanks to the _static_/compile-time knowledge of the hard-coded `Box` type in `Box<dyn Trait>`!

    <details><summary>Pseudo-code</summary>

    ```rust ,ignore
    //! Pseudo-code!
    fn drop(self: &mut Box<dyn …>) {
        let layout = self.dyn_layout(); // (self.vtable.layout)(self.ptr)
        unsafe {
            // SAFETY: this is conceptual pseudo-code and may have bugs.
            self.dyn_drop_in_place(); // (self.vtable.drop_in_place)(self.ptr)
            dealloc(&mut *self as *mut _, layout);
        }
    }
    ```

    </details>

  - #### when you drop a `Rc<dyn Trait>`:

     1. It _virtually/`dyn`_-amically queries the `Layout` knowledge of that `dyn Trait` type-erased data;
     1. It then embiggens the aforementioned layout so as to get the layout of all of the `Rc`'s actual pointee / actual allocation (that is, [the `RcBox`, _i.e._, the data alongside two reference counters](https://github.com/rust-lang/rust/blob/64165aac68af780182ff89a6eb3982e3c262266e/library/alloc/src/rc.rs#L290-L303)), so as to be able to access those counters,
     1. and then decrements the appropriate counters (mostly the strong count);
     1. if it detects that it was the last owner (strong count from 1 to 0):
         1. It _virtually/`dyn`_-amically drops the `dyn Trait` pointee _in place_;
         1. It then calls `dealloc` (`free`) for that whole `RcBox`'s backing storage (when there are no outstanding `Weak`s).

    The steps `2.`, `3.` and `4.2` are thus _statically_ dispatched, thanks to the _static_/compile-time knowledge of the hard-coded `Rc` type in `Rc<dyn Trait>`!

    <details><summary>Pseudo-code</summary>

    ```rust ,ignore
    //! Pseudo-code!
    fn drop(self: &mut Rc<dyn …>) {
        let layout = self.dyn_layout(); // (self.vtable.layout)(self.ptr)
        unsafe {
            // SAFETY: this is conceptual pseudo-code and may have bugs.
            let rcbox: &RcBox<dyn …> = adjust_ptr(self.ptr, layout);
            let prev = rcbox.strong_count.get();
            rcbox.strong_count.set(prev - 1);
            if prev == 1 {
                // if last strong owner
                rcbox.data.dyn_drop_in_place(); // (….vtable.drop_in_place)(….ptr)
                if rcbox.weak_count == … {
                    // if no outstanding weaks
                    dealloc(rcbox as *const _ as *mut _, layout);
                }
            }
        }
    }
    ```

    </details>

We can actually even go further, and wonder what Rust does:

  - #### when a `&mut dyn Trait` or a `&dyn Trait` goes out of scope:

     1. Nothing.

        (Since it knows that the `&[mut] _` types have no drop glue whatsoever)

    This step (or rather, lack thereof) is another example of _statically_ dispatched logic.

It should thus now be clear that:

  - whilst type erasure _of the pointee_ does happen whenever your deal with a `ConcretePtr<dyn Trait>` such as `Box<dyn Trait>`, `&mut dyn Trait`, _etc._

  - on the other hand, the `ConcretePtr` behind which such erasure happens is not, itself, type-erased! It is still statically-known, and functionality such as `Drop`, `Clone`, or even `Copy` may take advantage of that information (_e.g._, `&dyn Trait` is `Copy`).

### Another example: `dyn_clone()`

Let's now compare, in the context of type-erased `dyn Trait` pointees, a static operation _vs._ a virtual / `dyn`amically dispatched one.

For starters, let's consider the following `Trait` definition:

```rs
trait Trait : 'static {
    //                 &dyn Trait
    fn dyn_clone(self: &Self) -> Box<dyn Trait>;
}

impl<T : 'static + Clone> Trait for T {
    fn dyn_clone(self: &T) -> Box<dyn Trait> {
        Box::new(T::clone(self)) /* as Box<dyn Trait> */
    }
}
```

and now, let's think about and compare the behaviors of the two following functions:

```rs
fn clone_box(b: &Box<dyn Trait>) -> Box<dyn Trait> {
    b.dyn_clone()
}

fn clone_rc(r: &Rc<dyn Trait>) -> Rc<dyn Trait> {
    r.clone() // Rc::clone(r)
}
```

  - `clone_box` is `dyn`amically calling and delegating to `dyn Trait`'s `dyn_clone` virtual method;
  - `clone_rc` is statically / within-hard-coded code logic performing a (strong) reference-count increment inside the `RcBox<dyn Trait>` pointee, thereby never interacting with the `dyn Trait` value itself.

(Granted, the former is performing a statically-dispatched `Deref` coercion beforehand, and the latter may be `dyn`amically looking up `dyn Trait`'s `Layout`, but the main point still stands).

### From partially `dyn`amic to _fully_ `dyn`amic

From all this, I hope the hybrid static-`dyn`amic nature of Rust's `ConcretePtr<dyn ErasedPointee>` (wide) pointers logic is now more apparent and clearer.

From there, we can then wonder what happens if we made it all _fully_ `dynamic`: `VirtualPtr` is born!

#### Summary

  - _all_ of the `drop` glue is to be `dyn`amically dispatched (through some virtual `fn` pointer performing a `drop_ptr` operation):

    ```rust ,ignore
    //! Pseudo-code
    impl<T> DynDrop for Box<T> {
        fn dyn_drop_ptr(self)
        {
            drop::<Box<T>>(self);
        }
    }

    impl<T> DynDrop for Arc<T> {
        fn dyn_drop_ptr(self)
        {
            drop::<Arc<T>>(self);
        }
    }

    impl<T> DynDrop for &mut T {
        fn dyn_drop_ptr(self)
        {}
    }

    impl<T> DynDrop for &T {
        fn dyn_drop_ptr(self)
        {}
    }
    ```

    Notice how this shall therefore imbue with `move`/ownership semantics originally-`Copy` pointers such as `&T`. Indeed, once we go fully virtual, by virtue of being compatible/type-unified with non-`Copy` pointers such as `Box<T>` or `&mut T`, it means we have to conservatively assume any `VirtualPtr<…>` instance may have to run significant drop glue at most once, which thence makes `VirtualPtr`s not be `Copy`, even when they've originated from a `&T` reference.

  - `Clone`, if any, is also to be fully `dyn`amically dispatched as well:

    ```rust ,ignore
    //! Pseudo-code
    impl<T> DynClone for Box<T>
    where
        T : Clone,
    {
        fn dyn_clone_ptr(self: &Self)
          -> Self
        {
            Box::new(T::clone(&**self))
        }
    }

    impl<T> DynClone for Arc<T> {
        fn dyn_clone_ptr(self: &Self)
          -> Self
        {
            Arc::clone(self)
        }
    }

    /*
     * no `Clone` for `&mut`, obviously:
     * thus, no `From<&mut T>` for `VirtualPtr<dyn DynClone>` either.
     */

    impl<T> DynClone for &'_ T {
        fn dyn_clone_ptr(self: &Self)
          -> Self
        {
            // `&T : Copy`
            *self
        }
    }
    ```

    Regarding the previous point about `&T`-originated `VirtualPtr`s not being `Copy` anymore, we can see we can get the functional API back (_i.e._, `Clone`), if we pinky promise not to mix such `VirtualPtr`s with non-`Clone`-originating pointers (such as `&mut T`)

      - <details><summary>Bonus: <code>&mut T</code>-reborrowing</summary>

        If you think about `&mut T`, whilst not `Copy`, it's still kind of an interesting pointer, since a `&'short mut &'long mut T` can yield a `&'short mut T` through reborrowing, thereby removing one layer of indirection, by "squashing" the lifetimes together into their intersection (which here happens to be the shortest one, `'short`, since `'long : 'short`).

        In explicit API parlance, this would become:

        ```rust ,ignore
        impl<'long> DynReborrowMut for &'long mut T {
            fn dyn_reborrow_mut(
                //    &'short mut VirtualPtr<dyn 'long + …>
                self: &'short mut Self,
            ) -> &'short mut T
            //   VirtualPtr<dyn 'short + …>
            {
                *self /* `&mut **self` to be precise */
            }
        }

        impl<'long> DynReborrowMut for &'long T { // …
        # }
        ```

        Despite intellectually interesting, this is nonetheless a niche and contrived API which is therefore not exposed through `safer-ffi`'s `VirtualPtr` type, for it is deemed that `&'short mut VirtualPtr<dyn 'long + …>` ought to offer most of the API of a reborrowed `VirtualPtr<dyn 'short + …>`.

        ___

        </details>

### Related read/concept: `dyn *`

In a way, the API/functionality of `VirtualPtr` is quite similar to the very recent `dyn *` experimental[^exp] [unstable feature](https://dev-doc.rust-lang.org/1.68.0/unstable-book/language-features/dyn-star.html) of Rust.

As of this writing[^exp], there isn't that much proper documentation about it, and one would have to wander through Zulip discussions to know more about it, but for the following post:

> ### [_A Look at dyn* Code Generation_ — by Eric Holk](https://theincredibleholk.org/blog/2022/12/12/dyn-star-codegen/)

[^exp]: as of 1.68.0

<details><summary>Click here to see my own summary of <code>dyn *</code></summary>

The gist of it is that barring `repr(C)` / `extern "C"` / FFI-compatibility considerations about which `dyn *` does not worry, the idea is kind of the same as `VirtualPtr`, but for one extra quirk. Instead of a simple thin pointer-sized data pointer, a `dyn *` will rather use the following "erased data" type:

```rust ,ignore
union ErasedData<const N: usize = 1> {
    ptr: *mut ty::Erased,
    inline: [MaybeUninit<usize>; N],
}

struct dyn* Trait<const N: usize = 1> {
    data: ErasedData<N>,
    vtable: …,
}
```

Historically, most platforms have featured `*mut _` and `usize` having the same layout, so on those platforms and in the case of `N = 1` (and papering over `MaybeUninit` / padding bytes), you may still be seeing a `*mut ty::Erased` _directly_ rather than an `ErasedData`.

For instance, we could imagine all this applied to our `VirtualPtr`s: we'd now be able to implement it for non-pointer types, provided they fit within the `ErasedData` inline storage!

```rust ,ignore
//! Pseudo-code: currently *NOT* in `safer-ffi`

#[derive_ReprC(dyn)]
trait Example : Clone {
    fn dyn_print(&self);
}

impl Example for usize /* which fits inside `*mut Erased` */ {
    /* auto-generated:
    fn dyn_drop(self)
    {
        /* nothing to do to drop a `usize` since it is `Copy` */
    }

    fn dyn_clone(self: &Self)
      -> Self
    {
        *self
    } */

    fn dyn_print(self: &Self)
    {
        dbg!(*self);
    }
}

fn main()
{
    let n: usize = 42;
    // look ma: no indirection!
    let vptr: VirtualPtr<dyn Example> = n.into();
 /* let vptr = VirtualPtr {
        vptr: ErasedData { inline: n },
        vtable: VTable {
            release_vptr: usize::dyn_drop,
            retain_vptr: usize::dyn_clone,
            dyn_print: usize::dyn_print,
        },
    }; */
    vptr.dyn_print(); // correctly prints `42`.
}
```

  - Note that this is currently deemed too niche and is **not** featured by `safer-ffi`.

</details>
