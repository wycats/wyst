use std::{error::Error, fmt::Debug};

use crossterm::{style::Print as PrintCommand, ExecutableCommand};

use crate::{PortableStyle, Print, Style};

pub struct PrintCrossterm<'write, E>
where
    E: ExecutableCommand,
{
    write: &'write mut E,
}

impl<'write, E> PrintCrossterm<'write, E>
where
    E: ExecutableCommand,
{
    pub fn new(write: &'write mut E) -> PrintCrossterm<'write, E> {
        PrintCrossterm { write }
    }
}

impl<'write, E> Debug for PrintCrossterm<'write, E>
where
    E: ExecutableCommand,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrintCrossterm")
    }
}

impl<'write, E> Print for PrintCrossterm<'write, E>
where
    E: ExecutableCommand,
{
    type Style = PortableStyle;

    fn emit_text(&mut self, text: &str, style: Self::Style) -> Result<(), Box<dyn Error>> {
        style.apply_style(self.write)?;
        self.write.execute(PrintCommand(text))?;

        Ok(())
    }

    fn emit_break(&mut self, indent: crate::Indent<'_>) -> Result<(), Box<dyn Error>> {
        PortableStyle::invisible().apply_style(self.write)?;
        self.write.execute(PrintCommand("\n"))?;

        for _ in 0..indent.size {
            self.write.execute(PrintCommand(indent.chars))?;
        }

        Ok(())
    }
}
