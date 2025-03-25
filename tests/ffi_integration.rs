use ::core::ops::Not as _;

#[cfg_attr(
    not(feature = "ffi-tests"),
    ignore = "\
        ⚠️  the integration `ffi-tests` (C, C#, lua) are not being run.\n\
        Please add `--features ffi-tests`, or directly run `make -C ffi_tests`.\
    "
)]
#[test]
fn ffi_tests() {
    ::std::process::Command::new("make")
        .args(["-C", "ffi_tests"])
        .status()
        .expect("`make …` command to exist")
        .success()
        .not()
        .then(|| panic!());
}
