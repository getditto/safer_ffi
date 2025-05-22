#![cfg_attr(rustfmt, rustfmt::skip)]
use_prelude!();

use crate::prelude::c_slice;

__cfg_headers__! {
    use crate::__::{Definer, HeaderLanguage};
    use crate::layout::impls::metadata_nested_type_usage;
}

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
    type CLayout = OptionCLayout<T::CLayout>;

    #[inline]
    fn is_valid (it: &'_ Self::CLayout) -> bool {
        T::is_niche(&it.wrappedCLayout) || <T as ReprC>::is_valid(&it.wrappedCLayout)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OptionCLayout<T : CType> {
    wrappedCLayout: T,
}

unsafe
impl<T : CType> CType for OptionCLayout<T> {

    type OPAQUE_KIND = T::OPAQUE_KIND;

    __cfg_headers__! {
        fn short_name() -> String {
            T::short_name()
        }

        fn define_self__impl(language: &'_ dyn HeaderLanguage, definer: &'_ mut dyn Definer) -> io::Result<()> {
            T::define_self__impl(language, definer)
        }

        fn define_self(language: &'_ dyn HeaderLanguage, definer: &'_ mut dyn Definer) -> io::Result<()> {
            T::define_self(language, definer)
        }

        fn name(_language: &'_ dyn HeaderLanguage) -> String {
            T::name(_language)
        }

        fn name_wrapping_var(language: &'_ dyn HeaderLanguage, var_name: &'_ str) -> String {
            T::name_wrapping_var(language, var_name)
        }

        fn csharp_marshaler() -> Option<String> {
            T::csharp_marshaler()
        }

        fn metadata_type_usage() -> String {
            let nested_type = metadata_nested_type_usage::<T>();

            format!("\"kind\": \"{}\",\n\"type\": {{\n{}\n}}", "Optional", nested_type)
        }
    }
}

unsafe
impl<T : ReprC + CType> ReprC for OptionCLayout<T> {

    type CLayout = T::CLayout;

    fn is_valid(it: &'_ Self::CLayout) -> bool {
        T::is_valid(it)
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
