#![cfg_attr(rustfmt, rustfmt::skip)]
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
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/c"))
            .args(&[
                "-I", ".",
                "-o", C_BINARY,
                "main.c",
                "-L", "../..", "-l", "ffi_tests",
                "-l", "pthread", "-l", "dl", // For Linux
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

#[cfg(target_os = "macos")]
#[test]
fn test_csharp_code ()
{
    assert!(
        ::std::process::Command::new("/bin/ln")
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/csharp"))
            .args(&["-sf", "../../libffi_tests.dylib"])
            .status()
            .expect("Failed to symlink the Rust dynamic library")
            .success()
    );
    assert!(
        ::std::process::Command::new("dotnet")
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/csharp"))
            .arg("run")
            .status()
            .expect("Failed to compile the C binary")
            .success()
    );
}
