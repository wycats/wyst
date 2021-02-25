use wyst_source::{AddSpan, HasLen, Span, Spanned};

use crate::{
    delegate::QuoteResult,
    standard::{Delimiter, FlatToken, Quote},
    tree::{Leaf, Token},
};

pub struct TokenBuilder {
    pos: usize,
}

impl TokenBuilder {
    pub fn new() -> TokenBuilder {
        TokenBuilder { pos: 0 }
    }

    pub fn consume(&mut self, chars: impl HasLen) -> Span {
        let pre = self.pos;
        self.pos += chars.utf8_len();
        Span::from(pre..self.pos)
    }

    pub fn consume_token(&mut self, token: &Spanned<Token>) {
        self.pos = token.span().end().into()
    }

    pub fn eof(&mut self) -> Spanned<Token> {
        Token::Leaf(Leaf::EOF).spanned(Span::EOF(self.pos.into()))
    }

    pub fn ws(&mut self, chars: impl HasLen) -> Spanned<Token> {
        Token::Leaf(Leaf::Whitespace).spanned(self.consume(chars))
    }

    pub fn newline(&mut self) -> Spanned<Token> {
        Token::Leaf(Leaf::Newline).spanned(self.consume("\n"))
    }

    pub fn word(&mut self, chars: impl HasLen) -> Spanned<Token> {
        Token::Leaf(Leaf::Word).spanned(self.consume(chars))
    }

    pub fn quote(&mut self, quote: impl Into<Quote>, chars: impl HasLen) -> Spanned<Token> {
        let quote = quote.into();
        let outer_start = self.consume(quote.char());
        let inner = self.consume(chars);
        let outer_end = self.consume(quote.char());

        Token::Leaf(Leaf::Quoted(QuoteResult { quote, inner }))
            .spanned(outer_start.until(outer_end))
        // Top::Quoted((quote, inner)).spanned(start.until(end))
    }

    pub fn delimited(
        &mut self,
        delimiter: Delimiter,
        contents: impl FnOnce(&mut Self) -> Vec<Spanned<Token>>,
    ) -> Spanned<Token> {
        let open = self.consume(delimiter.open_char());

        let tokens = contents(self);

        for token in tokens.iter() {
            self.consume_token(token);
        }

        let close = self.consume(delimiter.close_char());

        Token::delimited(delimiter, tokens).spanned(open.until(close))
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
