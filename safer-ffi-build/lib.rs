pub
fn setup ()
{
    #[cfg(feature = "node-js")] {
        ::napi_build::setup();
    }
}
