/*! \file */
/*******************************************
 *                                         *
 *  File auto-generated by `::safer_ffi`.  *
 *                                         *
 *  Do not manually edit this file.        *
 *                                         *
 *******************************************/

#ifndef __RUST_POINT__
#define __RUST_POINT__

#ifdef __cplusplus
extern "C" {
#endif

/** \brief
 *  A `struct` usable from both Rust and C
 */
typedef struct Point {

    double x;

    double y;

} Point_t;

/** \brief
 *  Returns the middle point of `[a, b]`.
 */
Point_t mid_point (
    Point_t const * a,
    Point_t const * b);


#include <stddef.h>
#include <stdint.h>

/** \remark Has the same ABI as `uint8_t` **/
#ifdef DOXYGEN
typedef enum Figure
#else
typedef uint8_t Figure_t; enum
#endif
{
    /** . */
    FIGURE_CIRCLE,
    /** . */
    FIGURE_SQUARE,
}
#ifdef DOXYGEN
Figure_t
#endif
;

/** \brief
 *  Pretty-prints a point using Rust's formatting logic.
 */
void print_point (
    Point_t const * point);


#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* __RUST_POINT__ */
