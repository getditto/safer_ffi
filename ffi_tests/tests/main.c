#include <assert.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

#include "generated.h"

void cb (void *, char const *);

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
        slice_ref_int32_t ints = {
            .ptr = ints_array,
            .len = sizeof(ints_array) / sizeof(*ints_array),
        };
        assert(*max(ints) == 9);
    }

    // test max empty
    {
        int32_t ints_array[] = { };
        slice_ref_int32_t ints = {
            .ptr = ints_array,
            .len = sizeof(ints_array) / sizeof(*ints_array),
        };
        assert(max(ints) == NULL);
    }

    return EXIT_SUCCESS;
}

void cb (
    void * called,
    char const * s)
{
    *(bool *) called = true;
    assert(strcmp(s, "Hello, World!") == 0);
}
