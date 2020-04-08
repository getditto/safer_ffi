use ::std::{*,
};

fn main ()
{
    macro_rules! ENV_VAR {() => (
        "REPR_C_GENERATE_HEADERS"
    )}
    println!(concat!("cargo:rerun-if-env-changed=", ENV_VAR!()));
    if env::var(ENV_VAR!()).ok().map_or(false, |it| it == "1") {
        println!("cargo:rustc-cfg=feature=\"headers\"")
    }
}
