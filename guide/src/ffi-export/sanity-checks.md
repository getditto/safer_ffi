{{#include ../links.md}}

# `#[ffi_export]`

## Auto-generated sanity checks

The whole design of the [`ReprC`] trait, _i.e._, a trait that expresses that a
type has a C layout, _i.e._, that it has an _associated_ "raw"
[C type][`CType`] (types with no validity invariants whatsoever), means that
the actual `#[no_mangle]`-exported function is one using the associated
[C types][`CType`] in its function signature. This ensures that a foreign call
to such functions (_i.e._, C calling into that function) **will not directly
trigger "instant UB"**, contrary to a hand-crafted definition.

  - Indeed, if you were to export a function such as:

    ```rust,noplaypen
    #[repr(C)]
    enum LogLevel {
        Error,
        Warning,
        Info,
        Debug,
    }

    #[no_mangle] pub extern "C"
    fn set_log_level (level: LogLevel)
    {
        // ...
    }
    ```

    then C code calling `set_log_level` with a value different to the four only
    possible discriminants of `LogLevel` (`0, 1, 2, 3` in this case) would
    instantly trigger Undefined Behavior no matter what the body of
    `set_log_level` would be.

    Instead, when using `repr_c`, the following code:

    ```rust,noplaypen
    use ::repr_c::prelude::*;

    #[derive_ReprC]
    #[repr(u8)] // Associated CType: a plain `u8`
    enum LogLevel {
        Error,
        Warning,
        Info,
        Debug,
    }

    #[ffi_export]
    fn set_log_level (level: LogLevel)
    {
        // ...
    }
    ```

    unsugars to (something along the lines of):

    ```rust,noplaypen
    fn set_log_level (level: LogLevel)
    {
        // ...
    }

    mod hidden {
        #[no_mangle] pub unsafe extern "C"
        fn set_log_level (level: u8)
        {
            match ::repr_c::layout::from_raw(level) {
                | Some(level /* : LogLevel */) => {
                    super::set_log_level(level)
                },
                | None => {
                    // Got an invalid `LogLevel` bit-pattern
                    if compile_time_condition() {
                        eprintln!("Got an invalid `LogLevel` bit-pattern")
                        abort();
                    } else {
                        use ::std::hint::unreachable_unchecked as UB;
                        UB()
                    }
                },
            }
        }
    }
    ```

So, basically, there is an attempt to `transmute` the input
[C type][`CType`] to the expected [`ReprC`] type, but such attempt can fail
if the auto-generated sanity-check detects that so doing would not be
safe (_e.g._, input integer corresponding to no `enum` variant, NULL pointer
when the [`ReprC`] type is guaranteed not to be it, unaligned pointer when
the [`ReprC`] type is guaranteed to be aligned).

### Caveats

Such check cannot be exhaustive (in the case of pointers for instance, `repr_c`
cannot possibly know if it is valid to dereference a non-null and well-aligned
pointer). This means that there are still cases where UB can be triggered
nevertheless, hence it being named a _sanity_ check and not a _safety_ check.

  - Only in the case of a (field-less) `enum` can `repr_c` ensure lack of
    UB no matter the (integral) [C type][`CType`] instance given as input.

As you may notice by looking at the code, there is a `compile_time_condition()`
to actually `abort` instead of triggering UB. This means that when such
condition is not met, UB is actually triggered and we are back to the
`#[no_mangle]` case.

This is by design: such runtime checks may have a performance impact that some
programmers may deem unacceptable, so it is logical that there be some escape
hatch in that regard.

As of this writing, `compile_time_condition()` is currently
`cfg!(debug_assertions)`, which means that by default such checks are
_disabled_ on release.

  - This is not optimal safety-wise, since the default configuration is too
    loose. The author of the crate is aware of that and intending to replace
    that with:

      - [an `unsafe` attribute parameter](
        attributes.md#unsafely-disabling-the-runtime-sanity-checks)
        that would nevertheless only truly opt-out when
        `debug_assertions` are disabled.
