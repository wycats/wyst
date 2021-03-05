use wyst_core::wyst_copy;
use wyst_core_traits::WystCopy;

pub trait Style: WystCopy + Default {
    fn invisible() -> Self;
    fn normal() -> Self;
}

#[wyst_copy]
#[derive(Default)]
pub struct PlainStyle;

impl Style for PlainStyle {
    fn invisible() -> Self {
        PlainStyle
    }

    fn normal() -> Self {
        PlainStyle
    }
}
