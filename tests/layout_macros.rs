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
    tuple::Tuple2,
};

#[macro_rules_attribute(derive_ReprC!)]
#[repr(u8)]
/// Some docstring
pub
enum MyBool {
    False = 42,
    True,
}

#[macro_rules_attribute(derive_ReprC!)]
#[repr(C)]
/// Some docstring
pub
struct Foo {
    b: MyBool,
    field: ptr::NonNull<u8>,
}

#[test]
fn validity ()
{
    // `Foo_Layout` is `<Foo as ReprC>::CLayout`
    assert!(
        Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: 27 as _ }
        )
    );
    assert!(
        Foo::is_valid(&
            Foo_Layout { b: 43_u8.into(), field: 27 as _ }
        )
    );

    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 0.into(), field: 27 as _ }
        ))
    );
    assert!(
        bool::not(Foo::is_valid(&
            Foo_Layout { b: 42_u8.into(), field: 0 as _ }
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
        a: extern "C" fn (Tuple2<[Foo; 12], MyBool>),
        closure: BoxDynFn2<(), i32, usize>,
    }

    let ref mut out = Vec::new();
    <Crazy as ReprC>::CLayout::c_define_self(&mut MyDefiner {
        out,
        defines: Default::default(),
    });
    println!("{}", String::from_utf8_lossy(out));

    // where
    struct MyDefiner<'out> {
        out: &'out mut dyn io::Write,
        defines: Set<String>,
    }
    impl ::repr_c::layout::Definer for MyDefiner<'_> {
        fn insert (self: &'_ mut Self, name: &'_ str)
          -> bool
        {
            let mut inserted = false;
            self.defines.get_or_insert_with(name, |name| {
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
