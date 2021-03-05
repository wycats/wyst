use crate::{
    ir::{
        hir::{BreakId, IndentationHIR, TextPlacement},
        Atomic, HIR,
    },
    text::Text,
    texts::Texts,
};
use wyst_style::Style;

pub struct HirBuilder<'texts, S>
where
    S: Style,
{
    texts: &'texts mut Texts,
    current_id: BreakId,
    id_stack: Vec<BreakId>,
    current: Vec<HIR<S>>,
}

impl<'texts, S> HirBuilder<'texts, S>
where
    S: Style,
{
    pub fn new(texts: &'texts mut Texts) -> HirBuilder<'texts, S> {
        HirBuilder {
            texts,
            current_id: BreakId::default(),
            id_stack: vec![],
            current: vec![],
        }
    }

    pub fn done(mut self) -> Atomic<S> {
        self.current.push(HIR::EOF);
        Atomic::new(self.current)
    }

    fn add(mut self, hir: HIR<S>) -> Self {
        self.current.push(hir);
        self
    }

    fn intern(&mut self, text: impl AsRef<str>, style: S) -> Text<S> {
        self.texts.styled(text, style)
    }

    pub fn text(self, text: impl AsRef<str>) -> Self {
        self.styled(text, S::normal())
    }

    pub fn text_at(self, text: impl AsRef<str>, placement: impl Into<TextPlacement>) -> Self {
        self.styled_at(text, S::normal(), placement)
    }

    pub fn styled(self, text: impl AsRef<str>, style: S) -> Self {
        self.styled_at(text, style, TextPlacement::Anywhere)
    }

    pub fn styled_at(
        mut self,
        text: impl AsRef<str>,
        style: S,
        placement: impl Into<TextPlacement>,
    ) -> Self {
        let text = self.intern(text, style);
        self.add(HIR::bounded(text, placement.into()))
    }

    pub fn wbr(self, level: usize) -> Self {
        let current = self.current_id;
        self.add(HIR::wbr(current, level))
    }

    pub fn br(self) -> Self {
        self.add(HIR::br())
    }

    pub fn space(self, text: impl AsRef<str>) -> Self {
        self.styled_at(text, S::invisible(), TextPlacement::Interior)
    }

    pub fn nest(self, level: usize, ops: impl FnOnce(Self) -> Self) -> Self {
        ops(self.indent().wbr(level)).outdent()
    }

    pub fn group(self, ops: impl FnOnce(Self) -> Self) -> Self {
        let builder = self.start(BreakId::generate());
        let builder = ops(builder);
        builder.end()
    }

    pub fn start(mut self, mut id: BreakId) -> Self {
        std::mem::swap(&mut id, &mut self.current_id);
        self.id_stack.push(id);
        self
    }

    pub fn end(mut self) -> Self {
        let last = self.id_stack.pop().expect("unbalanced push/pop");
        self.current_id = last;
        self
    }

    pub fn indent(self) -> Self {
        self.add(HIR::Indentation(IndentationHIR::Indent))
    }

    pub fn outdent(self) -> Self {
        self.add(HIR::Indentation(IndentationHIR::Outdent))
    }
}
