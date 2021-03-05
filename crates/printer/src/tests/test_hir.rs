use std::{error::Error, io::BufWriter};

use pretty_assertions::assert_eq;

use wyst_style::{PortableStyle, PrintCrossterm};

use crate::{
    algorithm::layout,
    ir::{Atomic, HirBuilder, LirBuilder},
    texts::Texts,
    PrintConfig, Printer,
};

struct TestMirPass {
    texts: Texts,
    hir: Atomic<PortableStyle>,
}

impl TestMirPass {
    pub fn case(
        &mut self,
        config: impl Into<PrintConfig>,
        expected_lir: impl FnOnce(LirBuilder<PortableStyle>) -> LirBuilder<PortableStyle>,
        expected_output: impl AsRef<str>,
    ) -> Result<(), Box<dyn Error>> {
        let config: PrintConfig = config.into();
        let actual_lir = layout(self.hir.clone(), config);
        let expected_lir = LirBuilder::build(&mut self.texts, |b| expected_lir(b).br(0));

        let buf = {
            let out = vec![];
            let mut buf = BufWriter::new(out);
            let mut printer = PrintCrossterm::new(&mut buf);

            Printer::new(config, self.texts.resolver())
                .print_lir(actual_lir.clone(), &mut printer)?;
            buf.into_inner().unwrap()
        };

        let actual_output = String::from_utf8_lossy(&buf);

        assert_eq!(
            actual_output.as_ref(),
            format!("{}\n", expected_output.as_ref()),
            "OUT: {}",
            config
        );
        assert_eq!(actual_lir, expected_lir, "LIR: {}", config);

        Ok(())
    }
}

fn test_passes(
    hir: impl FnOnce(HirBuilder<PortableStyle>) -> HirBuilder<PortableStyle>,
) -> TestMirPass {
    let mut texts = Texts::default();
    let hir = hir(HirBuilder::new(&mut texts)).done();

    TestMirPass { texts, hir }
}

macro_rules! ops {
    (accum = { $($accum:tt)* } rest = { group { $($inner:tt)* } $($rest:tt)* }) => {
        ops! {
            accum = { $($accum)* . group(ops![$($inner)*]) } rest = { $($rest)* }
        }
    };

    (accum = { $($accum:tt)* } rest = { nest($level:tt) { $($inner:tt)* } $($rest:tt)* }) => {
        ops! {
            accum = { $($accum)* . nest($level, ops![$($inner)*]) } rest = { $($rest)* }
        }
    };

    (accum = { $($accum:tt)* } rest = { $id:ident ( $($inner:tt)* ) $($rest:tt)* }) => {
        ops! {
            accum = { $($accum)* . $id( $($inner)* ) } rest = { $($rest)* }
        }
    };

    (accum = { $($accum:tt)* } rest = { BR $($rest:tt)* }) => {
        ops! {
            accum = { $($accum)* . br() } rest = { $($rest)* }
        }
    };

    (accum = { $($accum:tt)* } rest = { SP $($rest:tt)* }) => {
        ops! {
            accum = { $($accum)* . space(" ") } rest = { $($rest)* }
        }
    };

    (accum = { $($accum:tt)* } rest = { $literal:tt $($rest:tt)* }) => {
        ops! {
            accum = { $($accum)* . text( $literal ) } rest = { $($rest)* }
        }
    };

    (accum = { $($accum:tt)* } rest = {}) => {
        |h| { h $($accum)* }
    };

    ($($token:tt)*) => {
        ops! { accum = {} rest = { $($token)* } }
    };

}

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn test_simple_ir() -> TestResult {
    let mut test = test_passes(ops!["hello"]);
    test.case(80, ops!["hello"], "hello")?;
    test.case(3, ops!["hello"], "hello")?;

    Ok(())
}

