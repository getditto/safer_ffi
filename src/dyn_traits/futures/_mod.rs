//! See [the dedicated secion of the guide](https://getditto.github.io/safer_ffi/dyn_traits/futures.html).

use ::core::future::Future;
use ::core::pin::Pin;
use ::core::task::Context;
use ::core::task::Poll;
use ::safer_ffi::prelude::*;

use super::*;

/// An FFI-safe `Poll<()>`.
#[derive_ReprC]
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollFuture {
    Completed = 0,
    Pending = -1,
}

/// Models a `Future` resolving to `()`.
#[derive_ReprC(dyn)]
pub trait FfiFuture {
    fn dyn_poll(
        self: Pin<&mut Self>,
        ctx: &'_ mut Context<'_>,
    ) -> PollFuture;
}

impl<F: Future<Output = ()>> FfiFuture for F {
    fn dyn_poll(
        self: Pin<&mut Self>,
        ctx: &'_ mut Context<'_>,
    ) -> PollFuture {
        match Future::poll(self, ctx) {
            | Poll::Pending => PollFuture::Pending,
            | Poll::Ready(()) => PollFuture::Completed,
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
                        | PollFuture::Completed => Poll::Ready(()),
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
