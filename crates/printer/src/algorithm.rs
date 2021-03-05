use wyst_style::Style;

use crate::{
    ir::{to_lines, Atomic, LIR},
    PrintConfig,
};

pub fn layout<S>(atomic: Atomic<S>, config: impl Into<PrintConfig>) -> Vec<LIR<S>>
where
    S: Style,
{
    let lines = to_lines(config.into(), &atomic.children);
    lines.to_lir()
}