#[test]
fn test_bounded_twice() -> TestResult {
    let mut test = test_passes(ops!["hello" "goodbye"]);

    test.case(80, ops!["hello" "goodbye"], "hellogoodbye")?;
    test.case(3, ops!["hello" "goodbye"], "hellogoodbye")?;

    Ok(())
}

#[test]
fn test_allowed_break() -> TestResult {
    let mut test = test_passes(ops!["hello" wbr(0) "goodbye"]);

    test.case(80, ops!["hello" "goodbye"], "hellogoodbye")?;
    test.case(7, ops!["hello" br(0) "goodbye"], "hello\ngoodbye")?;

    Ok(())
}

#[test]
fn test_different_layer_break() -> TestResult {
    let mut test =
        test_passes(ops![ "hello" "(" wbr(0) "this" wbr(1) "is" wbr(1) "inside" wbr(0) ")" ]);

    test.case(
        80,
        ops![ "hello" "(" "this" "is" "inside" ")" ],
        "hello(thisisinside)",
    )?;

    test.case(
        12,
        ops![
            "hello" "(" br(0)
            "this" "is" "inside" br(0)
            ")"
        ],
        "hello(\nthisisinside\n)",
    )?;

    test.case(
        7,
        ops![
            "hello" "(" br(0)
            "this" br(0)
            "is" br(0)
            "inside" br(0)
            ")"
        ],
        "hello(\nthis\nis\ninside\n)",
    )?;

    Ok(())
}

#[test]
fn test_interior_content() -> TestResult {
    let mut test = test_passes(ops![
        "hello" "(" wbr(1)
        "this" wbr(2) SP
        "is" wbr(2) SP
        "inside" wbr(1)
        ")"
    ]);

    test.case(
        80,
        ops!["hello" "(" "this" " " "is" " " "inside" ")"],
        "hello(this is inside)",
    )?;

    test.case(
        14,
        ops![
            "hello" "(" br(0)
            "this" " " "is" " " "inside" br(0)
            ")"
        ],
        "hello(\nthis is inside\n)",
    )?;

    test.case(
        7,
        ops![
            "hello" "(" br(0)
            "this" br(0)
            "is" br(0)
            "inside" br(0)
            ")"
        ],
        "hello(\nthis\nis\ninside\n)",
    )?;

    Ok(())
}

#[test]
fn test_indent() -> TestResult {
    let mut test = test_passes(ops![
        "hello" "("
            nest(0) {
                group {
                    "this" wbr(0) SP
                    "is" wbr(0) SP
                    "inside"
                }
            } wbr(0)
        ")"
    ]);

    test.case(
        80,
        ops!["hello" "(" "this" " " "is" " " "inside" ")"],
        "hello(this is inside)",
    )?;

    test.case(
        16,
        ops![
            "hello" "(" br(1)
                "this" " " "is" " " "inside" br(0)
            ")"
        ],
        "hello(\n  this is inside\n)",
    )?;

    test.case(
        14,
        ops![
            "hello" "(" br(1)
                "this" br(1)
                "is" br(1)
                "inside" br(0)
            ")"
        ],
        "hello(\n  this\n  is\n  inside\n)",
    )?;

    Ok(())
}

#[test]
fn test_atomic_subcontent() -> TestResult {
    let mut test = test_passes(ops![
        group {
            group { "hello" wbr(1) space(" ") "world" }
            wbr(0) space(" ")
            group { "hellooooo" wbr(1) space(" ") "world" }
        }
    ]);

    test.case(
        80,
        ops!["hello" " " "world" " " "hellooooo" " " "world"],
        "hello world hellooooo world",
    )?;

    // "hello world".len() == 11
    // "hellooooo world".len() == 15

    // each group can fit on a 15-character line
    test.case(
        15,
        ops!["hello" " " "world" br(0) "hellooooo" " " "world"],
        "hello world\nhellooooo world",
    )?;

    // "hello world" can fit on its own line without a break
    // "hellooooo world" cannot, and needs to break
    test.case(
        14,
        ops!["hello" " " "world" br(0) "hellooooo" br(0) "world"],
        "hello world\nhellooooo\nworld",
    )?;

    // neither of the sub-strings can fit on its own line without a break
    test.case(
        10,
        ops!["hello" br(0) "world" br(0) "hellooooo" br(0) "world"],
        "hello\nworld\nhellooooo\nworld",
    )?;

    Ok(())
}

