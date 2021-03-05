mod builder;

use wyst_core::wyst_copy;
use wyst_style::Style;

pub use self::builder::LirBuilder;
pub use crate::ir::{PrintConfig, Printer};
use crate::text::Text;

#[wyst_copy]
pub enum LIR<S>
where
    S: Style,
{
    Bounded(Text<S>),
    Break(usize),
}

pub fn measure_lir(ops: &[LIR<impl Style>], nesting_len: usize) -> usize {
    let mut max_width = 0;
    let mut current_width = 0;

    for op in ops.iter() {
        match op {
            LIR::Bounded(bounded) => {
                current_width += bounded.len();
            }
            LIR::Break(indent) => {
                max_width = max_width.max(current_width);
                current_width = indent * nesting_len;
            }
        }
    }

    max_width.max(current_width)
}
