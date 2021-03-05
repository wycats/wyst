pub mod crossterm;

use std::error::Error;

use crate::Style;

pub use self::crossterm::PrintCrossterm;

pub struct Indent<'chars> {
    pub size: usize,
    pub chars: &'chars str,
}

pub trait Print: std::fmt::Debug {
    type Style: Style;

    fn emit_text(&mut self, text: &str, style: Self::Style) -> Result<(), Box<dyn Error>>;
    fn emit_break(&mut self, indent: Indent<'_>) -> Result<(), Box<dyn Error>>;
}
