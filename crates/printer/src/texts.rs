use lasso::{Rodeo, RodeoResolver, Spur};

use wyst_core::wyst_data_value;
use wyst_style::Style;

use crate::text::Text;

#[wyst_data_value]
#[derive(Default)]
pub struct Texts {
    rodeo: Rodeo,
}

impl Texts {
    pub fn intern(&mut self, str: impl AsRef<str>) -> Spur {
        self.rodeo.get_or_intern(str)
    }

    pub fn styled<S>(&mut self, string: impl AsRef<str>, style: S) -> Text<S>
    where
        S: Style,
    {
        let string = string.as_ref();
        let spur = self.intern(string);
        Text::new(spur, string.len(), style)
    }

    pub fn read(self) -> ReadTexts {
        ReadTexts {
            rodeo: self.rodeo.into_resolver(),
        }
    }

    pub fn resolver(&self) -> ResolveTexts<'_> {
        ResolveTexts::Mutable(self)
    }
}

#[wyst_data_value]
pub struct ReadTexts {
    rodeo: RodeoResolver,
}

#[wyst_data_value]
pub enum ResolveTexts<'texts> {
    Mutable(&'texts Texts),
    Readonly(&'texts ReadTexts),
}

impl<'texts> ResolveTexts<'texts> {
    pub(crate) fn resolve(&'texts self, key: Spur) -> &'texts str {
        match self {
            ResolveTexts::Mutable(texts) => texts.rodeo.resolve(&key),
            ResolveTexts::Readonly(texts) => texts.rodeo.resolve(&key),
        }
    }

    pub(crate) fn clone(&self) -> ResolveTexts<'texts> {
        match self {
            ResolveTexts::Mutable(t) => ResolveTexts::Mutable(*t),
            ResolveTexts::Readonly(r) => ResolveTexts::Readonly(*r),
        }
    }
}

impl<'texts> Into<ResolveTexts<'texts>> for &'texts Texts {
    fn into(self) -> ResolveTexts<'texts> {
        ResolveTexts::Mutable(self)
    }
}

impl<'texts> Into<ResolveTexts<'texts>> for &'texts ReadTexts {
    fn into(self) -> ResolveTexts<'texts> {
        ResolveTexts::Readonly(self)
    }
}
