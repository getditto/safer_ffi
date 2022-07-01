#![cfg_attr(rustfmt, rustfmt::skip)]

fn main() {
    if cfg!(feature = "proc_macros") {
        println!("cargo:warning=[safer-ffi] \
            `proc-macros` feature is deprecated and will be removed\
        ");
    }
}
