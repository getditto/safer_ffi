use {
    ::core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    },
    ::safer_ffi::{
        prelude::*,
    },
    super::{
        *,
    },
};

/// An FFI-safe `Poll<()>`.
#[derive_ReprC]
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub
enum PollFuture {
    Success,
    Pending,
}

/// Models a `Future` resolving to `()`.
#[derive_ReprC(dyn)]
pub
trait FfiFuture {
    fn dyn_poll (self: Pin<&mut Self>, ctx: &'_ mut Context<'_>)
      -> PollFuture
    ;
}

impl<F : Future<Output = ()>> FfiFuture for F {
    fn dyn_poll (self: Pin<&mut Self>, ctx: &'_ mut Context<'_>)
      -> PollFuture
    {
        match Future::poll(self, ctx) {
            | Poll::Pending => PollFuture::Pending,
            | Poll::Ready(()) => PollFuture::Success,
        }
    }
}

match_! {([] [Send]) {( $([ $($Send:ident)? ])* ) => (
    $(
        impl VirtualPtr<dyn '_ + $($Send +)? FfiFuture> {
            pub
            async fn into_future (mut self)
            {
                ::futures::future::poll_fn(
                    move |cx| match Pin::new(&mut self).dyn_poll(cx) {
                        | PollFuture::Pending => Poll::Pending,
                        | PollFuture::Success => Poll::Ready(()),
                    }
                )
                .await
            }
        }
    )*
)}}

pub use executor::*;
mod executor;

#[cfg(test)]
mod tests;
