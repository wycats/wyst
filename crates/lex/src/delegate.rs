use logos::{Lexer, Logos};
use wyst_core::{new, wyst_copy};
use wyst_core_traits::WystCopy;
use wyst_source::{AddSpan, Offset, Span, Spanned};

use crate::standard::{FlatToken, Quote};

#[wyst_copy]
#[derive(new)]
pub struct QuoteResult {
    pub(crate) quote: Quote,
    pub(crate) inner: Span,
}

pub trait StandardDelegate<'source>: WystCopy {
    fn quote(
        start: Offset,
        quote: Quote,
        lexer: Lexer<'source, FlatToken>,
    ) -> (Spanned<QuoteResult>, Lexer<'source, FlatToken>) {
        match quote {
            Quote::DoubleQuote => Self::double_quote(start, quote, lexer),
            Quote::SingleQuote => Self::single_quote(start, quote, lexer),
            Quote::Backtick => Self::backtick(start, quote, lexer),
        }
    }

    fn comment(
        lexer: Lexer<'source, FlatToken>,
    ) -> (Option<Spanned<FlatToken>>, Lexer<'source, FlatToken>) {
        (None, lexer)
    }

    quote_impl!('source, double_quote, DefaultStringWord);
    quote_impl!('source, single_quote, DefaultStringWord);
    quote_impl!('source, backtick, DefaultStringWord);
}

#[wyst_copy]
#[derive(Logos)]
pub enum DefaultStringWord {
    #[error]
    Error,

    #[regex(r"([^\(\[\{\)\]\}\p{White_Space}][^\p{White_Space}\(\[\{]*)?")]
    Contents,
}

#[wyst_copy]
pub struct DefaultDelegate;

impl StandardDelegate<'_> for DefaultDelegate {}
