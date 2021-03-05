use crate::{ir::LIR, texts::Texts};
use wyst_style::Style;

pub struct LirBuilder<'texts, S>
where
    S: Style,
{
    texts: &'texts mut Texts,
    ops: Vec<LIR<S>>,
}

impl<'texts, S> LirBuilder<'texts, S>
where
    S: Style,
{
    pub fn build(
        texts: &'texts mut Texts,
        build: impl FnOnce(LirBuilder<'texts, S>) -> LirBuilder<'texts, S>,
    ) -> Vec<LIR<S>> {
        let builder = LirBuilder::new(texts);
        build(builder).done()
    }

    pub fn new(texts: &'texts mut Texts) -> LirBuilder<'texts, S> {
        LirBuilder { texts, ops: vec![] }
    }

    pub fn done(self) -> Vec<LIR<S>> {
        self.ops
    }

    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.ops
            .push(LIR::Bounded(self.texts.styled(text, S::normal())));
        self
    }

    pub fn space(self, text: impl AsRef<str>) -> Self {
        self.text(text)
    }

    pub fn styled(mut self, text: impl AsRef<str>, style: S) -> Self {
        self.ops.push(LIR::Bounded(self.texts.styled(text, style)));
        self
    }

    pub fn br(mut self, indent: usize) -> Self {
        self.ops.push(LIR::Break(indent));
        self
    }
}
