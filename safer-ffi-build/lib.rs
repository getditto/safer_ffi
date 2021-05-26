#![cfg_attr(rustfmt, rustfmt::skip)]

pub
fn setup ()
{
    #[cfg(feature = "node-js")] {
        if ::std::env::var("TARGET").as_deref() != Ok("wasm32-unknown-unknown") {
            ::napi_build::setup();
        }
    }
}