#[test]
fn test_atomic_subcontent_with_indentation() -> TestResult {
    let mut test = test_passes(ops![
        "(" nest(0) { group {
                group { "hello" wbr(1) SP "world" "," }
                wbr(0) SP
                group { "hellooooo" wbr(1) SP "world" }
         } } wbr(0)
        ")"
    ]);

    test.case(
        80,
        ops!["(" "hello" " " "world" "," " " "hellooooo" " " "world" ")"],
        "(hello world, hellooooo world)",
    )?;

    test.case(
        17,
        ops![
            "(" br(1)
                "hello" " " "world" "," br(1)
                "hellooooo" " " "world" br(0)
            ")"
        ],
        "(\n  hello world,\n  hellooooo world\n)",
    )?;

    test.case(
        16,
        ops![
            "(" br(1)
                "hello" " " "world" "," br(1)
                "hellooooo" br(1) "world" br(0)
            ")"
        ],
        "(\n  hello world,\n  hellooooo\n  world\n)",
    )?;

    test.case(
        13,
        ops![
            "(" br(1)
                "hello" br(1)
                "world" "," br(1)
                "hellooooo" br(1)
                "world" br(0)
            ")"
        ],
        "(\n  hello\n  world,\n  hellooooo\n  world\n)",
    )?;

    Ok(())
}

#[test]
fn test_prettier_playground_1() -> Result<(), Box<dyn Error>> {
    let mut test = test_passes(ops![
        "function" SP "HelloWorld"
        "(" "{"
        nest(1) {
            SP "greeting" SP "=" SP "\"hello\"" "," wbr(1) SP
            "greeted" SP "=" SP "'\"World\"'" "," wbr(1) SP
            "silent" SP "=" SP "false" "," wbr(1) SP
            "onMouseOver"
        }
        wbr(1) SP
        "}" ")"
        SP "{" "}"
    ]);

    test.case(
        100,
        ops![
            "function" SP "HelloWorld" "(" "{"
            SP "greeting" SP "=" SP "\"hello\"" "," SP
            "greeted" SP "=" SP "'\"World\"'" "," SP
            "silent" SP "=" SP "false" "," SP
            "onMouseOver" SP
            "}" ")" SP "{" "}"
        ],
        r#"function HelloWorld({ greeting = "hello", greeted = '"World"', silent = false, onMouseOver }) {}"#,
    )?;

    test.case(
        95,
        ops![
            "function" SP "HelloWorld" "(" "{" br(1)
                "greeting" SP "=" SP "\"hello\"" "," br(1)
                "greeted" SP "=" SP "'\"World\"'" "," br(1)
                "silent" SP "=" SP "false" "," br(1)
                "onMouseOver" br(0)
            "}" ")" SP "{" "}"
        ],
        "function HelloWorld({\n  greeting = \"hello\",\n  greeted = '\"World\"',\n  silent = false,\n  onMouseOver\n}) {}",
    )?;

    Ok(())
}

