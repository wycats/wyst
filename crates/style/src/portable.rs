use std::error::Error;

use crossterm::{
    style::{Color, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};
use wyst_core::wyst_copy;

use crate::Style;

#[wyst_copy]
#[derive(Default)]
pub struct PortableStyle {
    fg: Option<PortableColor>,
    bg: Option<PortableColor>,
    bold_hint: bool,
}

impl PortableStyle {
    pub fn apply_style(self, buf: &mut impl ExecutableCommand) -> Result<(), Box<dyn Error>> {
        if let Some(fg) = self.fg {
            buf.execute(SetForegroundColor(fg.into()))?;
            // buf.set_color(&fg.fg())?
        }

        if let Some(bg) = self.bg {
            buf.execute(SetBackgroundColor(bg.into()))?;
        }

        Ok(())
    }
}

impl Style for PortableStyle {
    fn invisible() -> Self {
        PortableStyle::normal()
    }

    fn normal() -> Self {
        PortableStyle {
            fg: None,
            bg: None,
            bold_hint: false,
        }
    }
}

/// Windows: https://stackoverflow.com/questions/17125440/c-win32-console-color/17125539
/// ANSI: https://en.wikipedia.org/wiki/ANSI_escape_code
#[wyst_copy]
pub enum PortableColor {
    Black,
    /// "bright white"
    White,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkBlue,
    DarkMagenta,
    DarkCyan,
    /// "bright black"
    DarkGray,
    /// "white"
    LightGrey,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
}

impl Into<Color> for PortableColor {
    fn into(self) -> Color {
        match self {
            PortableColor::Black => Color::Black,
            PortableColor::White => Color::White,
            PortableColor::DarkRed => Color::DarkRed,
            PortableColor::DarkGreen => Color::DarkGreen,
            PortableColor::DarkYellow => Color::DarkYellow,
            PortableColor::DarkBlue => Color::DarkBlue,
            PortableColor::DarkMagenta => Color::DarkMagenta,
            PortableColor::DarkCyan => Color::DarkCyan,
            PortableColor::DarkGray => Color::DarkGrey,
            PortableColor::LightGrey => Color::Grey,
            PortableColor::LightRed => Color::Red,
            PortableColor::LightGreen => Color::Green,
            PortableColor::LightYellow => Color::Yellow,
            PortableColor::LightBlue => Color::Blue,
            PortableColor::LightMagenta => Color::Magenta,
            PortableColor::LightCyan => Color::Cyan,
        }
    }
}
