#[test]
fn test_c_code ()
{
    const C_BINARY: &'static str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        // "/target",
        "/c_binary"
    );
    assert!(
        ::std::process::Command::new("clang")
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"))
            .args(&[
                "-I", "..",
                "-o", C_BINARY,
                "main.c",
                "-L", "..",
                "-l", "ffi_tests",
                // "-Wl,rpath=$ORIGIN/", /* cdylib under Linux */
            ])
            .status()
            .expect("Failed to compile the C binary")
            .success()
    );
    assert!(
        ::std::process::Command::new(C_BINARY)
            .status()
            .expect("Failed to run the C binary")
            .success()
        ,
        "The C test failed."
    );
}
