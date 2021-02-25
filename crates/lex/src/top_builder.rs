use wyst_source::{AddSpan, HasLen, Span, Spanned};

use crate::standard::{Delimiter, FlatToken, Quote};

pub struct TopBuilder {
    pos: usize,
}

impl TopBuilder {
    pub fn new() -> TopBuilder {
        TopBuilder { pos: 0 }
    }

    fn consume(&mut self, chars: impl HasLen) -> Span {
        let pre = self.pos;
        self.pos += chars.utf8_len();
        Span::from(pre..self.pos)
    }

    pub fn eof(&mut self) -> Spanned<FlatToken> {
        FlatToken::EOF.spanned(Span::EOF(self.pos.into()))
    }

    pub fn ws(&mut self, chars: impl HasLen) -> Spanned<FlatToken> {
        FlatToken::Whitespace.spanned(self.consume(chars))
    }

    pub fn newline(&mut self) -> Spanned<FlatToken> {
        FlatToken::Newline.spanned(self.consume("\n"))
    }

    pub fn word(&mut self, chars: impl HasLen) -> Spanned<FlatToken> {
        FlatToken::Word.spanned(self.consume(chars))
    }

    pub fn quote(&mut self, chars: impl HasLen, quote: Quote) -> Spanned<FlatToken> {
        let start = self.consume(quote.char());
        let inner = self.consume(chars);
        let end = self.consume(quote.char());

        FlatToken::Quoted((quote, inner)).spanned(start.until(end))
    }

    pub fn comment(
        &mut self,
        chars: impl HasLen,
        (pre, post): (impl HasLen, impl HasLen),
    ) -> Spanned<FlatToken> {
        let start = self.consume(pre);
        let body = self.consume(chars);
        let end = self.consume(post);

        FlatToken::Comment(body).spanned(start.until(end))
    }

    pub fn open(&mut self, delimiter: &str) -> Spanned<FlatToken> {
        match delimiter {
            "(" => FlatToken::Open(Delimiter::Paren).spanned(self.consume("(")),
            "[" => FlatToken::Open(Delimiter::Bracket).spanned(self.consume("[")),
            "{" => FlatToken::Open(Delimiter::Brace).spanned(self.consume("{")),
            other => panic!(
                "unexpected {} character passed to Builder#open (allowed: '(', '[', '{{')",
                other
            ),
        }
    }

    pub fn close(&mut self, delimiter: &str) -> Spanned<FlatToken> {
        match delimiter {
            ")" => FlatToken::Close(Delimiter::Paren).spanned(self.consume(")")),
            "]" => FlatToken::Close(Delimiter::Bracket).spanned(self.consume("]")),
            "}" => FlatToken::Close(Delimiter::Brace).spanned(self.consume("}")),
            other => panic!(
                "unexpected {} character passed to Builder#close (allowed: ')', ']', '}}')",
                other
            ),
        }
    }
}
