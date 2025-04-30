#include <assert.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "generated.h"

#define SLICE_REF(ty, ...) /* __VA_ARGS__ is array input */ \
    (slice_ref_ ## ty) { \
        .ptr = __VA_ARGS__, \
        .len = sizeof(__VA_ARGS__) / sizeof(ty), \
    }

void cb (
    void * called,
    char const * s)
{
    *(bool *)called = true;
    assert(strcmp(s, "Hello, World!") == 0);
}

bool foo_cb_called = false;
void foo_cb (
    foo_t * foo)
{
    assert(
        read_foo(foo)
        ==
        42
    );
    foo_cb_called = true;
}

static atomic_int X = 0;
void arc_call (
    atomic_int * x)
{
    puts("Called from the other side");
}
void arc_retain (
    atomic_int * x)
{
    *x += 1;
}
void arc_release (
    atomic_int * x)
{
    *x -= 1;
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

    // test foo
    {
        foo_t * foo = new_foo();
        assert(
            read_foo(foo)
            ==
            42
        );
        free_foo(foo);
        free_foo(NULL);
    }

    // test foo, cb version
    with_foo(foo_cb);
    assert(foo_cb_called == true);

    // test async_get_ft
    assert(async_get_ft() == 42);

    // test constant
    assert(FOO == 42);

    // test the currified thing
    assert(returns_a_fn_ptr()(0x42) == 0x4200);

    X = 1;
    call_in_the_background((ArcDynFn0_void_t) {
        .env_ptr = ((void *) &X),
        .call = ((void (*)(void *)) arc_call),
        .release = ((void (*)(void *)) arc_release),
        .retain = ((void (*)(void *)) arc_retain),
    });
    while (X > 0);

    assert(my_renamed_ptr_api() == (void *)0xbad000);

    puts("C: [ok]");

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated"
    deprecated_function_1();
    deprecated_function_2();
#pragma GCC diagnostic pop

    return EXIT_SUCCESS;
}
