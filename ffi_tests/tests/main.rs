#![cfg_attr(rustfmt, rustfmt::skip)]
#[test]
fn test_c_code ()
{
    const C_BINARY: &'static str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        // "/target",
        "/c_binary"
    );
    // _e.g._, `-lSystem -lresolv -lc -lm`
    let ref native_static_libs =
        String::from_utf8(
            ::std::process::Command::new("/bin/bash")
                .args(&["-c", r#"
                    rustc \
                        --print native-static-libs \
                        --crate-type staticlib \
                        -</dev/null \
                        2>&1 \
                        >/dev/null \
                    | grep native-static-libs \
                    | cut -d' ' -f3-
                "#])
                .output()
                .unwrap()
                .stdout
        )
        .unwrap()
    ;
    let mut clang_cmd = ::scopeguard::guard_on_unwind(
        ::std::process::Command::new("clang"),
        |clang_cmd| {
            println!("Clang command: `{:?}`", clang_cmd);
            println!("Command run in: `{:?}`", ::std::env::current_dir());
        },
    );
    assert!(
        clang_cmd
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/c"))
            .args(&[
                "-I", ".",
                "-o", C_BINARY,
                "main.c",
                "-L", "../..", "-l", "ffi_tests",
                // "-Wl,rpath=$ORIGIN/", /* cdylib under Linux */
            ])
            // Add extra necessary `-l` flags
            .args(native_static_libs.split(' ').map(str::trim))
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

#[test]
fn test_lua_code()
{
    let output = ::std::process::Command::new("luajit")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/lua"))
        .arg("tests.lua")
        .output()
        .expect("Failed to run Lua tests");

    assert!(
        output.status.success(),
        "Lua tests failed with output:\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
