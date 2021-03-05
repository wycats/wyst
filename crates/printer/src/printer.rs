use wyst_core::wyst_copy;

#[wyst_copy]
pub struct PrinterConfig {
    page_width: usize,
    nest: &'static str,
}

impl From<usize> for PrinterConfig {
    fn from(page_width: usize) -> Self {
        PrinterConfig {
            page_width,
            nest: "  ",
        }
    }
}

impl Default for PrinterConfig {
    fn default() -> Self {
        PrinterConfig {
            page_width: 80,
            nest: "  ",
        }
    }
}

#[wyst_copy]
pub struct Nesting {
    pub(crate) level: usize,
}

impl Default for Nesting {
    fn default() -> Self {
        Nesting { level: 0 }
    }
}

impl From<usize> for Nesting {
    fn from(level: usize) -> Self {
        Nesting { level }
    }
}
