use crate::layout::ConcreteReprC;

/// An ABI-stable version of `core::option::Option`.
/// Its usage is expected to be the same as a standard Option, converting to and from said Option when necessary.
///
/// Note that this uses an explicit `bool` flag to store the discriminant.
///
/// In a reduced set of cases of "non-nullable types", _i.e._, types
/// where the null bit-pattern is a niche of the type (which is especially
/// relevant for `&T`, `&mut T`, and <code>[repr_c::Box]\<T\></code>),
/// usage of the stdlib [`Option`][core::option::Option] enables
/// [discriminant elision], wherein the `None` value is represented using
/// the null bitpattern (_e.g._, the `NULL` pointer).
///
/// [repr_c::Box]: crate::prelude::repr_c::Box
/// [discriminant elision]: https://doc.rust-lang.org/1.78.0/core/option/index.html#representation
#[cfg_attr(feature = "stabby", stabby::stabby)]
#[repr(C, u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaggedOption<T> {
    None = 0,
    Some(T) = 1,
}

unsafe impl<T: ConcreteReprC> crate::layout::ReprC for TaggedOption<T>
where
    crate::tuple::Tuple2<bool, crate::layout::CLayoutOf<T>>: crate::layout::ReprC,
{
    type CLayout =
        crate::layout::CLayoutOf<crate::tuple::Tuple2<bool, crate::layout::CLayoutOf<T>>>;
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

impl<T> From<core::option::Option<T>> for TaggedOption<T> {
    fn from(value: core::option::Option<T>) -> Self {
        match value {
            Some(v) => Self::Some(v),
            None => Self::None,
        }
    }
}

impl<T> From<TaggedOption<T>> for core::option::Option<T> {
    fn from(value: TaggedOption<T>) -> Self {
        match value {
            TaggedOption::Some(v) => Self::Some(v),
            TaggedOption::None => Self::None,
        }
    }
}

impl<T> From<T> for TaggedOption<T> {
    fn from(value: T) -> Self {
        Self::Some(value)
    }
}

impl<T> TaggedOption<T> {
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
            Self::None => default(),
            Self::Some(v) => f(v),
        }
    }
    /// Unwraps `self`'s value, producing a new value using `default` if it is `None`.
    pub fn unwrap_or_else<D: FnOnce() -> T>(self, default: D) -> T {
        match self {
            Self::None => default(),
            Self::Some(v) => v,
        }
    }
    /// Applies `f` to `self`'s internals to produce an `Option`, returning `None` if `self` was `None`.
    pub fn and_then<U, F: FnOnce(T) -> TaggedOption<U>>(self, f: F) -> TaggedOption<U> {
        self.map_or_else(|| TaggedOption::None, f)
    }

    /// Takes the current value of `self`, replacing it with `None`
    pub fn take(&mut self) -> Self {
        core::mem::replace(self, TaggedOption::None)
    }

    /// Converts `self` into a standard Rust [Option](core::option::Option).
    pub fn into_rust(self) -> core::option::Option<T> {
        self.into()
    }
}

#[test]
fn option() {
    use crate::layout::ReprC;

    for i in 0..=u8::MAX {
        let expected = Some(i);
        let converted = TaggedOption::from(expected);
        assert!(TaggedOption::<u8>::is_valid(unsafe {
            &core::mem::transmute(converted)
        }));
        assert_eq!(converted, i.into());
        assert_eq!(expected, converted.into());
        assert!(TaggedOption::Some(i)
            .and_then(|_| TaggedOption::<()>::None)
            .into_rust()
            .is_none());
        assert_eq!(unsafe { core::mem::transmute_copy::<_, u8>(&converted) }, 1);
    }
    assert_eq!(
        unsafe { core::mem::transmute_copy::<_, u8>(&TaggedOption::<u8>::None) },
        0
    );
}
