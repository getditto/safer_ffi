#include <stdlib.h>

#include "rust_points.h"

int main (int argc, char const * const argv[])
{
    Point_t a = { .x = 84, .y = 45 };
    Point_t b = { .x = 0, .y = 39 };
    Point_t m = mid_point(&a, &b);
    print_point(&m);
    return EXIT_SUCCESS;
}
