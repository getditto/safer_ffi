use super::*;

pub
unsafe
trait HasNiche : ReprC {
    fn is_niche (it: &'_ <Self as ReprC>::CLayout)
      -> bool
    ;
}

unsafe
impl<T : ReprC + HasNiche> ReprC
    for Option<T>
{
    type CLayout = <T as ReprC>::CLayout;

    #[inline]
    fn is_valid (it: &'_ Self::CLayout)
      -> bool
    {
        T::is_niche(it) || <T as ReprC>::is_valid(it)
    }
}

macro_rules! unsafe_impls {(
    $(
        $(@for[$($generics:tt)*])?
        $T:ty => |$it:pat| $expr:expr
    ),* $(,)?
) => (
    $(
        unsafe
        impl$(<$($generics)*>)? HasNiche
            for $T
        {
            #[inline]
            fn is_niche ($it: &'_ <Self as ReprC>::CLayout)
              -> bool
            {
                $expr
            }
        }
    )*
)}

unsafe_impls! {
    @for['__, T : '__ + ReprC]
    &'__ T => |it| it.is_null(),
    @for['__, T : '__ + ReprC]
    &'__ mut T => |it| it.is_null(),

    @for['__, T : '__ + ReprC]
    crate::slice_mut<'__, T> => |it| it.ptr.is_null(),
    @for['__, T : '__ + ReprC]
    crate::slice_ref<'__, T> => |it| it.ptr.is_null(),
    @for[T : ReprC]
    crate::slice::slice_raw<T> => |it| it.ptr.is_null(),

    // crate::str_raw => |it| it.ptr.is_null(),
    crate::str_ref<'_> => |it| it.ptr.is_null(),

    char_p::Ref<'_> => |it| it.is_null(),
    char_p::char_p_raw => |it| it.is_null(),

    bool => |&it| {
        it == unsafe { mem::transmute(None::<bool>) }
    },
}

cfg_alloc! {
    unsafe_impls! {
        @for[T : ReprC]
        Box<T> => |it| it.is_null(),
        @for[T : ReprC]
        crate::slice_boxed<T> => |it| it.ptr.is_null(),
        @for[T : ReprC]
        Vec<T> => |it| it.ptr.is_null(),

        crate::str_boxed => |it| it.ptr.is_null(),
        String => |it| it.ptr.is_null(),

        char_p::Boxed => |it| it.is_null(),
    }
}
