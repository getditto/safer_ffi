crate::layout::macros::__cfg_headers__! {
    pub(crate) trait StrSeparator {
        fn sep(&self) -> &'static str;
        fn or_empty(&self) -> &dyn ::core::fmt::Display;
    }

    impl StrSeparator for &str {
        fn sep(&self) -> &'static str {
            if self.is_empty() { "" } else { " " }
        }
        fn or_empty(&self) -> &dyn ::core::fmt::Display {
            self
        }
    }

    impl StrSeparator for Option<&dyn ::core::fmt::Display> {
        fn sep(&self) -> &'static str {
            if self.is_none() { "" } else { " " }
        }
        fn or_empty(&self) -> &dyn ::core::fmt::Display {
            self.unwrap_or(&"")
        }
    }
}
