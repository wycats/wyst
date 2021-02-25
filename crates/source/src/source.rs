use wyst_macros::{unit_tests, wyst_data};

use crate::Span;

#[wyst_data]
pub struct Source {
    filename: camino::Utf8PathBuf,
    contents: String,
}

impl Source {
    pub fn new(filename: impl Into<camino::Utf8PathBuf>, contents: impl Into<String>) -> Source {
        Source {
            filename: filename.into(),
            contents: contents.into(),
        }
    }

    pub fn slice(&self, span: Span) -> &str {
        span.slice(&self.contents)
    }
}

unit_tests!(
    all({
        struct Sources {
            main: Source,
            main_eq: Source,
            s2: Source,
            s3: Source,
            s4: Source,
        }

        impl Sources {
            fn test() -> Sources {
                Sources {
                    main: Source::new("test.ts", "hello world"),
                    main_eq: Source::new("test.ts", "hello world"),
                    s2: Source::new("test1.ts", "hello world"),
                    s3: Source::new("test.ts", "hello world!"),
                    s4: Source::new("test1.ts", "hello world!"),
                }
            }
        }
    }),
    tests(
        ("Source#filename", {
            assert_eq!(Sources::test().main.filename, "test.ts");
        }),
        ("Source#contents", {
            assert_eq!(Sources::test().main.contents, String::from("hello world"))
        }),
        ("Source#eq", {
            let sources = Sources::test();

            assert_eq!(sources.main, sources.main_eq);
            assert_ne!(sources.main, sources.s2);
            assert_ne!(sources.main, sources.s3);
            assert_ne!(sources.main, sources.s4);
        }),
        ("Source (as map key)", {
            let sources = Sources::test();

            let mut map = std::collections::HashMap::new();
            map.insert(sources.main.clone(), "s1");

            assert_eq!(map.get(&sources.main), Some(&"s1"));
            assert_eq!(map.get(&sources.main_eq), Some(&"s1"));
            assert_eq!(map.get(&sources.s2), None);
            assert_eq!(map.get(&sources.s3), None);
            assert_eq!(map.get(&sources.s4), None);
        }),
        ("Source#slice", {
            let Sources { main, .. } = Sources::test();
            let span = Span::new(6, 11);

            assert_eq!(main.slice(span), "world");
        })
    )
);
