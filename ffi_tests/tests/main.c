#include <assert.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "generated.h"

void cb (void *, char const *);

#define SLICE_REF(ty, ...) /* __VA_ARGS__ is array input */ \
    (slice_ref_ ## ty) { \
        .ptr = __VA_ARGS__, \
        .len = sizeof(__VA_ARGS__) / sizeof(ty), \
    }

int main (
    int argc,
    char const * const argv[])
{
    // test concat
    {
        char * s = concat("Hello, ", "World!");
        assert(strcmp(s, "Hello, World!") == 0);
        free_char_p(s);
    }

    // test with_concat
    {
        bool called = false;
        with_concat(
            "Hello, ",
            "World!",
            (RefDynFnMut1_void_char_const_ptr_t) {
                .env_ptr = (void *) &called,
                .call = cb,
            }
        );
        assert(called == true);
    }

    // test max
    {
        int32_t ints_array[] = { -27, -42, 9, -8 };
        assert(
            *max(SLICE_REF(int32_t, ints_array))
            ==
            9
        );
    }

    // test max empty
    {
        assert(
            max(SLICE_REF(int32_t, (int32_t []) {}))
            ==
            NULL
        );
    }

    return EXIT_SUCCESS;
}

void cb (
    void * called,
    char const * s)
{
    *(bool *)called = true;
    assert(strcmp(s, "Hello, World!") == 0);
}
