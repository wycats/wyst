use wyst_core::unit_tests;

pub trait HasLen {
    fn utf8_len(&self) -> usize;
}

impl HasLen for char {
    fn utf8_len(&self) -> usize {
        self.len_utf8()
    }
}

impl HasLen for &char {
    fn utf8_len(&self) -> usize {
        self.len_utf8()
    }
}

impl HasLen for &str {
    fn utf8_len(&self) -> usize {
        self.len()
    }
}

impl HasLen for String {
    fn utf8_len(&self) -> usize {
        self.len()
    }
}

unit_tests!(tests(
    ("HasLen for char", { assert_eq!(HasLen::utf8_len(&'h'), 1) }),
    ("HasLen for &char", {
        assert_eq!(HasLen::utf8_len(&&'h'), 1)
    }),
));
