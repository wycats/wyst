#[macro_export]
macro_rules! quote_impl {
    ($lifetime:tt, $name:ident, $enum:tt) => {
        fn $name(
            start: Offset,
            quote: Quote,
            lexer: logos::Lexer<$lifetime, $crate::FlatToken>,
        ) -> (
            $crate::wyst_source::Spanned<$crate::QuoteResult>,
            logos::Lexer<$lifetime, $crate::FlatToken>,
        ) {
            let mut quoted: logos::Lexer<$lifetime, $enum> = lexer.morph();

            let (outer, inner) = match quoted.next() {
                Some($enum::Error) => {
                    panic!("error is impossible")
                }
                Some($enum::Contents) => {
                    let child_span = quoted.span();
                    let inner_start = $crate::wyst_source::Offset::from(child_span.start);
                    let inner_end =
                        $crate::wyst_source::Offset::from(child_span.end) - quote.char();
                    (
                        $crate::wyst_source::Span::new(start, child_span.end),
                        $crate::wyst_source::Span::new(inner_start, inner_end),
                    )
                }
                None => todo!(),
            };

            (
                $crate::QuoteResult { quote, inner }.spanned(outer),
                quoted.morph(),
            )
        }
    };
}
