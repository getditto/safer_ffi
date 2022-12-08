use super::*;

/// Models an `async` runtime's _handle_.
#[derive_ReprC(dyn, Clone)]
pub
trait FfiFutureExecutor : Send + Sync {
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
        future: VirtualPtr<dyn '_ + FfiFuture>,
    )
    ;

    fn dyn_enter (
        self: &'_ Self,
    ) -> VirtualPtr<dyn '_ + DropGlue>
    {
        Box::new(ImplDropGlue(())).into()
    }
}

match_! {([] [Send + Sync]) {( $([ $($SendSync:tt)* ])* ) => (
    $(
        impl VirtualPtr<dyn 'static + FfiFutureExecutor + $($SendSync)*> {
            #[cfg(feature = "tokio")]
            pub
            fn spawn_within_tokio_reactor<R : 'static + Send> (
                self: &'_ Self,
                fut: impl 'static + Send + Future<Output = R>,
            ) -> impl Future<Output = R>
            {
                self.spawn(::async_compat::Compat::new(fut))
            }

            pub
            fn spawn<R : 'static + Send> (
                self: &'_ Self,
                fut: impl 'static + Send + Future<Output = R>,
            ) -> impl Future<Output = R>
            {
                let (tx, rx) = ::futures::channel::oneshot::channel();
                let fut = self.dyn_spawn(
                    Box::pin(async move {
                        tx.send(fut.await).ok();
                    })
                    .into()
                );
                async move {
                    fut.into_future().await;
                    rx  .await
                        .expect("\
                    executor dropped the `spawn`ed task before its completion\
                        ")
                }
            }

            pub
            fn spawn_blocking<R : 'static + Send> (
                self: &'_ Self,
                action: impl 'static + Send + FnOnce() -> R,
            ) -> impl Future<Output = Result<R, ::futures::channel::oneshot::Canceled>>
            {
                let (tx, rx) = ::futures::channel::oneshot::channel();
                let mut action = Some(move || {
                    let _ = tx.send(action());
                });
                let action = move || {
                    action
                        .take()
                        .expect("\
                    executor called the `.spawn_blocking()` closure more than once\
                        ")
                        ()
                };
                let fut = self.dyn_spawn_blocking(Box::new(action).into());
                async move {
                    fut.into_future().await;
                    rx.await
                }
            }

            #[cfg(feature = "tokio")]
            pub
            fn block_on_within_tokio_reactor<R> (
                self: &'_ Self,
                fut: impl Future<Output = R>
            ) -> R
            {
                self.block_on(::async_compat::Compat::new(fut))
            }

            pub
            fn block_on<R> (
                self: &'_ Self,
                fut: impl Future<Output = R>
            ) -> R
            {
                let mut ret = None;
                let fut = async {
                    ret = Some(fut.await);
                };
                {
                    ::futures::pin_mut!(fut);
                    self.dyn_block_on(
                        fut.into()
                    );
                }
                ret.expect("`.dyn_block_on()` did not complete")
            }

            /// "Enters" a context within which certain executor-specific
            /// `Handle::current()`-functionalities shall work properly.
            pub
            fn enter (
                self: &'_ Self,
            ) -> impl '_ + Sized
            {
                self.dyn_enter()
            }
        }
    )*
)}}

cfg_match!(feature = "tokio" => {
    impl FfiFutureExecutor for ::tokio::runtime::Handle {
        fn dyn_spawn (
            self: &'_ Self,
            future: VirtualPtr<dyn 'static + Send + FfiFuture>,
        ) -> VirtualPtr<dyn 'static + Send + FfiFuture>
        {
            let fut = self.spawn(future.into_future());
            let fut = async {
                fut .await
                    .unwrap_or_else(|caught_panic| {
                        ::std::panic::resume_unwind(caught_panic.into_panic())
                    })
            };
            Box::pin(fut)
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
                    .unwrap_or_else(|caught_panic| {
                        ::std::panic::resume_unwind(caught_panic.into_panic())
                    })
            };
            Box::pin(fut)
                .into()
        }

        fn dyn_block_on (
            self: &'_ Self,
            future: VirtualPtr<dyn '_ + FfiFuture>,
        )
        {
            self.block_on(future.into_future())
        }

        fn dyn_enter (
            self: &'_ Self,
        ) -> VirtualPtr<dyn '_ + DropGlue>
        {
            Box::new(ImplDropGlue(self.enter())).into()
        }
    }
});

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
        ) -> $crate::prelude::repr_c::Arc<dyn 'static + Send + Sync + Fn()>
        {
            let waker = task_context.waker().clone();
            Arc::new(move || waker.wake_by_ref()).into()
        }
    };
)}
