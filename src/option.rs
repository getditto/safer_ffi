#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Option<T> {
    None = 0,
    Some(T) = 1,
}
unsafe impl<T: crate::layout::ReprC> crate::layout::ReprC for Option<T>
where
    crate::tuple::Tuple2<u8, T>: crate::layout::ReprC,
{
    type CLayout = crate::layout::CLayoutOf<crate::tuple::Tuple2<u8, T>>;
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        match unsafe { core::mem::transmute_copy::<_, u8>(it) } {
            0 => true,
            1 => T::is_valid(unsafe {
                &*(it as *const _ as *const u8)
                    .add(core::mem::align_of::<T>())
                    .cast()
            }),
            _ => false,
        }
    }
}

impl<T> From<core::option::Option<T>> for Option<T> {
    fn from(value: core::option::Option<T>) -> Self {
        match value {
            Some(v) => Self::Some(v),
            None => Self::None,
        }
    }
}

impl<T> From<Option<T>> for core::option::Option<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Option::Some(v) => Self::Some(v),
            Option::None => Self::None,
        }
    }
}

impl<T> Option<T> {
    /// Returns a reference to `self`'s contents if it is `Some`.
    pub fn as_ref(&self) -> core::option::Option<&T> {
        match self {
            Self::None => None,
            Self::Some(v) => Some(v),
        }
    }
    /// Returns a mutable reference to `self`'s contents if it is `Some`.
    pub fn as_mut(&mut self) -> core::option::Option<&mut T> {
        match self {
            Self::None => None,
            Self::Some(v) => Some(v),
        }
    }
    /// Applies `f` to `self`'s content if it is some, otherwise calls `default`.
    pub fn map_or_else<U, D: FnOnce() -> U, F: FnOnce(T) -> U>(self, default: D, f: F) -> U {
        match self {
            Option::None => default(),
            Option::Some(v) => f(v),
        }
    }
    /// Unwraps `self`'s value, producing a new value using `default` if it is `None`.
    pub fn unwrap_or_else<D: FnOnce() -> T>(self, default: D) -> T {
        match self {
            Option::None => default(),
            Option::Some(v) => v,
        }
    }
    /// Applies `f` to `self`'s internals to produce an `Option`, returning `None` if `self` was `None`.
    pub fn and_then<U, F: FnOnce(T) -> Option<U>>(self, f: F) -> Option<U> {
        self.map_or_else(|| Option::None, f)
    }
}
