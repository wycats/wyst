mod builder;
mod line;
mod lines;
mod opportunities;
mod process;

use std::{fmt::Display, ops::Index};

use uuid::Uuid;
use wyst_core::{wyst_copy, wyst_data, wyst_display};
use wyst_style::Style;

pub use self::builder::HirBuilder;
pub use self::lines::to_lines;
use crate::text::Text;

#[wyst_data]
pub struct Atomic<S>
where
    S: Style,
{
    pub children: Vec<HIR<S>>,
}

impl<S> Index<usize> for Atomic<S>
where
    S: Style,
{
    type Output = HIR<S>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.children[index]
    }
}

impl<S> Atomic<S>
where
    S: Style,
{
    pub fn new(children: Vec<HIR<S>>) -> Atomic<S> {
        Atomic { children }
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }
}

#[wyst_copy]
pub enum BreakId {
    Named(&'static str),
    Id(Uuid),
}

impl Display for BreakId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakId::Named(name) => write!(f, "{}", name),
            BreakId::Id(uuid) => {
                let hash = blake3::hash(uuid.as_bytes());
                let mut hash = hash.to_hex();
                hash.truncate(7);

                write!(f, "{}", hash.as_ref())
            }
        }
    }
}

impl Default for BreakId {
    fn default() -> Self {
        BreakId::Id(Uuid::default())
    }
}

impl BreakId {
    fn generate() -> Self {
        BreakId::Id(Uuid::new_v4())
    }
}

#[wyst_display("{}[{}]", "self.level", "self.id")]
#[wyst_copy]
pub struct NamedBreakLevel {
    pub(crate) level: usize,
    pub(crate) id: BreakId,
}

impl Into<BreakLevel> for NamedBreakLevel {
    fn into(self) -> BreakLevel {
        BreakLevel::Level(self)
    }
}

impl NamedBreakLevel {
    /// Does this `NamedBreakLevel` cover another [BreakLevel]. A `NamedBreakLevel` covers another
    /// [BreakLevel] if skipping this `NamedBreakLevel` implies that the other [BreakLevel] should
    /// be skipped.
    pub(crate) fn implies_skip(self, other: impl Into<BreakLevel>) -> bool {
        let other = other.into();

        match other {
            // An unconditional level can never be skipped.
            BreakLevel::Unconditional => false,

            BreakLevel::Level(NamedBreakLevel { level, id }) => {
                // If this `NamedBreakLevel` does not share an id with the other break level, it cannot
                // imply that the other break level is skipped.
                if self.id != id {
                    false
                } else {
                    // The alogrithm for taking a break is monotonic. This means that once break N is
                    // taken, breaks N - 1, ... are also taken. Therefore, if we know that we're going
                    // to skip break N, that also means that we will skip breaks N + 1, ...
                    //
                    // Therefore, if this `NamedBreakLevel`'s level is <= `level`, then skipping this
                    // level implies skipping the other level.
                    self.level <= level
                }
            }
        }
    }
}

#[wyst_copy]
pub enum BreakLevel {
    Unconditional,
    /// For a given BreakId, all breaks at a given level must either be taken or not taken.
    Level(NamedBreakLevel),
}

impl Display for BreakLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakLevel::Unconditional => write!(f, "br"),
            BreakLevel::Level(level) => write!(f, "wbr({})", level),
        }
    }
}

impl From<()> for BreakLevel {
    fn from(_level: ()) -> Self {
        BreakLevel::Unconditional
    }
}

impl Into<BreakOpportunity> for BreakLevel {
    fn into(self) -> BreakOpportunity {
        BreakOpportunity { level: self }
    }
}

impl<S> Into<HIR<S>> for BreakLevel
where
    S: Style,
{
    fn into(self) -> HIR<S> {
        HIR::BreakOpportunity(self.into())
    }
}

#[wyst_display("{}", "self.level")]
#[wyst_copy]
pub struct BreakOpportunity {
    pub(crate) level: BreakLevel,
}

impl<S> Into<HIR<S>> for BreakOpportunity
where
    S: Style,
{
    fn into(self) -> HIR<S> {
        HIR::BreakOpportunity(self)
    }
}

