pub
fn setup ()
{
    #[cfg(feature = "js")] {
        if ::std::env::var("TARGET").as_ref().map(|s| &**s) != Ok("wasm32-unknown-unknown") {
            ::napi_build::setup();
        }
    }
}
