use std::collections::VecDeque;

use wyst_core::{unit_tests, wyst_copy, wyst_data};
use wyst_source::{AddSpan, Offset, Span, Spanned};

use crate::reader::{Reader, ReaderNext};
use crate::{delegate::QuoteResult, standard::Delimiter};

#[wyst_copy]
pub enum Leaf {
    #[allow(unused)]
    Error,

    EOF,
    Word,
    Quoted(QuoteResult),
    Comment(Span),
    Whitespace,
    Newline,
}

#[wyst_data]
pub struct Program {
    children: Vec<Spanned<Token>>,
}

#[wyst_data]
pub struct Delimited {
    delimiter: Delimiter,
    children: Vec<Spanned<Token>>,
}

impl Delimited {
    fn push(&mut self, token: Spanned<Token>) {
        self.children.push(token);
    }
}

#[wyst_data]
pub enum Token {
    Leaf(Leaf),
    Delimited(Delimited),
}

impl Token {
    pub fn delimited(
        delimiter: impl Into<Delimiter>,
        tokens: impl IntoIterator<Item = Spanned<Token>>,
    ) -> Token {
        Token::Delimited(Delimited {
            delimiter: delimiter.into(),
            children: tokens.into_iter().collect(),
        })
    }
}

/// "{" "(" hello ")" "}"
///
/// push "{"
/// push "("
/// push "hello"
pub struct TokenTree {
    stack: Vec<(Delimited, Offset)>,
    current_parent: Option<(Delimited, Offset)>,
    finished_tokens: VecDeque<Spanned<Token>>,
    done: bool,
}

impl TokenTree {
    pub fn new() -> TokenTree {
        TokenTree {
            stack: vec![],
            current_parent: None,
            finished_tokens: VecDeque::new(),
            done: false,
        }
    }

    fn push_leaf(&mut self, leaf: Leaf, span: Span) {
        let token = Token::Leaf(leaf).spanned(span);

        match &mut self.current_parent {
            Some((parent, _)) => parent.push(token),
            None => self.finished_tokens.push_back(token),
        }
    }
}

impl Reader for TokenTree {
    fn next(&mut self) -> ReaderNext<Spanned<Token>> {
        match self.finished_tokens.pop_front() {
            Some(token) => ReaderNext::Token(token),
            None => {
                if self.done {
                    ReaderNext::EOF
                } else {
                    ReaderNext::Buffer
                }
            }
        }
    }

    fn word(&mut self, _source: &wyst_source::Source, span: Span) {
        self.push_leaf(Leaf::Word, span);
    }

    fn whitespace(&mut self, _source: &wyst_source::Source, span: Span) {
        self.push_leaf(Leaf::Whitespace, span);
    }

    fn newline(&mut self, _source: &wyst_source::Source, span: Span) {
        self.push_leaf(Leaf::Newline, span);
    }

    fn comment(&mut self, _source: &wyst_source::Source, span: Span, body: Span) {
        self.push_leaf(Leaf::Comment(body), span)
    }

    fn quoted(&mut self, _source: &wyst_source::Source, span: Span, quote: QuoteResult) {
        self.push_leaf(Leaf::Quoted(quote), span)
    }

    fn open(&mut self, _source: &wyst_source::Source, span: Span, delimiter: Delimiter) {
        if let Some(parent) = self.current_parent.take() {
            self.stack.push(parent);
        }

        self.current_parent = Some((
            Delimited {
                delimiter,
                children: vec![],
            },
            span.start(),
        ));
    }

    fn close(&mut self, _source: &wyst_source::Source, span: Span, _delimiter: Delimiter) {
        let (parent, start) = self
            .current_parent
            .take()
            .expect("BUG: unbalanced open/close");

        let token = Token::Delimited(parent).spanned(Span::new(start, span.end()));

        match self.stack.pop() {
            Some((mut tail, tail_offset)) => {
                tail.push(token);
                self.current_parent = Some((tail, tail_offset));
            }
            None => self.finished_tokens.push_back(token),
        }
    }

    fn eof(&mut self, _source: &wyst_source::Source, span: Span) {
        self.done = true;

        if let Some(_) = self.current_parent {
            panic!("eof() called when there was still an open delimiter")
        };

        if self.stack.len() > 0 {
            panic!("eof() called when there was still an open delimiter")
        }

        self.finished_tokens
            .push_back(Token::Leaf(Leaf::EOF).spanned(span));
    }
}

unit_tests!(
    all({
        use crate::delegate::DefaultDelegate;
        use crate::standard::FlatToken;
        use crate::token_builder::TokenBuilder;
        use std::sync::Arc;
        use wyst_source::Source;

        fn source(string: &str) -> Arc<Source> {
            Arc::new(Source::new("<test>", string))
        }
    }),
    tests(
        ("whitespace", {
            let s = source("   ");
            let tokens: Vec<_> = FlatToken::read_source::<DefaultDelegate>(&s).collect();
            let mut b = TokenBuilder::new();

            assert_eq!(tokens, &[b.ws("   "), b.eof()])
        }),
        ("newlines", {
            let s = source("   \n   ");
            let tokens: Vec<_> = FlatToken::read_source::<DefaultDelegate>(&s).collect();
            let mut b = TokenBuilder::new();

            assert_eq!(tokens, &[b.ws("   "), b.newline(), b.ws("   "), b.eof()])
        }),
        ("delimiters", {
            let s = source("   hello { world() }   ");
            let tokens: Vec<_> = FlatToken::read_source::<DefaultDelegate>(&s).collect();

            let mut b = TokenBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.ws(" "),
                    b.delimited(Delimiter::Brace, |b| {
                        vec![
                            b.ws(" "),
                            b.word("world"),
                            b.delimited(Delimiter::Paren, |_| vec![]),
                            b.ws(" "),
                        ]
                    }),
                    b.ws("   "),
                    b.eof()
                ]
            );
        }),
        ("quotes (not special)", {
            let s = source(r#"   hello( "world" )   "#);
            let tokens: Vec<_> = FlatToken::read_source::<DefaultDelegate>(&s).collect();

            let mut b = TokenBuilder::new();

            assert_eq!(
                tokens,
                &[
                    b.ws("   "),
                    b.word("hello"),
                    b.delimited(Delimiter::Paren, |b| vec![
                        b.ws(" "),
                        b.quote('"', "world"),
                        b.ws(" "),
                    ]),
                    b.ws("   "),
                    b.eof()
                ]
            );
        })
    )
);
