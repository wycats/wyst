use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use wyst_core::{unit_tests, wyst_copy, wyst_display, Display, WystData};

use crate::{len::HasLen, Spanned};

#[wyst_copy]
#[derive(Display)]
pub struct Offset {
    byte: usize,
}

impl Add<char> for Offset {
    type Output = Offset;

    fn add(self, rhs: char) -> Self::Output {
        Offset {
            byte: self.byte + rhs.len_utf8(),
        }
    }
}

impl Sub<char> for Offset {
    type Output = Offset;

    fn sub(self, rhs: char) -> Self::Output {
        Offset {
            byte: self.byte - rhs.len_utf8(),
        }
    }
}

impl From<usize> for Offset {
    fn from(byte: usize) -> Self {
        Offset { byte }
    }
}

impl From<&usize> for Offset {
    fn from(byte: &usize) -> Self {
        Offset { byte: *byte }
    }
}

impl Into<usize> for Offset {
    fn into(self) -> usize {
        self.byte
    }
}

impl Offset {
    pub fn char_span(self, char: impl HasLen) -> Span {
        Span::new(self, self.byte + char.utf8_len())
    }
}

#[wyst_copy]
pub enum Span {
    Interior(InteriorSpan),
    EOF(Offset),
}

#[wyst_display("{}..{}", "self.start", "self.end")]
#[wyst_copy]
pub struct InteriorSpan {
    start: Offset,
    end: Offset,
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Span::new(range.start, range.end)
    }
}

impl Span {
    pub fn new(start: impl Into<Offset>, end: impl Into<Offset>) -> Span {
        Span::Interior(InteriorSpan {
            start: start.into(),
            end: end.into(),
        })
    }

    pub fn map<U>(self, value: impl FnOnce() -> U) -> Spanned<U>
    where
        U: WystData,
    {
        Spanned::new(self, value())
    }

    pub fn start(self) -> Offset {
        match self {
            Span::Interior(span) => span.start,
            Span::EOF(offset) => offset,
        }
    }

    pub fn end(self) -> Offset {
        match self {
            Span::Interior(span) => span.end,
            Span::EOF(offset) => offset,
        }
    }

    pub fn eof(offset: impl Into<Offset>) -> Span {
        Span::EOF(offset.into())
    }

    pub fn until(self, end: Span) -> Span {
        match (self, end) {
            (Span::Interior(from), Span::Interior(to)) => Span::new(from.start, to.end),
            (Span::Interior(from), Span::EOF(to)) => Span::new(from.start, to),
            (Span::EOF(from), Span::EOF(_)) => Span::EOF(from),
            (Span::EOF(from), Span::Interior(to)) => {
                if from == to.end {
                    Span::EOF(from)
                } else {
                    panic!("Cannot create a Span that starts from EOF and ends before EOF (you tried: from=EOF({}), to={})", from, to)
                }
            }
        }
    }

    // pub fn char(start: impl Into<Offset>, )

    pub(crate) fn slice(self, source: &str) -> &str {
        match self {
            Span::Interior(span) => &source[span.start.byte..span.end.byte],
            Span::EOF(_) => "",
        }
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
    ("Offset#char_span", {
        assert_eq!(Offset::from(0).char_span('h'), Span::new(0, 'h'.len_utf8()));
        assert_eq!(
            Offset::from(1).char_span('e'),
            Span::new(1, 1 + 'e'.len_utf8())
        );
    }),
    ("Span#copy", {
        let span = Span::new(0, 0);

        fn bytes(s: Span) -> (usize, usize) {
            match s {
                Span::EOF(_) => panic!("unexpected eof"),
                Span::Interior(s) => (s.start.byte, s.end.byte),
            }
        }

        // This code is testing Copy
        assert_eq!(bytes(span), (0, 0));
        assert_eq!(bytes(span), (0, 0));
    })
));
