use wyst_core::WystData;
use wyst_source::{Source, Span, Spanned};

use crate::{
    delegate::QuoteResult,
    standard::{Delimiter, FlatToken},
    tree::Token,
};

pub enum ReaderNext<T: WystData> {
    Buffer,
    Token(T),
    EOF,
}

/// Reads a sequence of tokens into a structure
pub trait Reader {
    fn next(&mut self) -> ReaderNext<Spanned<Token>>;
    fn word(&mut self, source: &Source, span: Span);
    fn whitespace(&mut self, source: &Source, span: Span);
    fn newline(&mut self, source: &Source, span: Span);
    fn comment(&mut self, source: &Source, span: Span, body: Span);
    fn quoted(&mut self, source: &Source, span: Span, quote: QuoteResult);
    fn open(&mut self, source: &Source, span: Span, delimiter: Delimiter);
    fn close(&mut self, source: &Source, span: Span, delimiter: Delimiter);
    fn eof(&mut self, source: &Source, span: Span);
}

pub struct ReadTokens<'source, T: Reader, I: Iterator<Item = Spanned<FlatToken>>> {
    reader: T,
    source: &'source Source,
    input: I,
}

impl<'source, T, I> ReadTokens<'source, T, I>
where
    T: Reader,
    I: Iterator<Item = Spanned<FlatToken>>,
{
    pub fn new(reader: T, tokens: I, source: &'source Source) -> ReadTokens<'source, T, I> {
        Self {
            reader,
            input: tokens,
            source,
        }
    }

    fn process_token(&mut self, token: Spanned<FlatToken>) {
        let span = token.span();
        let item = token.item();

        let reader = &mut self.reader;
        let source = &self.source;

        match item {
            FlatToken::Error => todo!("error token (can we make this impossible?)"),
            FlatToken::Word => reader.word(source, span),
            FlatToken::Whitespace => reader.whitespace(source, span),
            FlatToken::Newline => reader.newline(source, span),
            FlatToken::Comment(body) => reader.comment(source, span, *body),
            FlatToken::Quoted((quote, inner)) => {
                reader.quoted(source, span, QuoteResult::new(*quote, *inner))
            }
            FlatToken::Open(d) => reader.open(source, span, *d),
            FlatToken::Close(d) => reader.close(source, span, *d),
            FlatToken::EOF => reader.eof(source, span),
        }
    }
}

impl<'source, T, I> Iterator for ReadTokens<'source, T, I>
where
    T: Reader,
    I: Iterator<Item = Spanned<FlatToken>>,
{
    type Item = Spanned<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.next() {
                ReaderNext::Buffer => {}
                ReaderNext::Token(token) => return Some(token),
                ReaderNext::EOF => return None,
            }

            let next = self.input.next()?;

            self.process_token(next);
        }
    }
}
