# `#[repr({integer})]` (fieldless) `enum`

<span class="warning">

TODO: lengthy explanation of why `#[repr(C)] enum`s are _bad_

</span>

```rust,noplaypen
#[derive_ReprC]
#[repr(i8)]
pub
enum Direction {
    Up = 1,
    Down = -1,
}

#[ffi_export]
fn is_up (dir: Direction)
  -> bool
{
    matches!(dir, Direction::Up)
}
```

  - generates:

    ```c
    typedef int8_t Direction_t; enum {
        DIRECTION_UP = 1,
        DIRECTION_DOWN = -1,
    };

    bool is_up(Direction_t dir);
    ```
