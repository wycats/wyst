use wyst_core::{wyst_data, WystData};
use wyst_core_traits::WystCopy;

use crate::span::Span;

#[wyst_data(new)]
pub struct Spanned<T>
where
    T: WystData,
{
    span: Span,
    item: T,
}

impl<T> Spanned<T>
where
    T: WystData,
{
    pub fn item(&self) -> &T {
        &self.item
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn map<U>(self, mapper: impl FnOnce(T) -> U) -> Spanned<U>
    where
        U: WystCopy,
    {
        Spanned::new(self.span, mapper(self.item))
    }
}

pub trait AddSpan: WystData {
    fn spanned(self, span: impl Into<Span>) -> Spanned<Self> {
        Spanned::new(span.into(), self)
    }
}

impl<T> AddSpan for T where T: WystData {}
