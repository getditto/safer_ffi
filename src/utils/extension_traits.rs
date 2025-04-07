crate::layout::macros::__cfg_headers__! {
    pub(crate) trait StrSeparator {
        fn sep(&self) -> &'static str;
    }

    impl StrSeparator for str {
        fn sep(&self) -> &'static str {
            if self.is_empty() { "" } else { " " }
        }
    }
}