#[test]
fn test_glimmer_example() -> Result<(), Box<dyn Error>> {
    //   <div     class="entry"    >
    //   <h1>{{  title    }}</h1>
    //   <div   class="body">
    //             {{   body         }}
    // </div> </div>

    let mut test = test_passes(ops![
        "<" "div" nest(1) { SP "class" "=" "\"" "entry" "\"" wbr(1) } ">" br() indent()
            group { "<h1>" br() indent() "{{" wbr(2) "title" wbr(2) "}}" br() outdent() "</h1>" }
        outdent() br() "</div>"
    ]);

    test.case(
        20,
        ops!["<" "div" SP "class" "=" "\"" "entry" "\"" ">" br(1) "<h1>" br(2) "{{" "title" "}}" br(1) "</h1>" br(0) "</div>" ],
        "<div class=\"entry\">\n  <h1>\n    {{title}}\n  </h1>\n</div>",
    )?;

    test.case(
        18,
        ops!["<" "div" br(1) "class" "=" "\"" "entry" "\"" br(0) ">" br(1) "<h1>" br(2) "{{" "title" "}}" br(1) "</h1>" br(0) "</div>" ],
        "<div\n  class=\"entry\"\n>\n  <h1>\n    {{title}}\n  </h1>\n</div>",
    )?;

    Ok(())
}

