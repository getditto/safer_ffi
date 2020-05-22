# `#[repr(C)] struct`

<span class="warning">

TODO: More concrete examples + generics

</span>

```rust,noplaypen
#[derive_ReprC]
#[repr(C)]
pub
struct Point {
    x: i32,
    y: i32,
}

#[ffi_export]
fn get_origin ()
  -> Point
{
    Point { x: 0, y: 0 }
}
```

  - generates:

    ```c
    typedef struct Point {
        int32_t x;
        int32_t y;
    } Point_t;

    Point_t get_origin (void);
    ```
