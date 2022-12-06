use super::*;

#[cfg(feature = "tokio")]
#[test]
fn async_test ()
{
    let runtime = ::tokio::runtime::Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    // Add pointer indirection to allow type erasure,
    // and then `.into()` takes care of virtualizing the pointer.
    let ffi_future_executor = Box::new(handle).into();
    let x = test_spawner(ffi_future_executor);
    assert_eq!(x, 42);
}

#[ffi_export]
fn test_spawner (
    executor: VirtualPtr<dyn 'static + FfiFutureExecutor>,
) -> i32
{
    let x: i32 = executor.block_on(async {
        let x: i32 =
            executor.spawn(async {
                async {}.await;
                42
            })
            .await
        ;
        x
    });
    x
}

crate::ffi_export_future_helpers!();
