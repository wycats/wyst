use std::marker::PhantomData;

use crate::texts::Texts;
use wyst_style::Style;

pub struct FragmentBuilder<S>
where
    S: Style,
{
    texts: Texts,
    style: PhantomData<S>,
}

/// This is the core interface that makes it possible to build each of the fragments.
impl<S> FragmentBuilder<S>
where
    S: Style,
{
    pub fn new(texts: impl Into<Texts>) -> FragmentBuilder<S> {
        FragmentBuilder {
            texts: texts.into(),
            style: PhantomData,
        }
    }

    pub fn done(self) -> Texts {
        self.texts
    }

    // pub fn empty(&mut self) -> Fragment<S> {
    //     Fragment::Atom(Atom::Empty)
    // }

    // pub fn atom(&mut self, string: impl AsRef<str>) -> Fragment<S> {
    //     Fragment::Atom(Atom::Atomic(self.texts.styled(string, S::normal())))
    // }

    // pub fn styled(&mut self, string: impl AsRef<str>, style: S) -> Fragment<S> {
    //     Fragment::Atom(Atom::Atomic(self.texts.styled(string, style)))
    // }

    // pub fn boundary(&mut self, string: impl AsRef<str>) -> Fragment<S> {
    //     Fragment::Atom(Atom::Boundary(self.texts.styled(string, S::invisible())))
    // }

    // pub fn group(&mut self, list: impl FnOnce(&mut Self) -> Vec<Fragment<S>>) -> Fragment<S> {
    //     Fragment::Layout(Layout::Group(list(self)))
    // }

    // pub fn wrap(&mut self, list: impl FnOnce(&mut Self) -> Vec<Fragment<S>>) -> Fragment<S> {
    //     Fragment::Layout(Layout::Wrap(list(self)))
    // }

    // pub fn nest(
    //     &mut self,
    //     nest: impl FnOnce(&mut Self) -> (Fragment<S>, Fragment<S>, Fragment<S>),
    // ) -> Fragment<S> {
    //     let (prefix, body, suffix) = nest(self);

    //     Fragment::Layout(Layout::Nest(Nest::boxed(prefix, suffix, body)))
    // }

    // pub fn choice(
    //     &mut self,
    //     choice: impl FnOnce(&mut Self) -> (Fragment<S>, Fragment<S>),
    // ) -> Fragment<S> {
    //     let (inline, block) = choice(self);
    //     Fragment::Layout(Layout::Choice(Choice::boxed(inline, block)))
    // }

    // pub fn space(&mut self) -> Fragment<S> {
    //     self.boundary(" ")
    // }
}
