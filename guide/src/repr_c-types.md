|                         | Traditional FFI | `repr_c`                 |
|-------------------------|-----------------|--------------------------|
| Mutable pointer or NULL | `*mut T`        | `Option<&mut T>`         |
| Mutable pointer         | `*mut T`        | `&mut T`                 |
| Owned pointer or NULL   | `*mut T`        | `Option<repr_c::Box<T>>` |
| Owned pointer           | `*mut T`        | `repr_c::Box<T>`         |
