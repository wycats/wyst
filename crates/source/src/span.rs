use wyst_macros::{unit_tests, wyst_data};

#[wyst_data(Copy)]
pub struct Offset {
    byte: usize,
}

impl From<usize> for Offset {
    fn from(byte: usize) -> Self {
        Offset { byte }
    }
}

#[wyst_data(Copy)]
pub struct Span {
    start: Offset,
    end: Offset,
}

impl Span {
    pub fn new(start: impl Into<Offset>, end: impl Into<Offset>) -> Span {
        Span {
            start: start.into(),
            end: end.into(),
        }
    }

    pub(crate) fn slice(self, source: &str) -> &str {
        &source[self.start.byte..self.end.byte]
    }
}

unit_tests!(tests(
    ("Offset#copy", {
        let offset = Offset { byte: 0 };

        fn byte(o: Offset) -> usize {
            o.byte
        }

        // This code is testing Copy
        assert_eq!(byte(offset), 0);
        assert_eq!(byte(offset), 0);
    }),
    ("Span#copy", {
        let span = Span::new(0, 0);

        fn bytes(s: Span) -> (usize, usize) {
            (s.start.byte, s.end.byte)
        }

        // This code is testing Copy
        assert_eq!(bytes(span), (0, 0));
        assert_eq!(bytes(span), (0, 0));
    })
));
