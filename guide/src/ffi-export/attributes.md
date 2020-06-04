{{#include ../links.md}}

# `#[ffi_export]`

## Attributes

<div class="warning">

These are not yet implemented

</div>

  - **Non-`"C"` ABIs**

    Currently `#[ffi_export]` defaults to a `#[no_mangle] pub extern "C"`
    function definition, _i.e._, it exports a function using the default C ABI
    of the platform it is compiled against (_target_ platform).

    Sometimes a special ABI is required, in which case specifying the ABI is
    desirable.

    **Imagined syntax**: an optional `ABI = "<abi>"` attribute parameter:

    ```rust,noplaypen
    #[ffi_export(ABI = "system")]
    fn ...
    ```

  - **Custom `export_name`**.

    To override the name (the _symbol_) the item is exported with (by virtue of
    the default `#[no_mangle]`, the item is exported with a symbol equal to the
    identifier used for its name), one could imagine someone wanting to develop
    their own namespacing tool / name mangling convention when controling both
    ends of the FFI, so they may want to provide an `export_name` override too.

    **Imagined syntax**: an optional `export_name = ...` attribute parameter.

  - **`unsafe`-ly disabling the runtime [sanity checks]**.

    <span id="unsafely-disabling-the-runtime-sanity-checks"></span>

    As mentioned in the [sanity checks] section, it is intended that all
    `#[ffi_export]`-ed functions perform some sanity checks on the raw inputs
    they receive, before transmuting those to the actual [`ReprC`] types.
    Still, for some functions where performance is critical and the caller
    of the `#[ffi_export]`-ed function is trusted not give invalid values,
    it will be possible to opt-out of such check when `debug_assertions` are
    disabled by marking each function where one wants to disable the checks
    with an `unsafe` parameter, such as:

    ```rust,noplaypen
    #[ffi_export]
    #[safer_ffi(unsafe { skip_sanity_checks() })]
    fn ...
    ```

    or on specific params:

    ```rust,noplaypen
    #[ffi_export]
    fn set_log_level (
        #[safer_ffi(unsafe { skip_sanity_checks() })]
        level: LogLevel,
        ...
    ) -> ...
    ```