impl BreakOpportunity {
    fn unconditional() -> BreakOpportunity {
        BreakOpportunity {
            level: BreakLevel::Unconditional,
        }
    }

    fn conditional(id: BreakId, level: usize) -> BreakOpportunity {
        BreakOpportunity {
            level: BreakLevel::Level(NamedBreakLevel { level, id }),
        }
    }

    fn name(self) -> &'static str {
        match self.level {
            BreakLevel::Unconditional => "br",
            BreakLevel::Level(NamedBreakLevel { .. }) => "wbr",
        }
    }
}

#[wyst_copy]
pub enum IndentationHIR {
    Indent,
    Outdent,
}

impl Display for IndentationHIR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndentationHIR::Indent => write!(f, "indent"),
            IndentationHIR::Outdent => write!(f, "outdent"),
        }
    }
}

impl IndentationHIR {
    fn name(self) -> &'static str {
        match self {
            IndentationHIR::Indent => "Indent",
            IndentationHIR::Outdent => "Outdent",
        }
    }

    pub fn apply(self, current: usize) -> usize {
        match self {
            IndentationHIR::Indent => current + 1,
            IndentationHIR::Outdent => current - 1,
        }
    }
}

#[wyst_copy]
pub enum TextPlacement {
    /// Content is ignored at the edge.
    Interior,
    /// Content is only allowed at the edge.
    Exterior,
    /// Content is unconditionally included.
    Anywhere,
}

impl Display for TextPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextPlacement::Interior => write!(f, "interior"),
            TextPlacement::Exterior => write!(f, "exterior"),
            TextPlacement::Anywhere => write!(f, ""),
        }
    }
}

impl Into<Placement> for TextPlacement {
    fn into(self) -> Placement {
        match self {
            TextPlacement::Interior => Placement::Interior,
            TextPlacement::Exterior => Placement::Exterior,
            TextPlacement::Anywhere => Placement::Anywhere,
        }
    }
}

#[wyst_copy]
pub enum Placement {
    Interior,
    Exterior,
    Anywhere,
    Indentation,
}

#[wyst_copy]
pub struct TextHIR<S>
where
    S: Style,
{
    pub(crate) placement: TextPlacement,
    pub(crate) text: Text<S>,
}

impl<S> Display for TextHIR<S>
where
    S: Style,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Text(")?;
        write!(f, "{:?}, len={}", self.text.text, self.text.len())?;

        match self.placement {
            TextPlacement::Interior => write!(f, ", interior")?,
            TextPlacement::Exterior => write!(f, ", exterior")?,
            TextPlacement::Anywhere => {}
        }

        write!(f, ")")
    }
}

#[wyst_copy]
pub enum HIR<S>
where
    S: Style,
{
    Bounded(TextHIR<S>),
    Indentation(IndentationHIR),
    /// An opportunity for a break at a given level.
    BreakOpportunity(BreakOpportunity),
    EOF,
}

impl<S> Display for HIR<S>
where
    S: Style,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HIR::Bounded(text) => write!(f, "{}", text),
            HIR::Indentation(IndentationHIR::Indent) => write!(f, "indent"),
            HIR::Indentation(IndentationHIR::Outdent) => write!(f, "outdent"),
            HIR::BreakOpportunity(br) => write!(f, "{}", br),
            HIR::EOF => write!(f, "EOF"),
        }
    }
}

impl<S> HIR<S>
where
    S: Style,
{
    pub(crate) fn name(self) -> &'static str {
        match self {
            HIR::Bounded(_) => "Bounded",
            HIR::Indentation(indent) => indent.name(),
            HIR::BreakOpportunity(wbr) => wbr.name(),
            HIR::EOF => "EOF",
        }
    }

    pub(crate) fn bounded(text: Text<S>, placement: TextPlacement) -> HIR<S> {
        HIR::Bounded(TextHIR { placement, text })
    }

    pub(crate) fn wbr(id: BreakId, level: usize) -> HIR<S> {
        HIR::BreakOpportunity(BreakOpportunity::conditional(id, level))
    }

    pub(crate) fn br() -> HIR<S> {
        HIR::BreakOpportunity(BreakOpportunity::unconditional())
    }
}
