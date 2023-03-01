# Example: FFI-safe `Future`s and executors

By the way, all this is actually already directly featured by `safer-ffi` itself if you enable the `futures` or `tokio` features.

## `FfiFuture`

 1. `Future` Trait:

    ```rust ,ignore
    trait Future {
        type Output;

        fn poll(self: Pin<&mut Self>, &'_ mut Context<'_>)
          -> Poll<Self::Output>
        ;
    }
    ```

 1. _Simplify_ it: `Future<Output = ()>`

    ```rust ,ignore
    trait SimpleFuture {
        fn dyn_poll(self: Pin<&mut Self>, &'_ mut Context<'_>)
          -> Poll<()>
        ;
    }
    ```

      - Renamed to `dyn_poll` for more readable semantics down the line.

 1. Make it FFI-compatible: define an FFI-safe `Poll<()>` equivalent

    ```rust
    #[derive_ReprC]
    #[repr(u8)]
    enum FfiPoll {
        Completed,
        Pending,
    }

    #[derive_ReprC(dyn)]
    trait FfiFuture {
        fn dyn_poll(self: Pin<&mut Self>, &'_ mut Context<'_>)
          -> FfiPoll
        ;
    }
    ```

      - We have reached a point where `#[derive_ReprC(dyn)]` can be used!

      - This means we have now gotten the `impl -> dyn` "coërcions":

          - ```rust ,ignore
            impl<'f, F : 'f + FfiFuture> From<
                Pin<Box<F>>
            > for // ↓
                VirtualPtr<dyn 'f + FfiFuture>
            ```

          - ```rust ,ignore
            impl<'f, F : 'f + FfiFuture> From<
                Pin<&'f mut F>
            > for // ↓
                VirtualPtr<dyn 'f + FfiFuture>
            ```

 1. Convenience conversions to/from Rust Futures.

      - From:

        ```rust
        impl<F : Future<Output = ()>> FfiFuture for F {
            fn dyn_poll(self: Pin<&mut Self>, ctx: &'_ mut Context<'_>)
              -> FfiPoll
            {
                match self.poll(ctx) {
                    | Poll::Pending => FfiPoll::Pending,
                    | Poll::Ready(()) => FfiPoll::Completed,
                }
            }
        }
        ```

      - Into:

        ```rust
        impl<'fut> VirtualPtr<dyn 'fut + FfiFuture> {
            fn into_future(self)
              -> impl 'fut + Future<Output = ()>
            {
                let mut vptr = self;
                ::core::future::poll_fn(move /* vptr */ |cx| {
                    match Pin::new(&mut vptr).dyn_poll(cx) {
                        | FfiPoll::Pending => Poll::Pending,
                        | FfiPoll::Completed => Poll::Ready(()),
                    }
                })
            }
        }
        ```

## Bonus: offering `Wake` ups to the FFI:

Remember: `Context` is an opaque FFI type, which makes it unusable there. FFI users won't be able to provide their own custom `dyn FfiFuture` objects (other than trivial `Future`s or `Future`s busy-looping the polls).

We can palliate this by exposing virtual methods / accessors specific to the opaque `Context` type:

```rust
#[macro_export]
macro_rules! ffi_export_future_helpers {() => (
    const _: () = {
        use $crate::ඞ::std::{sync::Arc, task::Context, prelude::v1::*};

        #[ffi_export]
        fn rust_future_task_context_wake (
            task_context: &'static Context<'static>,
        )
        {
            task_context.waker().wake_by_ref()
        }

        #[ffi_export]
        fn rust_future_task_context_get_waker (
            task_context: &'static Context<'static>,
        ) -> Arc<dyn 'static + Send + Sync + Fn()>
        {
            let waker = task_context.waker().clone();
            Arc::new(move || waker.wake_by_ref()).into()
        }
    };
)}
```

## FFI-safe `Executor` / `Runtime Handle`

Note: `::tokio` does not expose an `Executor` API, but rather, a `Runtime`, which packages/bundles both the `Executor` part[^exec] as well as the `Reactor` part[^reac]. The latter is the reason some of `::tokio`'s **leaf** `Future`s (such as File I/O and `sleep`s) cannot be polled, by default, by other executors… See <https://docs.rs/async-compat> for more info.

