#![cfg_attr(rustfmt, rustfmt::skip)]
use_prelude!();
use crate::prelude::c_slice;

pub
unsafe
trait HasNiche : ReprC {
    fn is_niche (it: &'_ <Self as ReprC>::CLayout)
      -> bool
    {
        // default implementation (the `is_niche()` heuristic does not need to
        // be 100% accurate, since it's just a sanity check helper):
        Self::is_valid(it).not()
    }
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
        $T:ty => |$it:pat_param| $expr:expr
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

    @for[T : ReprC]
    ptr::NonNull<T> => |it| it.is_null(),
    @for[T : ReprC]
    ptr::NonNullRef<T> => |it| it.is_null(),
    @for[T : ReprC]
    ptr::NonNullMut<T> => |it| it.is_null(),
    @for[T : ReprC]
    ptr::NonNullOwned<T> => |it| it.is_null(),

    @for['__, T : '__ + ReprC]
    c_slice::Mut<'__, T> => |it| it.ptr.is_null(),
    @for['__, T : '__ + ReprC]
    c_slice::Ref<'__, T> => |it| it.ptr.is_null(),
    @for[T : ReprC]
    c_slice::Raw<T> => |it| it.ptr.is_null(),

    // crate::str::Raw => |it| it.ptr.is_null(),
    // str::Ref<'_> => |it| it.ptr.is_null(),

    // char_p::Ref<'_> => |it| it.is_null(),
    // char_p::Raw => |it| it.is_null(),

    // bool => |&it| {
    //     it == unsafe { mem::transmute(None::<bool>) }
    // },
}

cfg_alloc! {
    unsafe_impls! {
        // @for[T : ReprC]
        // repr_c::Box<T> => |it| it.is_null(),
        @for[T : ReprC]
        c_slice::Box<T> => |it| it.ptr.is_null(),
        @for[T : ReprC]
        repr_c::Vec<T> => |it| it.ptr.is_null(),

        // str::Box => |it| it.ptr.is_null(),
        // repr_c::String => |it| it.ptr.is_null(),

        // char_p::Box => |it| it.is_null(),
    }
}

unsafe_impls! {
    @for['out, T : 'out + ReprC]
    Out<'out, T> => |it| it.is_null()
}
