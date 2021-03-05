use std::error::Error;

use wyst_core::{new, wyst_copy, wyst_data_value, wyst_display};
use wyst_source::HasLen;
use wyst_style::{Indent, Print, Style};

use crate::{
    algorithm::layout,
    ir::{Atomic, HIR, LIR},
    texts::ResolveTexts,
};

#[wyst_display(
    "Display(page_width: {}, indent: {:?})",
    "self.page_width",
    "self.indent"
)]
#[wyst_copy]
#[derive(new)]
pub struct PrintConfig {
    pub(crate) page_width: usize,
    pub(crate) indent: &'static str,
}

impl PrintConfig {
    pub(crate) fn indent_width(self, indent: usize) -> usize {
        self.indent.utf8_len() * indent
    }
}

impl From<usize> for PrintConfig {
    fn from(size: usize) -> Self {
        PrintConfig {
            page_width: size,
            indent: "  ",
        }
    }
}

#[wyst_data_value]
#[derive(new)]
pub struct Printer<'texts> {
    config: PrintConfig,
    resolve: ResolveTexts<'texts>,
}

impl<'texts> Printer<'texts> {
    pub fn print<S>(
        &self,
        hir: Vec<HIR<S>>,
        printer: &mut impl Print<Style = S>,
    ) -> Result<(), Box<dyn Error>>
    where
        S: Style,
    {
        let lir = layout(Atomic::<S>::new(hir), self.config);

        self.print_lir(lir, printer)
    }

    pub fn print_lir<S>(
        &self,
        lir: Vec<LIR<S>>,
        printer: &mut impl Print<Style = S>,
    ) -> Result<(), Box<dyn Error>>
    where
        S: Style,
    {
        for op in lir.into_iter() {
            match op {
                LIR::Bounded(text) => {
                    let resolve = text.resolve(&self.resolve);
                    printer.emit_text(resolve, text.style())?
                }
                LIR::Break(n) => printer.emit_break(Indent {
                    size: n,
                    chars: self.config.indent,
                })?,
            }
        }

        Ok(())
    }
}
