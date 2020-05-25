# Appendix: A quick reminder of C compilation in Unix

Exporting / generating a C library requires _two_ things:

  - **the header file(s)** (`.h`), which contain the C signatures and thus the
    type (and ABI!) information of the exported functions and types.

      - Such file(s) must be `#include`d at the beginning of the C code, and are
        thus required to compile any C source file that _directly_ calls into
        our Rust functions.

      - It may be necessary to tell the compiler (the C preprocessor, to be
        exact) the path to the folder containg the file(s), by using the `-I`
        flag: `-I path/to/headers/dir`

  - **the object file(s)** (`.o`), or archive (`.a`) of such files (also called
    a _static library_), or a dynamic library (`.so` on Linux, `.dylib` on
    OS X), which contain the machine code with the actual logic of such
    functions.

      - When linking, such file(s) must be referred to:

          - either by full path in the case of `.o` and `.a` files,

          - or, in the case of libraries (`.a` and `.so` / `.dylib`),
            when those are named as `libsome_name.extension`, by using the
            `-l` flag, and feeding it the parameter `some_name`
            (`-l some_name`).

              - It may be necessary to tell the compiler (the linker, to be
                exact) the path to the folder containg the file(s), by using
                the `-L` flag: `-L path/to/libraries/dir`

          - yes, there are _two_ ways to refer to a static library, due to its
            dual nature of being both a library and a "simple" archive of raw
            `.o` files.

        In all cases, "remember" to refer to the library object files _after_
        the files for your downstream binary:

        ```bash
        # Incorrect
        cc -L my_lib/dir -l mylib_name main.o -o main
        # Correct
        cc main.o -L my_lib/dir -l mylib_name -o main
        ```

          - This is because the linker may disregard symbols that are not (yet)
            needed, so the callers need to come before the callees.


## Static _vs._ Dynamic library

If you don't know which to use, **it is highly recommended to use a static
library**. Indeed:

  - Dynamic (also named _shared_) libraries are mainly a file-size optimization
    when having multiple downstream binaries that all depend upon the same
    (_shared_) library, which is quite unlikely to be the case for a Rust
    library.

  - Dynamic libraries result in the produced program being split among multiple
    files (the main binary and the dynamic library), which is not only
    slightly less convinient than a bundled single file, but it also incurs in
    requiring a correct setup system-wise or binary-wise so that the dynamic
    library can be found at _load time_, _i.e._, each and every time the binary
    is run.

    This means the the dynamic library needs to be located:

      - either in special directories such as `/usr/lib`, which may require
        `root` access and/or a special (`make`) `install`ation step.

          - I'd even say that this is, by the way, the main _raison d'être_ for
            tools such as Docker: having a reliable dynamic-library setup is
            so painful that one ends up scripting each and every step of the
            installation process to guarantee that all the tools are correcly
            laid out within the filesystem, and that there are no extraneous
            misinteractions.

      - or in a relative path (`-Wl,-rpath,...` flag), either relative to the
        working directory, or relative to the location of the main binary. In
        both cases this may expose the user to code injection (one can easily
        shadow the dynamic library with their own in such cases), which,
        especially when the main binary has special privileges, is a security
        hazard.

  - When _all_ the libraries used by a binary are static, one gets to have a
    **stand-alone program**, also called "portable" (only across machines of
    the same architecture, though), which, contrary to "setup hell", leads to
    very simple "installation"s (simply copy-paste the binary, and you can run
    it!)

  - That being said, the layer of indirection that dynamic libraries introduce
    can be beneficial or interesting in very special cases, which leads to some
    situations where releasing the library as a dynamic one is mandatory.
    In such cases there is no real choice, and you should be using Rust's
    `cdylib`'s `crate-type` to generate the shared library.
