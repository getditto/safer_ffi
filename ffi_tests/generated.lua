-- File auto-generated by `::safer_ffi`.
--
-- Do not manually edit this file.

local ffi = require "ffi"

ffi.cdef [[

// <No documentation available>
// enum has the same ABI as `uint8_t`
typedef enum Wow {
    // <No documentation available>
    WOW_LEROY,
    // <No documentation available>
    WOW_JENKINS,
}; typedef uint8_t Wow_t;

// <No documentation available>
typedef struct AnUnusedStruct {
    // <No documentation available>
    Wow_t are_you_still_there;
} AnUnusedStruct_t;

// <No documentation available>
typedef struct ArraysStruct {
    // <No documentation available>
    float floats[3];

    // <No documentation available>
    uint64_t sizes[5];

    // <No documentation available>
    uint8_t dim_2[2][1];

    // <No documentation available>
    uint8_t dim_3[3][2][1];
} ArraysStruct_t;

// <No documentation available>
static const int32_t FOO = 42;

// <No documentation available>
// enum has the same ABI as `int8_t`
typedef enum Bar {
    // <No documentation available>
    BAR_A = 43,
    // <No documentation available>
    BAR_B = 42,
}; typedef int8_t Bar_t;

// Hello, `World`!
typedef struct next_generation {
    // I test some `gen`-eration.
    Bar_t generation;

    // with function pointers and everything!
    void * (*cb)(bool);
} next_generation_t;

// The layout of `&str` is opaque/subject to changes.
typedef struct Opaque__str Opaque__str_t;

// <No documentation available>
extern const char* SOME_NAME;

// <No documentation available>
typedef struct ConstGenericStruct_uint8_1 {
    // <No documentation available>
    uint8_t data[1];
} ConstGenericStruct_uint8_1_t;

// <No documentation available>
typedef struct ConstGenericStruct_uint8_2 {
    // <No documentation available>
    uint8_t data[2];
} ConstGenericStruct_uint8_2_t;

// <No documentation available>
typedef struct ConstGenericStruct_uint16_3 {
    // <No documentation available>
    uint16_t data[3];
} ConstGenericStruct_uint16_3_t;

// <No documentation available>
typedef struct SpecificConstGenericContainer {
    // <No documentation available>
    ConstGenericStruct_uint8_1_t field1;

    // <No documentation available>
    ConstGenericStruct_uint8_2_t field2;

    // <No documentation available>
    ConstGenericStruct_uint16_3_t field3;
} SpecificConstGenericContainer_t;

// Hello, `World`!
// enum has the same ABI as `uint8_t`
typedef enum triforce {
    // <No documentation available>
    TRIFORCE_DIN = 3,
    // <No documentation available>
    TRIFORCE_FARORE = 1,
    // <No documentation available>
    TRIFORCE_NARYU,
}; typedef uint8_t triforce_t;

// https://github.com/getditto/safer_ffi/issues/45
int32_t
_issue_45 (
    int32_t __arg_0);

// <No documentation available>
typedef struct Enum Enum_t;

// <No documentation available>
Enum_t *
_my_enum_is_opaque (void);

// The layout of `alloc::string::String` is opaque/subject to changes.
typedef struct Opaque_String Opaque_String_t;

// <No documentation available>
Opaque_String_t *
_some_opaque_std_lib_type (void);

// <No documentation available>
int32_t
async_get_ft (void);

// `Arc<dyn Send + Sync + Fn() -> Ret>`
typedef struct ArcDynFn0_void {
    // <No documentation available>
    void * env_ptr;

    // <No documentation available>
    void (*call)(void *);

    // <No documentation available>
    void (*release)(void *);

    // <No documentation available>
    void (*retain)(void *);
} ArcDynFn0_void_t;

// <No documentation available>
void
call_in_the_background (
    ArcDynFn0_void_t f);

// This is a `#[repr(C)]` enum, which leads to a classic enum def.
typedef enum SomeReprCEnum {
    // This is some variant.
    SOME_REPR_C_ENUM_SOME_VARIANT,
} SomeReprCEnum_t;

// <No documentation available>
void
check_SomeReprCEnum (
    SomeReprCEnum_t _baz);

// <No documentation available>
void
check_bar (
    Bar_t _bar);

// Concatenate the two input strings into a new one.
//
// The returned string must be freed using `free_char_p`.
char *
concat (
    char const * fst,
    char const * snd);

// Frees a string created by `concat`.
void
free_char_p (
    char * _string);

// <No documentation available>
typedef struct foo foo_t;

// <No documentation available>
void
free_foo (
    foo_t * foo);

// `&'lt [T]` but with a guaranteed `#[repr(C)]` layout.
//
// # C layout (for some given type T)
//
// ```c
// typedef struct {
// // Cannot be NULL
// T * ptr;
// size_t len;
// } slice_T;
// ```
//
// # Nullable pointer?
//
// If you want to support the above typedef, but where the `ptr` field is
// allowed to be `NULL` (with the contents of `len` then being undefined)
// use the `Option< slice_ptr<_> >` type.
typedef struct slice_ref_int32 {
    // Pointer to the first element (if any).
    int32_t const * ptr;

    // Element count
    size_t len;
} slice_ref_int32_t;

// Returns a pointer to the maximum integer of the input slice, or `NULL` if
// it is empty.
int32_t const *
max (
    slice_ref_int32_t xs);

// <No documentation available>
void *
my_renamed_ptr_api (void);

// <No documentation available>
foo_t *
new_foo (void);

// <No documentation available>
int32_t
read_foo (
    foo_t const * foo);

// <No documentation available>
uint16_t (*
returns_a_fn_ptr (void))(uint8_t);

// The layout of `core::task::wake::Context` is opaque/subject to changes.
typedef struct Opaque_Context Opaque_Context_t;

// <No documentation available>
ArcDynFn0_void_t
rust_future_task_context_get_waker (
    Opaque_Context_t const * task_context);

// <No documentation available>
void
rust_future_task_context_wake (
    Opaque_Context_t const * task_context);

// <No documentation available>
typedef struct Erased Erased_t;

// An FFI-safe `Poll<()>`.
// enum has the same ABI as `int8_t`
typedef enum PollFuture {
    // <No documentation available>
    POLL_FUTURE_COMPLETED = 0,
    // <No documentation available>
    POLL_FUTURE_PENDING = -1,
}; typedef int8_t PollFuture_t;

// <No documentation available>
typedef struct FfiFutureVTable {
    // <No documentation available>
    void (*release_vptr)(Erased_t *);

    // <No documentation available>
    PollFuture_t (*dyn_poll)(Erased_t *, Opaque_Context_t *);
} FfiFutureVTable_t;

// <No documentation available>
typedef struct VirtualPtr__Erased_ptr_FfiFutureVTable {
    // <No documentation available>
    Erased_t * ptr;

    // <No documentation available>
    FfiFutureVTable_t vtable;
} VirtualPtr__Erased_ptr_FfiFutureVTable_t;

// `Box<dyn 'static + Send + FnMut() -> Ret>`
typedef struct BoxDynFnMut0_void {
    // <No documentation available>
    void * env_ptr;

    // <No documentation available>
    void (*call)(void *);

    // <No documentation available>
    void (*free)(void *);
} BoxDynFnMut0_void_t;

// <No documentation available>
typedef struct DropGlueVTable {
    // <No documentation available>
    void (*release_vptr)(Erased_t *);
} DropGlueVTable_t;

// <No documentation available>
typedef struct VirtualPtr__Erased_ptr_DropGlueVTable {
    // <No documentation available>
    Erased_t * ptr;

    // <No documentation available>
    DropGlueVTable_t vtable;
} VirtualPtr__Erased_ptr_DropGlueVTable_t;

// <No documentation available>
typedef struct FfiFutureExecutorVTable {
    // <No documentation available>
    void (*release_vptr)(Erased_t *);

    // <No documentation available>
    Erased_t * (*retain_vptr)(Erased_t const *);

    // <No documentation available>
    VirtualPtr__Erased_ptr_FfiFutureVTable_t (*dyn_spawn)(Erased_t const *, VirtualPtr__Erased_ptr_FfiFutureVTable_t);

    // <No documentation available>
    VirtualPtr__Erased_ptr_FfiFutureVTable_t (*dyn_spawn_blocking)(Erased_t const *, BoxDynFnMut0_void_t);

    // <No documentation available>
    void (*dyn_block_on)(Erased_t const *, VirtualPtr__Erased_ptr_FfiFutureVTable_t);

    // <No documentation available>
    VirtualPtr__Erased_ptr_DropGlueVTable_t (*dyn_enter)(Erased_t const *);
} FfiFutureExecutorVTable_t;

// <No documentation available>
typedef struct VirtualPtr__Erased_ptr_FfiFutureExecutorVTable {
    // <No documentation available>
    Erased_t * ptr;

    // <No documentation available>
    FfiFutureExecutorVTable_t vtable;
} VirtualPtr__Erased_ptr_FfiFutureExecutorVTable_t;

// <No documentation available>
int32_t
test_spawner (
    VirtualPtr__Erased_ptr_FfiFutureExecutorVTable_t executor);

// `&'lt mut (dyn 'lt + Send + FnMut(A1) -> Ret)`
typedef struct RefDynFnMut1_void_char_const_ptr {
    // <No documentation available>
    void * env_ptr;

    // <No documentation available>
    void (*call)(void *, char const *);
} RefDynFnMut1_void_char_const_ptr_t;

// Same as `concat`, but with a callback-based API to auto-free the created
// string.
void
with_concat (
    char const * fst,
    char const * snd,
    RefDynFnMut1_void_char_const_ptr_t cb);

// <No documentation available>
bool
with_foo (
    void (*cb)(foo_t *));

]]