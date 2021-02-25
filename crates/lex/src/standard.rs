use std::marker::PhantomData;

use logos::{Lexer, Logos};
use wyst_core::{new, unit_tests, wyst_copy};
use wyst_source::{Source, Span, Spanned};

use crate::{
    delegate::StandardDelegate,
    reader::ReadTokens,
    tree::{Token, TokenTree},
};

/// `Quoted` is the representation of a `FlatToken` that begins with a quote (either double-quote,
/// single-quote or backtick). The token itself has the outer span (which surrounds the quoted),
/// while the `Quoted` contains the inner span (which represents the contents inside the quotes).
#[wyst_copy]
#[derive(new)]
pub struct Quoted {
    quote: Quote,
    inner: Span,
}

/// `FlatToken` represents the most basic tokenization of a source string.
///
/// It breaks up the source into:
///
/// - Whitespace: non-newline whitespace
/// - Newline: `\n`
/// - Quoted: a chunk of text surrounded by a quote (double-quote, single-quote or backtick)
/// - Open: an opening delimiter (`[`, `{` or `(`)
/// - Close: a closing delimiter (`]`, `}` or `)`)
/// - Comment
///
/// Quotation rules are pluggable by passing an implementation of `StandardDelegate` to
/// `read_source` or `lex_source`.
///
/// The purpose of the `FlatToken` step is to tokenize atomic tokens from the source. Unlike a
/// traditional lexer, it does not attempt to determine keywords. It generally defers the
/// determination of the **meaning** of a particular atomic token to a subequent phase.
///
/// The tokens produced by the lexing process are read by the reader into a token tree.
#[wyst_copy]
#[derive(Logos)]
pub enum FlatToken {
    #[error]
    Error,

    #[regex(r#"[^"'`\(\[\{\)\]\}\p{White_Space}][^"'`\p{White_Space}\(\[\{]*"#)]
    Word,

    #[regex(r"[^\S\n]+")]
    Whitespace,

    #[token("\n")]
    Newline,

    #[token("\"", |lexer| ((Quote::DoubleQuote, Span::from(lexer.span()))))]
    #[token("'", |lexer| ((Quote::SingleQuote, Span::from(lexer.span()))))]
    #[token("`", |lexer| ((Quote::Backtick, Span::from(lexer.span()))))]
    Quoted((Quote, Span)),

    #[token("(", |_| Delimiter::Paren)]
    #[token("[", |_| Delimiter::Bracket)]
    #[token("{", |_| Delimiter::Brace)]
    Open(Delimiter),

    #[token(")", |_| Delimiter::Paren)]
    #[token("]", |_| Delimiter::Bracket)]
    #[token("}", |_| Delimiter::Brace)]
    Close(Delimiter),

    Comment(Span),

    EOF,
}

impl FlatToken {
    pub fn lex_source<'source, S>(source: &'source str) -> LexTop<'source, S>
    where
        S: StandardDelegate<'source>,
    {
        LexTop {
            lexer: Some(FlatToken::lexer(source)),
            done: false,
            source: PhantomData,
        }
    }

    pub fn read_source<'source, S>(
        source: &'source Source,
    ) -> impl Iterator<Item = Spanned<Token>> + 'source
    where
        S: StandardDelegate<'source> + 'source,
    {
        let lexed = FlatToken::lex_source::<S>(source.contents());
        lexed.read(source)
    }
}

pub struct LexTop<'source, S>
where
    S: StandardDelegate<'source>,
{
    lexer: Option<Lexer<'source, FlatToken>>,
    done: bool,
    source: PhantomData<S>,
}

