#![cfg_attr(rustfmt, rustfmt::skip)]

pub
fn setup ()
{
    #[cfg(feature = "node-js")] {
        ::napi_build::setup();
    }
}