[^exec]: the `block_on()` runtime in charge of calling `.poll()` to make a future progress, with concurrent `.poll()`s to spawed `Future`s.

[^reac]: background thread(s)/threadpool to which certain `Waker::wake()` calls are scheduled, needed by certain leaf futures.


 1. A `trait` abstraction thereof:

    ```rust
    trait Executor : Send + Sync {
        fn block_on<T>(
            self: &'_ Self,
            future: impl '_ + Future<Output = T>,
        ) -> T
        ;
        fn spawn(
            self: &'_ Self,
            future: impl 'static + Send + Future<Output = ()>
        ) -> Pin<Box<dyn Send + Future<Output = ()>>>
        ;
    }
    ```

 1. Making it `dyn`-safe:

    ```rust
    trait Executor : Send + Sync {
        fn dyn_block_on(
            self: &'_ Self,
            future: Pin<Box<dyn '_ + Future<Output = ()>>>,
        )
        ;
        fn dyn_spawn(
            self: &'_ Self,
            future: Pin<Box<dyn Send + Future<Output = ()>>>
        ) -> Pin<Box<dyn Send + Future<Output = ()>>>
        ;
    }
    ```

 1. Making it FFI-safe:

    ```rust
    #[derive_ReprC(dyn, Clone)]
    trait FfiFutureExecutor : Send + Sync {
        fn dyn_block_on(
            self: &'_ Self,
            future: VirtualPtr<dyn '_ + FfiFuture>,
        )
        ;
        fn dyn_spawn(
            self: &'_ Self,
            future: VirtualPtr<dyn Send + FfiFuture>
        ) -> VirtualPtr<dyn Send + FfiFuture>
        ;
    }
    ```

 1. From a `::tokio::Handle`:

    ```rust ,ignore
    impl FfiFutureExecutor for ::tokio::runtime::Handle {
        fn dyn_block_on (
            self: &'_ Self,
            ffi_fut: VirtualPtr<dyn '_ + FfiFuture>,
        )
        {
            self.block_on(ffi_fut.into_future())
        }

        fn dyn_spawn (
            self: &'_ Self,
            future: VirtualPtr<dyn Send + FfiFuture>,
        ) -> VirtualPtr<dyn Send + FfiFuture>
        {
            let handle_result = self.spawn(future.into_future());
            let handle_unwrapped = async {
                handle
                    .await
                    .unwrap_or_else(|caught_panic| {
                        ::std::panic::resume_unwind(caught_panic.into_panic())
                    })
            };
            Box::pin(handle_unwrapped)
                .into()
        }
    }
    ```

 1. Usable as an `Executor`:

    ```rust
    /// Notice how we got the generics back!
    impl VirtualPtr<dyn FfiFutureExecutor> {
        fn block_on<R> (
            self: &'_ Self,
            fut: impl Future<Output = R>
        ) -> R
        {
            // We use `Option<R>` as a simple non-`'static` channel
            // to get the generic payload back.
            let mut ret = None;
            self.dyn_block_on(
                // From< Pin<&mut F> >
                ::core::pin::pin!(async {
                    ret = Some(fut.await);
                })
                .into()
            );
            ret.expect("`.dyn_block_on()` did not complete")
        }

        fn spawn<R : 'static + Send> (
            self: &'_ Self,
            fut: impl 'static + Send + Future<Output = R>,
        ) -> impl 'static + Future<Output = R>
        {
            // Channel to be able to get the generic payload back.
            let (tx, rx) = ::futures::channel::oneshot::channel();
            let ffi_handle = self.dyn_spawn(
                // From< Pin<Box<F>> >
                Box::pin(async move {
                    tx.send(fut.await).ok();
                })
                .into()
            );
            let handle = async move {
                ffi_handle.into_future().await;
                rx  .await
                    .expect("\
                        executor dropped the `spawn`ed task \
                        before its completion\
                    ")
            };
            handle
        }
    }
    ```
