use ::core::ops::Not as _;

fn main() {
    ::std::process::Command::new("/bin/sh")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .arg("-c")
        .args(::std::env::args_os().nth(1))
        .status()
        .unwrap()
        .success()
        .not()
        .then(|| ::std::process::exit(-1));
}
