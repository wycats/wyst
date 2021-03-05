use lasso::Key;
use wyst_core::{new, wyst_copy, wyst_display};
use wyst_style::Style;

use crate::texts::ResolveTexts;

/// FIXME: Include Style in Display
#[wyst_display("Text(id={}, len={})", "self.text.into_usize()", "self.len")]
#[wyst_copy]
#[derive(new)]
pub struct Text<S>
where
    S: Style,
{
    pub(crate) text: lasso::Spur,
    len: usize,
    style: S,
}

impl<S> Text<S>
where
    S: Style,
{
    pub fn len(self) -> usize {
        self.len
    }

    pub fn fits(self, space: usize) -> bool {
        space >= self.len
    }

    pub fn resolve<'resolver>(self, resolver: &'resolver ResolveTexts) -> &'resolver str {
        resolver.resolve(self.text)
    }

    pub fn style(self) -> S {
        self.style
    }
}

pub trait Printable {
    fn fmt_text(&self, texts: ResolveTexts<'_>, fmt: &mut std::fmt::Formatter) -> std::fmt::Result;

    fn format<'printable>(
        &'printable self,
        texts: impl Into<ResolveTexts<'printable>>,
    ) -> PrintableFormatter<'printable, Self>
    where
        Self: Sized,
    {
        PrintableFormatter {
            printable: self,
            texts: texts.into(),
        }
    }
}

impl<S> Printable for Text<S>
where
    S: Style,
{
    fn fmt_text(&self, texts: ResolveTexts<'_>, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let text = texts.resolve(self.text);
        write!(fmt, "{}", text)
    }
}

pub struct PrintableFormatter<'printable, P>
where
    P: Printable,
{
    printable: &'printable P,
    texts: ResolveTexts<'printable>,
}

impl<'texts, P> std::fmt::Display for PrintableFormatter<'texts, P>
where
    P: Printable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.printable.fmt_text(self.texts.clone(), f)
    }
}
