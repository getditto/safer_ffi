cfg_not_headers! {
    #[macro_export]
    macro_rules! headers_generator {(
        #[test]
        $test_name:ident ()
            => $file_path:expr
    ) => (
        #[test]
        #[ignore]
        fn headers_generator ()
        {
            eprintln!(concat!(
                "The 'headers' feature of `::repr_c` must be enabled for this",
                " to work.",
            ));
        }
    )}
}
cfg_headers! {
    #[macro_export]
    macro_rules! headers_generator {(
        #[test]
        $test_name:ident ()
            => $file_path:expr
    ) => (
        #[test]
        fn $test_name ()
          -> $crate::std::io::Result<()>
        {
            let file_path = $file_path;
            Ok({
                use $crate::std::*;

                let ref mut definer = $crate::headers::HashSetDefiner {
                    out: &mut
                        fs::OpenOptions::new()
                            .create(true)/*or*/.truncate(true)
                            .write(true)
                            .open(file_path)?
                    ,
                    defines_set: default::Default::default(),
                };
                $crate::inventory::iter
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .try_for_each(|::repr_c::TypeDef(define)| define(definer))
                    ?
                ;
            })
        }
    )}
}

cfg_headers! {
    pub
    struct HashSetDefiner<'out> {
        pub
        out: &'out mut dyn ::std::io::Write,

        pub
        defines_set: ::std::collections::HashSet<String>,
    }

    impl crate::layout::Definer
        for HashSetDefiner<'_>
    {
        fn insert (self: &'_ mut Self, name: &'_ str)
          -> bool
        {
            self.defines_set
                .insert(name.to_owned())
        }

        fn out (self: &'_ mut Self)
          -> &'_ mut dyn ::std::io::Write
        {
            &mut self.out
        }
    }
}