impl<'source, S> LexTop<'source, S>
where
    S: StandardDelegate<'source> + 'source,
{
    pub fn read(self, source: &'source Source) -> impl Iterator<Item = Spanned<Token>> + 'source {
        ReadTokens::new(TokenTree::new(), self, source)
    }
}

impl<'source, S> Iterator for LexTop<'source, S>
where
    S: StandardDelegate<'source>,
{
    type Item = Spanned<FlatToken>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let (comment, lexer) =
            S::comment(self.lexer.take().expect("iterating top is not reentrant"));

        self.lexer = Some(lexer);

        if let Some(comment) = comment {
            return Some(comment);
        }

        let token = self.lexer.as_mut().map(|l| l.next()).unwrap();
        let span = self.lexer.as_ref().map(|l| l.span()).unwrap();

        match token {
            Some(FlatToken::Quoted((quote, span))) => {
                let lexer = self.lexer.take().expect("iterating top is not reentrant");

                let (spanned_quote, lexer) = S::quote(span.start(), quote, lexer);
                let token = Some(
                    spanned_quote
                        .span()
                        .map(|| FlatToken::Quoted((quote, spanned_quote.item().inner))),
                );
                self.lexer = Some(lexer);
                token
            }
            Some(token) => Some(Spanned::new(span, token)),
            None => {
                self.done = true;
                Some(Spanned::new(Span::EOF(span.start.into()), FlatToken::EOF))
            }
        }
    }
}

#[wyst_copy]
pub enum Quote {
    // "
    DoubleQuote,
    // '
    SingleQuote,
    // `
    Backtick,
}

impl From<char> for Quote {
    fn from(char: char) -> Self {
        match char {
            '"' => Quote::DoubleQuote,
            '\'' => Quote::SingleQuote,
            '`' => Quote::Backtick,
            other => panic!("Cannot convert '{}' to Quote (quote must be one of: \" (double quote), ' (single quote), or ` (backtick))", other)
        }
    }
}

impl Quote {
    pub fn char(self) -> char {
        match self {
            Quote::DoubleQuote => '"',
            Quote::SingleQuote => '\'',
            Quote::Backtick => '`',
        }
    }
}

#[wyst_copy]
pub enum Delimiter {
    // "(" ")"
    Paren,
    // "{" "}"
    Brace,
    // "[" "]"
    Bracket,
}

impl Delimiter {
    pub fn open_char(self) -> char {
        match self {
            Delimiter::Paren => '(',
            Delimiter::Brace => '{',
            Delimiter::Bracket => '[',
        }
    }

    pub fn close_char(self) -> char {
        match self {
            Delimiter::Paren => ')',
            Delimiter::Brace => '}',
            Delimiter::Bracket => ']',
        }
    }
}

