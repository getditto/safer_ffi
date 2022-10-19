use {
    ::core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    },
    ::safer_ffi::{
        prelude::*,
    },
};

/// An FFI-safe `Poll<()>`.
#[derive_ReprC]
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub
enum PollFuture {
    Success = 0,
    Pending = 1,
    Failure = -1,
}

/// Output has to be `Result<(), ()>`.
#[derive_ReprC]
pub
trait FfiFuture {
    fn poll (self: Pin<&mut Self>, ctx: &'_ mut Context<'_>)
      -> PollFuture
    ;
}

impl<F : Future<Output = Result<(), ()>>> FfiFuture for F {
    fn poll (self: Pin<&mut Self>, ctx: &'_ mut Context<'_>)
      -> PollFuture
    {
        match Future::poll(self, ctx) {
            Poll::Pending => PollFuture::Pending,
            Poll::Ready(Ok(())) => PollFuture::Success,
            Poll::Ready(Err(())) => PollFuture::Failure,
        }
    }
}

match_! {(
    []
    [Send]
) {(
    $([ $($Send:ident)? ])*
) => (
    $(
        impl VirtualPtr<dyn '_ + $($Send +)? FfiFuture> {
            pub
            async fn into_future (mut self)
              -> Result<(), ()>
            {
                ::futures::future::poll_fn(
                    move |cx| match Pin::new(&mut self).poll(cx) {
                        PollFuture::Pending => Poll::Pending,
                        PollFuture::Success => Poll::Ready(Ok(())),
                        PollFuture::Failure => Poll::Ready(Err(())),
                    }
                )
                .await
            }
        }
    )*
)}}


#[derive_ReprC]
pub trait FfiFutureExecutor {
    fn dyn_spawn (
        self: &'_ Self,
        future: VirtualPtr<dyn 'static + Send + FfiFuture>,
    ) -> VirtualPtr<dyn 'static + Send + FfiFuture>
    ;

    fn dyn_spawn_blocking (
        self: &'_ Self,
        action: repr_c::Box<dyn 'static + Send + FnMut()>,
    ) -> VirtualPtr<dyn 'static + Send + FfiFuture>
    ;

    fn dyn_block_on (
        self: &'_ Self,
        future: VirtualPtr<dyn '_ + Send + FfiFuture>,
    )
    ;
}

impl VirtualPtr<dyn 'static + Send + Sync + FfiFutureExecutor> {
    fn spawn (
        self: &'_ Self,
        fut: impl 'static + Send + Future<Output = Result<(), ()>>,
    ) -> impl Future<Output = Result<(), ()>>
    {
        self.dyn_spawn(Box::new(fut).into())
            .into_future()
    }

    fn spawn_blocking (
        self: &'_ Self,
        action: impl 'static + Send + FnOnce(),
    ) -> impl Future<Output = Result<(), ()>>
    {
        let mut action = Some(action);
        let action = move || {
            action
                .take()
                .expect("\
                    `.spawn_blocking()` called the given closure \
                    more than once\
                ")
                ()
        };
        self.dyn_spawn_blocking(Box::new(action).into())
            .into_future()
    }

    fn block_on<R : Send> (
        self: &'_ Self,
        fut: impl Send + Future<Output = R>
    ) -> R
    {
        let mut ret = None::<R>;
        self.dyn_block_on(
            Box::new(async {
                ret = Some(fut.await);
                Ok(())
            })
            .into()
        );
        ret.expect("`.dyn_block_on()` did not complete")
    }
}

cfg_match! { feature = "tokio" => {
    #[deny(unconditional_recursion)]
    impl FfiFutureExecutor for ::tokio::runtime::Handle {
        fn dyn_spawn (
            self: &'_ Self,
            future: VirtualPtr<dyn 'static + Send + FfiFuture>,
        ) -> VirtualPtr<dyn 'static + Send + FfiFuture>
        {
            let fut = self.spawn(future.into_future());
            let fut = async {
                fut.await.unwrap_or(Err(()))
            };
            Box::new(fut)
                .into()
        }

        fn dyn_spawn_blocking (
            self: &'_ Self,
            action: repr_c::Box<dyn 'static + Send + FnMut()>,
        ) -> VirtualPtr<dyn 'static + Send + FfiFuture>
        {
            let fut = self.spawn_blocking(|| { action }.call());
            let fut = async {
                fut .await
                    .map_err(drop)
            };
            Box::new(fut)
                .into()
        }

        fn dyn_block_on (
            self: &'_ Self,
            future: VirtualPtr<dyn '_ + Send + FfiFuture>,
        )
        {
            self.block_on(future.into_future()).ok();
        }
    }

    #[cfg(any())]
    #[deny(unconditional_recursion)]
    impl FfiFutureExecutor for ::tokio::runtime::Runtime {
        fn dyn_spawn (
            self: &'_ Self,
            future: VirtualPtr<dyn 'static + Send + FfiFuture>,
        ) -> VirtualPtr<dyn 'static + Send + FfiFuture>
        {
            let fut = self.spawn(future.into_future());
            let fut = async {
                fut.await.unwrap_or(Err(()))
            };
            Box::new(fut)
                .into()
        }

        fn dyn_spawn_blocking (
            self: &'_ Self,
            action: repr_c::Box<dyn 'static + Send + FnMut()>,
        ) -> VirtualPtr<dyn 'static + Send + FfiFuture>
        {
            let fut = self.spawn_blocking(move || action.call());
            let fut = async {
                fut.await.map_err(drop)
            };
            Box::new(fut)
                .into()
        }

        fn dyn_block_on (
            self: &'_ Self,
            future: VirtualPtr<dyn '_ + Send + FfiFuture>,
        )
        {
            self.block_on(future.into_future()).ok();
        }
    }
}}

#[macro_export]
macro_rules! ffi_export_future_helpers {() => (
    const _: () = {
        #[ffi_export]
        fn rust_task_context_wake (
            task_context: &'static ::core::task::Context<'static>,
        )
        {
            task_context.waker().wake_by_ref()
        }

        #[ffi_export]
        fn rust_task_context_get_waker (
            task_context: &'static ::core::task::Context<'static>,
        ) -> $crate::prelude::repr_c::Arc<dyn 'static + Send + Sync + Fn()>
        {
            let waker = task_context.waker().clone();
            ::std::sync::Arc::new(move || waker.wake_by_ref()).into()
        }
    };
)}

#[cfg(test)]
mod tests {
    use super::*;

    // Can convert a future into an ffi one, and back!
    async fn check ()
    {
        let fut: VirtualPtr<dyn FfiFuture> = Box::new(async {
            async {}.await;
            Ok(())
        }).into();
        fut.into_future().await;
    }

    #[ffi_export]
    fn test_spawner (
        executor: VirtualPtr<dyn 'static + Send + Sync + FfiFutureExecutor>,
    )
    {
        let _: i32 = executor.spawn(async {

        });
        let _: i32 = executor.block_on(async {
            async {}.await;
            42
        });
    }

    ffi_export_future_helpers!();
}