#[test]
fn test_prettier_playground() {
    // PLAYGROUND INPUT

    // function HelloWorld({greeting = "hello", greeted = '"World"', silent = false, onMouseOver,}) {
    //
    //     if(!greeting){return null};
    //
    //        // TODO: Don't use random in render
    //     let num = Math.floor (Math.random() * 1E+7).toString().replace(/\.\d+/ig, "")
    //
    //     return <div className='HelloWorld' title={`You are visitor number ${ num }`} onMouseOver={onMouseOver}>
    //
    //       <strong>{ greeting.slice( 0, 1 ).toUpperCase() + greeting.slice(1).toLowerCase() }</strong>
    //       {greeting.endsWith(",") ? " " : <span style={{color: '\grey'}}>", "</span> }
    //       <em>
    //       { greeted }
    //       </em>
    //       { (silent)
    //         ? "."
    //         : "!"}
    //
    //       </div>;
    //
    //   }

    // PLAYGROUND DOC OUTPUT

    // [
    //     "function ",
    //     "HelloWorld",
    //     group([
    //         "(",
    //         "{",
    //         indent([
    //         line,
    //         group(["greeting", " = ", '"hello"']),
    //         ",",
    //         line,
    //         group(["greeted", " = ", "'\"World\"'"]),
    //         ",",
    //         line,
    //         group(["silent", " = ", "false"]),
    //         ",",
    //         line,
    //         group(["onMouseOver"]),
    //         ]),
    //         ifBreak(","),
    //         line,
    //         "}",
    //         ")",
    //     ]),
    //     " ",
    //     "{",
    //     indent([
    //         hardline,
    //         breakParent,
    //         wrappedGroup([
    //         "if (",
    //         group([indent([softline, "!", "greeting"]), softline]),
    //         ")",
    //         " ",
    //         "{",
    //         indent([hardline, breakParent, "return", " ", "null", ";"]),
    //         hardline,
    //         breakParent,
    //         "}",
    //         ]),
    //         hardline,
    //         breakParent,
    //         hardline,
    //         breakParent,
    //         "// TODO: Don't use random in render",
    //         hardline,
    //         breakParent,
    //         wrappedGroup([
    //         "let",
    //         " ",
    //         wrappedGroup([
    //             "num",
    //             " =",
    //             " ",
    //             wrappedGroup([
    //             "Math",
    //             ".",
    //             "floor",
    //             group([
    //                 "(",
    //                 indent([
    //                 softline,
    //                 group([
    //                     group([group(["Math", ".", "random", "(", ")"])]),
    //                     indent([" ", group(["*", line, "1e7"])]),
    //                 ]),
    //                 ]),
    //                 ifBreak(""),
    //                 softline,
    //                 ")",
    //             ]),
    //             indent(
    //                 wrappedGroup([
    //                 hardline,
    //                 breakParent,
    //                 ".",
    //                 "toString",
    //                 "(",
    //                 ")",
    //                 hardline,
    //                 breakParent,
    //                 ".",
    //                 "replace",
    //                 group([
    //                     "(",
    //                     indent([softline, "/\\.\\d+/gi", ",", line, '""']),
    //                     ifBreak(""),
    //                     softline,
    //                     ")",
    //                 ]),
    //                 ])
    //             ),
    //             ]),
    //         ]),
    //         indent([]),
    //         ";",
    //         ]),
    //         hardline,
    //         breakParent,
    //         hardline,
    //         breakParent,
    //         "return",
    //         " ",
    //         wrappedGroup([
    //         ifBreak("("),
    //         indent([
    //             softline,
    //             wrappedGroup([
    //             group([
    //                 "<",
    //                 "div",
    //                 indent([
    //                 line,
    //                 "className",
    //                 "=",
    //                 '"',
    //                 "HelloWorld",
    //                 '"',
    //                 line,
    //                 "title",
    //                 "=",
    //                 group([
    //                     "{",
    //                     lineSuffixBoundary,
    //                     "`",
    //                     "You are visitor number ",
    //                     group(["${", "num", lineSuffixBoundary, "}"]),
    //                     "`",
    //                     lineSuffixBoundary,
    //                     "}",
    //                 ]),
    //                 line,
    //                 "onMouseOver",
    //                 "=",
    //                 group([
    //                     "{",
    //                     indent([softline, "onMouseOver"]),
    //                     softline,
    //                     lineSuffixBoundary,
    //                     "}",
    //                 ]),
    //                 ]),
    //                 softline,
    //                 ">",
    //             ]),
    //             indent([
    //                 hardline,
    //                 breakParent,
    //                 wrappedGroup([
    //                 conditionalGroup([
    //                     group([
    //                     group(["<", "strong", indent([]), ">"]),
    //                     group([
    //                         "{",
    //                         group([
    //                         group([
    //                             conditionalGroup([
    //                             [
    //                                 "greeting",
    //                                 ".",
    //                                 "slice",
    //                                 group([
    //                                 "(",
    //                                 indent([softline, "0", ",", line, "1"]),
    //                                 ifBreak(""),
    //                                 softline,
    //                                 ")",
    //                                 ]),
    //                                 ".",
    //                                 "toUpperCase",
    //                                 "(",
    //                                 ")",
    //                             ],
    //                             [
    //                                 "greeting",
    //                                 indent(
    //                                 wrappedGroup([
    //                                     hardline,
    //                                     breakParent,
    //                                     ".",
    //                                     "slice",
    //                                     group([
    //                                     "(",
    //                                     indent([softline, "0", ",", line, "1"]),
    //                                     ifBreak(""),
    //                                     softline,
    //                                     ")",
    //                                     ]),
    //                                     hardline,
    //                                     breakParent,
    //                                     ".",
    //                                     "toUpperCase",
    //                                     "(",
    //                                     ")",
    //                                 ])
    //                                 ),
    //                             ],
    //                             ]),
    //                         ]),
    //                         indent([
    //                             " ",
    //                             group([
    //                             "+",
    //                             line,
    //                             conditionalGroup([
    //                                 [
    //                                 "greeting",
    //                                 ".",
    //                                 "slice",
    //                                 group([
    //                                     "(",
    //                                     indent([softline, "1"]),
    //                                     ifBreak(""),
    //                                     softline,
    //                                     ")",
    //                                 ]),
    //                                 ".",
    //                                 "toLowerCase",
    //                                 "(",
    //                                 ")",
    //                                 ],
    //                                 [
    //                                 "greeting",
    //                                 indent(
    //                                     wrappedGroup([
    //                                     hardline,
    //                                     breakParent,
    //                                     ".",
    //                                     "slice",
    //                                     group([
    //                                         "(",
    //                                         indent([softline, "1"]),
    //                                         ifBreak(""),
    //                                         softline,
    //                                         ")",
    //                                     ]),
    //                                     hardline,
    //                                     breakParent,
    //                                     ".",
    //                                     "toLowerCase",
    //                                     "(",
    //                                     ")",
    //                                     ])
    //                                 ),
    //                                 ],
    //                             ]),
    //                             ]),
    //                         ]),
    //                         ]),
    //                         lineSuffixBoundary,
    //                         "}",
    //                     ]),
    //                     "</",
    //                     "strong",
    //                     ">",
    //                     ]),
    //                     wrappedGroup([
    //                     group(["<", "strong", indent([]), ">"]),
    //                     indent([
    //                         hardline,
    //                         breakParent,
    //                         wrappedGroup([
    //                         group([
    //                             "{",
    //                             group([
    //                             group([
    //                                 conditionalGroup([
    //                                 [
    //                                     "greeting",
    //                                     ".",
    //                                     "slice",
    //                                     group([
    //                                     "(",
    //                                     indent([softline, "0", ",", line, "1"]),
    //                                     ifBreak(""),
    //                                     softline,
    //                                     ")",
    //                                     ]),
    //                                     ".",
    //                                     "toUpperCase",
    //                                     "(",
    //                                     ")",
    //                                 ],
    //                                 [
    //                                     "greeting",
    //                                     indent(
    //                                     wrappedGroup([
    //                                         hardline,
    //                                         breakParent,
    //                                         ".",
    //                                         "slice",
    //                                         group([
    //                                         "(",
    //                                         indent([softline, "0", ",", line, "1"]),
    //                                         ifBreak(""),
    //                                         softline,
    //                                         ")",
    //                                         ]),
    //                                         hardline,
    //                                         breakParent,
    //                                         ".",
    //                                         "toUpperCase",
    //                                         "(",
    //                                         ")",
    //                                     ])
    //                                     ),
    //                                 ],
    //                                 ]),
    //                             ]),
    //                             indent([
    //                                 " ",
    //                                 group([
    //                                 "+",
    //                                 line,
    //                                 conditionalGroup([
    //                                     [
    //                                     "greeting",
    //                                     ".",
    //                                     "slice",
    //                                     group([
    //                                         "(",
    //                                         indent([softline, "1"]),
    //                                         ifBreak(""),
    //                                         softline,
    //                                         ")",
    //                                     ]),
    //                                     ".",
    //                                     "toLowerCase",
    //                                     "(",
    //                                     ")",
    //                                     ],
    //                                     [
    //                                     "greeting",
    //                                     indent(
    //                                         wrappedGroup([
    //                                         hardline,
    //                                         breakParent,
    //                                         ".",
    //                                         "slice",
    //                                         group([
    //                                             "(",
    //                                             indent([softline, "1"]),
    //                                             ifBreak(""),
    //                                             softline,
    //                                             ")",
    //                                         ]),
    //                                         hardline,
    //                                         breakParent,
    //                                         ".",
    //                                         "toLowerCase",
    //                                         "(",
    //                                         ")",
    //                                         ])
    //                                     ),
    //                                     ],
    //                                 ]),
    //                                 ]),
    //                             ]),
    //                             ]),
    //                             lineSuffixBoundary,
    //                             "}",
    //                         ]),
    //                         ]),
    //                     ]),
    //                     hardline,
    //                     breakParent,
    //                     "</",
    //                     "strong",
    //                     ">",
    //                     ]),
    //                 ]),
    //                 hardline,
    //                 breakParent,
    //                 group([
    //                     "{",
    //                     group([
    //                     group([
    //                         "greeting",
    //                         ".",
    //                         "endsWith",
    //                         group([
    //                         "(",
    //                         indent([softline, '","']),
    //                         ifBreak(""),
    //                         softline,
    //                         ")",
    //                         ]),
    //                     ]),
    //                     " ? ",
    //                     ifBreak("("),
    //                     indent([softline, '" "']),
    //                     softline,
    //                     ifBreak(")"),
    //                     " : ",
    //                     ifBreak("("),
    //                     indent([
    //                         softline,
    //                         conditionalGroup([
    //                         group([
    //                             group([
    //                             "<",
    //                             "span",
    //                             indent([
    //                                 line,
    //                                 "style",
    //                                 "=",
    //                                 group([
    //                                 "{",
    //                                 group([
    //                                     "{",
    //                                     indent([
    //                                     line,
    //                                     group([
    //                                         group([
    //                                         "color",
    //                                         ":",
    //                                         group(indent([line, '"grey"'])),
    //                                         ]),
    //                                     ]),
    //                                     ]),
    //                                     ifBreak(","),
    //                                     line,
    //                                     "}",
    //                                 ]),
    //                                 lineSuffixBoundary,
    //                                 "}",
    //                                 ]),
    //                             ]),
    //                             softline,
    //                             ">",
    //                             ]),
    //                             '",',
    //                             line,
    //                             '"',
    //                             "</",
    //                             "span",
    //                             ">",
    //                         ]),
    //                         wrappedGroup([
    //                             group([
    //                             "<",
    //                             "span",
    //                             indent([
    //                                 line,
    //                                 "style",
    //                                 "=",
    //                                 group([
    //                                 "{",
    //                                 group([
    //                                     "{",
    //                                     indent([
    //                                     line,
    //                                     group([
    //                                         group([
    //                                         "color",
    //                                         ":",
    //                                         group(indent([line, '"grey"'])),
    //                                         ]),
    //                                     ]),
    //                                     ]),
    //                                     ifBreak(","),
    //                                     line,
    //                                     "}",
    //                                 ]),
    //                                 lineSuffixBoundary,
    //                                 "}",
    //                                 ]),
    //                             ]),
    //                             softline,
    //                             ">",
    //                             ]),
    //                             indent([hardline, breakParent, fill('",', line, '"')]),
    //                             hardline,
    //                             breakParent,
    //                             "</",
    //                             "span",
    //                             ">",
    //                         ]),
    //                         ]),
    //                     ]),
    //                     softline,
    //                     ifBreak(")"),
    //                     ]),
    //                     lineSuffixBoundary,
    //                     "}",
    //                 ]),
    //                 hardline,
    //                 breakParent,
    //                 conditionalGroup([
    //                     group([
    //                     group(["<", "em", indent([]), ">"]),
    //                     group([
    //                         "{",
    //                         indent([softline, "greeted"]),
    //                         softline,
    //                         lineSuffixBoundary,
    //                         "}",
    //                     ]),
    //                     "</",
    //                     "em",
    //                     ">",
    //                     ]),
    //                     wrappedGroup([
    //                     group(["<", "em", indent([]), ">"]),
    //                     indent([
    //                         hardline,
    //                         breakParent,
    //                         wrappedGroup([
    //                         group([
    //                             "{",
    //                             indent([softline, "greeted"]),
    //                             softline,
    //                             lineSuffixBoundary,
    //                             "}",
    //                         ]),
    //                         ]),
    //                     ]),
    //                     hardline,
    //                     breakParent,
    //                     "</",
    //                     "em",
    //                     ">",
    //                     ]),
    //                 ]),
    //                 hardline,
    //                 breakParent,
    //                 group([
    //                     "{",
    //                     group([
    //                     "silent",
    //                     indent([
    //                         line,
    //                         "? ",
    //                         align(2, ['"."']),
    //                         line,
    //                         ": ",
    //                         align(2, ['"!"']),
    //                     ]),
    //                     ]),
    //                     lineSuffixBoundary,
    //                     "}",
    //                 ]),
    //                 ]),
    //             ]),
    //             hardline,
    //             breakParent,
    //             "</",
    //             "div",
    //             ">",
    //             ]),
    //         ]),
    //         softline,
    //         ifBreak(")"),
    //         ]),
    //         ";",
    //     ]),
    //     hardline,
    //     breakParent,
    //     "}",
    //     hardline,
    //     breakParent,
    // ];
}
