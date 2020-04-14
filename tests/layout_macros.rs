#![feature(hash_set_entry)]

#[macro_use]
extern crate macro_rules_attribute;

use ::std::{
    collections::HashSet as Set,
    io,
    ptr,
    ops::Not as _,
};
use ::repr_c::{
    closure::boxed::BoxDynFn2,
    layout::{
        CType,
        derive_CType,
        derive_ReprC,
        ReprC,
    },
    slice::*,
    tuple::Tuple2,
};

// #[derive(ReprC)]
#[macro_rules_attribute(derive_ReprC!)]
#[repr(u8)]
/// Some docstring
pub
enum MyBool {
    False = 42,
    True, // = 43
}

derive_ReprC! {
    #[repr(C)]
    /// Some docstring
    pub
    struct Foo['a,] {
        b: MyBool,
        field: RefSlice<'a, u32>,
    }
}

#[test]
fn validity ()
{
    // `Foo_Layout` is `<Foo as ReprC>::CLayout`
    assert!(
        Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: SlicePtr_Layout { ptr: 4 as _, len: 0 } }
        )
    );
    assert!(
        Foo::is_valid(&
            Foo_Layout { b: 43_u8.into(), field: SlicePtr_Layout { ptr: 4 as _, len: 0 } }
        )
    );

    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 0.into(), field: SlicePtr_Layout { ptr: 4 as _, len: 0 } }
        ))
    );
    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: SlicePtr_Layout { ptr: 0 as _, len: 0 } }
        ))
    );
    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: SlicePtr_Layout { ptr: 3 as _, len: 0 } }
        ))
    );
}

#[cfg(feature = "headers")]
#[test]
fn generate_headers ()
{
    #[macro_rules_attribute(derive_ReprC!)]
    #[repr(C)]
    pub
    struct Crazy {
        a: extern "C" fn (extern "C" fn(), Tuple2<[Foo<'static>; 12], ::repr_c::Box<MyBool>>),
        closure: BoxDynFn2<(), i32, usize>,
    }

    let ref mut out =
        ::std::io::stderr()
    ;
    <Crazy as ReprC>::CLayout::c_define_self(&mut MyDefiner {
        out,
        defines: Default::default(),
    });

    // where
    struct MyDefiner<'out> {
        out: &'out mut dyn io::Write,
        defines: Set<String>,
    }
    use ::core::cell::Cell;
    thread_local! {
        static DEPTH: Cell<usize> = Cell::new(0);
    }
    impl ::repr_c::layout::Definer for MyDefiner<'_> {
        fn insert (self: &'_ mut Self, name: &'_ str)
          -> bool
        {
            let mut inserted = false;
            self.defines.get_or_insert_with(name, |name| {
                let depth = DEPTH.with(Cell::get);
                eprintln!("{pad}> Defining `{}`", name, pad = "  ".repeat(depth));
                DEPTH.with(|it| it.set(depth + 1));
                inserted = true;
                name.to_owned()
            });
            inserted
        }

        fn out (self: &'_ mut Self)
          -> &'_ mut dyn io::Write
        {
            &mut self.out
        }
    }
}