unit_tests!(
    all({
        use crate::delegate::{DefaultDelegate, StandardDelegate};
        use crate::top_builder::TopBuilder;
        use wyst_source::{AddSpan, Offset};
    }),
    tests(
        ("whitespace", {
            let tokens: Vec<_> = FlatToken::lex_source::<DefaultDelegate>("   ").collect();
            let mut b = TopBuilder::new();

            assert_eq!(tokens, &[b.ws("   "), b.eof()])
        }),
        ("newlines", {
            let tokens: Vec<_> = FlatToken::lex_source::<DefaultDelegate>("   \n   ").collect();
            let mut b = TopBuilder::new();

            assert_eq!(tokens, &[b.ws("   "), b.newline(), b.ws("   "), b.eof()])
        }),
        ("delimiters", {
            let tokens: Vec<_> =
                FlatToken::lex_source::<DefaultDelegate>("   hello { world() }   ").collect();

            let mut b = TopBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.ws(" "),
                    b.open("{"),
                    b.ws(" "),
                    b.word("world"),
                    b.open("("),
                    b.close(")"),
                    b.ws(" "),
                    b.close("}"),
                    b.ws("   "),
                    b.eof()
                ]
            );
        }),
        ("quotes (by default)", {
            let tokens: Vec<_> =
                FlatToken::lex_source::<DefaultDelegate>(r#"   hello( "world" )   "#).collect();

            let mut b = TopBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.open("("),
                    b.ws(" "),
                    b.quote("world", Quote::DoubleQuote),
                    b.ws(" "),
                    b.close(")"),
                    b.ws("   "),
                    b.eof()
                ]
            );
        }),
        ("double quotes (with custom delegate)", {
            #[wyst_copy]
            #[derive(Logos)]
            enum DoubleQuote {
                #[error]
                Error,

                #[regex(r#"[^"]*""#)]
                Contents,
            }

            impl<'a> StandardDelegate<'a> for DoubleQuote {
                quote_impl!('a, double_quote, DoubleQuote);
            }

            let tokens: Vec<_> =
                FlatToken::lex_source::<DoubleQuote>(r#"   hello( "world" )   "#).collect();

            let mut b = TopBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.open("("),
                    b.ws(" "),
                    b.quote("world", Quote::DoubleQuote),
                    b.ws(" "),
                    b.close(")"),
                    b.ws("   "),
                    b.eof()
                ]
            );
        }),
        ("single quotes (with custom delegate)", {
            #[wyst_copy]
            #[derive(Logos)]
            enum SingleQuote {
                #[error]
                Error,

                #[regex(r#"[^']*'"#)]
                Contents,
            }

            impl<'a> StandardDelegate<'a> for SingleQuote {
                quote_impl!('a, single_quote, SingleQuote);
            }

            let tokens: Vec<_> =
                FlatToken::lex_source::<SingleQuote>(r#"   hello( 'world' )   "#).collect();

            let mut b = TopBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.open("("),
                    b.ws(" "),
                    b.quote("world", Quote::SingleQuote),
                    b.ws(" "),
                    b.close(")"),
                    b.ws("   "),
                    b.eof()
                ]
            );
        }),
        ("backticks (with custom delegate)", {
            #[wyst_copy]
            #[derive(Logos)]
            enum Backtick {
                #[error]
                Error,

                #[regex(r#"[^`]*`"#)]
                Contents,
            }

            impl<'a> StandardDelegate<'a> for Backtick {
                quote_impl!('a, backtick, Backtick);
            }

            let tokens: Vec<_> =
                FlatToken::lex_source::<Backtick>(r#"   hello( `world` )   "#).collect();

            let mut b = TopBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.open("("),
                    b.ws(" "),
                    b.quote("world", Quote::Backtick),
                    b.ws(" "),
                    b.close(")"),
                    b.ws("   "),
                    b.eof()
                ]
            );
        }),
        ("comments", {
            let tokens: Vec<_> = FlatToken::lex_source::<Comment>(
                "   hello( `world` ) # line comment\n  # line comment\n  ",
            )
            .collect();

            let mut b = TopBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.open("("),
                    b.ws(" "),
                    b.quote("world", Quote::Backtick),
                    b.ws(" "),
                    b.close(")"),
                    b.ws(" "),
                    b.comment(" line comment", ("#", "")),
                    b.newline(),
                    b.ws("  "),
                    b.comment(" line comment", ("#", "")),
                    b.newline(),
                    b.ws("  "),
                    b.eof()
                ]
            );

            #[wyst_copy]
            #[derive(Logos)]
            enum Comment {
                #[error]
                Error,

                #[regex(r"#[^\n]*`")]
                Contents,
            }

            impl<'a> StandardDelegate<'a> for Comment {
                fn comment(
                    lexer: logos::Lexer<'a, FlatToken>,
                ) -> (Option<Spanned<FlatToken>>, Lexer<'a, FlatToken>) {
                    let remainder = lexer.remainder();

                    if remainder.chars().nth(0) == Some('#') {
                        let mut comment: Lexer<'a, Comment> = lexer.morph();

                        match comment.next() {
                            None => (None, comment.morph()),
                            Some(_) => {
                                let span = comment.span();
                                (
                                    Some(
                                        FlatToken::Comment(Span::new(
                                            Offset::from(span.start) + '#',
                                            span.end,
                                        ))
                                        .spanned(span),
                                    ),
                                    comment.morph(),
                                )
                            }
                        }
                    } else {
                        (None, lexer)
                    }
                }
            }
        })
    )
);
