use std::fmt::{self, Display};

use wyst_core::{wyst_copy, wyst_data};
use wyst_style::Style;

use crate::{
    ir::{
        hir::{IndentationHIR, TextHIR, TextPlacement},
        HIR,
    },
    text::Text,
};

/// Each line goes through the following stages:
///
/// Start:
///   - indentation: consume
///   - content(interior): ignore
///   - content(exterior): Interior -> consume
///   - content(anywhere): Interior -> consume
///   - break: flush_line -> Indentation
///
/// Indentation:
///   - indentation: consume
///   - content(interior): ignore
///   - content(exterior): flush_line -> Interior -> consume
///   - content(anywhere): flush_line -> Interior -> consume
///   - break -> flush_line -> Indentation
///
/// Interior:
///   - indentation -> Buffering([op])
///   - content(interior) -> Buffering([consume])
///   - content(exterior) -> Buffering(exterior=consume)
///   - content(anywhere): consume
///   - break -> Indentation
///
/// Buffering:
///   - indentation -> Buffering([...ops, consume])
///   - content(interior) -> Buffering([...ops, consume])
///   - content(exterior) -> flush_interior(ops) -> Buffering(exterior=consume)
///   - content(anywhere) -> flush_interior(ops) -> Interior -> consume
///   - break -> flush_exterior(ops) -> Indentation
///
/// Operations:
///
/// flush_line:
///   - assert: empty SpeculativeBuffer
///   - emit: Break(indentation)
///
/// consume:
///   - emit: ToLIR(op)
///
/// flush_interior:
///   - for each speculative op:
///     - if op is compatible with Placement=Interior, emit
///
/// flush_exterior:
///   - if there's a speculative exterior op, emit
///   - for each speculative op:
///     - if op is compatible with Placement=Indentation, emit
#[wyst_copy]
pub(crate) enum LineStage {
    Start { indent: usize },
    Indentation { indent: usize },
    Interior { indent: usize },
    Buffering { indent: usize },
    EOF,
}

impl Display for LineStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineStage::Start { indent } => write!(f, "Start({})", indent),
            LineStage::Indentation { indent } => write!(f, "Indentation({})", indent),
            LineStage::Interior { indent } => write!(f, "Interior({})", indent),
            LineStage::Buffering { indent } => write!(f, "Buffering({})", indent),
            LineStage::EOF => write!(f, "EOF"),
        }
    }
}

impl LineStage {
    pub(crate) fn indent_level(self) -> usize {
        match self {
            LineStage::Start { indent } => indent,
            LineStage::Indentation { indent } => indent,
            LineStage::Interior { indent } => indent,
            LineStage::Buffering { indent } => indent,
            LineStage::EOF => 0,
        }
    }

    pub(crate) fn indent(self) -> LineStage {
        match self {
            LineStage::Start { indent } => LineStage::Start { indent: indent + 1 },
            LineStage::Indentation { indent } => LineStage::Indentation { indent: indent + 1 },
            LineStage::Interior { indent } => LineStage::Interior { indent: indent + 1 },
            LineStage::Buffering { indent } => LineStage::Buffering { indent: indent + 1 },
            LineStage::EOF => self,
        }
    }

    pub(crate) fn outdent(self) -> LineStage {
        match self {
            LineStage::Start { indent } => LineStage::Start { indent: indent - 1 },
            LineStage::Indentation { indent } => LineStage::Indentation { indent: indent - 1 },
            LineStage::Interior { indent } => LineStage::Interior { indent: indent - 1 },
            LineStage::Buffering { indent } => LineStage::Buffering { indent: indent - 1 },
            LineStage::EOF => self,
        }
    }

    /// Determine what to do next. Before calling this method, filter out break opportunities that
    /// should be skipped.
    pub(crate) fn do_next<S>(self, hir: HIR<S>) -> NextStage<S>
    where
        S: Style,
    {
        match self {
            LineStage::Start { indent } => match hir {
                HIR::Bounded(TextHIR { placement, text }) => match placement {
                    TextPlacement::Interior => NextStage::Ignore,
                    TextPlacement::Exterior | TextPlacement::Anywhere => NextStage::TransitionTo {
                        next: LineStage::Interior { indent },
                        then_consume: Some(text),
                    },
                },
                HIR::Indentation(IndentationHIR::Indent) => NextStage::TransitionTo {
                    next: LineStage::Start { indent: indent + 1 },
                    then_consume: None,
                },
                HIR::Indentation(IndentationHIR::Outdent) => NextStage::TransitionTo {
                    next: LineStage::Start { indent: indent - 1 },
                    then_consume: None,
                },
                HIR::BreakOpportunity(_) => NextStage::FlushLine(FlushLine {
                    next: LineStage::Indentation { indent },
                    then_consume: None,
                }),
                HIR::EOF => NextStage::FlushLine(FlushLine {
                    next: LineStage::EOF,
                    then_consume: None,
                }),
            },
            LineStage::Indentation { indent } => match hir {
                HIR::Bounded(TextHIR { placement, text }) => match placement {
                    TextPlacement::Interior => NextStage::Ignore,
                    TextPlacement::Exterior | TextPlacement::Anywhere => {
                        NextStage::FlushLine(FlushLine {
                            next: LineStage::Interior { indent },
                            then_consume: Some(text),
                        })
                    }
                },
                HIR::Indentation(IndentationHIR::Indent) => NextStage::TransitionTo {
                    next: LineStage::Indentation { indent: indent + 1 },
                    then_consume: None,
                },
                HIR::Indentation(IndentationHIR::Outdent) => NextStage::TransitionTo {
                    next: LineStage::Indentation { indent: indent - 1 },
                    then_consume: None,
                },
                HIR::BreakOpportunity(_) => NextStage::FlushLine(FlushLine {
                    next: LineStage::Indentation { indent },
                    then_consume: None,
                }),
                HIR::EOF => NextStage::FlushLine(FlushLine {
                    next: LineStage::EOF,
                    then_consume: None,
                }),
            },
            LineStage::Interior { indent } => match hir {
                HIR::Bounded(TextHIR { placement, text }) => match placement {
                    TextPlacement::Interior => NextStage::InitializeBuffer {
                        initialize: InitializeBuffer::Interior(text),
                        next: LineStage::Buffering { indent },
                    },
                    TextPlacement::Exterior => NextStage::InitializeBuffer {
                        initialize: InitializeBuffer::Exterior(text),
                        next: LineStage::Buffering { indent },
                    },
                    TextPlacement::Anywhere => NextStage::Consume { consume: text },
                },
                HIR::Indentation(indentation) => NextStage::Buffer {
                    consume: indentation.into(),
                    next: LineStage::Buffering { indent },
                },
                HIR::BreakOpportunity(_) => NextStage::TransitionTo {
                    next: LineStage::Indentation { indent },
                    then_consume: None,
                },
                HIR::EOF => NextStage::FlushLine(FlushLine {
                    next: LineStage::EOF,
                    then_consume: None,
                }),
            },
            LineStage::Buffering { indent } => match hir {
                HIR::Bounded(TextHIR { placement, text }) => match placement {
                    TextPlacement::Interior => NextStage::Buffer {
                        consume: text.into(),
                        next: LineStage::Buffering { indent },
                    },
                    TextPlacement::Exterior => NextStage::PeekedExterior {
                        next: LineStage::Buffering { indent },
                        exterior: text,
                    },
                    TextPlacement::Anywhere => NextStage::PeekedAnywhere {
                        next: LineStage::Interior { indent },
                        consume: text,
                    },
                },
                HIR::Indentation(indentation) => NextStage::Buffer {
                    consume: indentation.into(),
                    next: LineStage::Buffering { indent },
                },
                HIR::BreakOpportunity(_) => NextStage::FlushExterior {
                    next: LineStage::Indentation { indent },
                },
                HIR::EOF => NextStage::FlushExteriorAndLine(FlushLine {
                    next: LineStage::EOF,
                    then_consume: None,
                }),
            },
            LineStage::EOF => NextStage::EOF,
        }
    }
}

#[wyst_copy]
pub(crate) struct FlushLine<S>
where
    S: Style,
{
    pub(crate) next: LineStage,
    pub(crate) then_consume: Option<Text<S>>,
}

impl<S> Display for FlushLine<S>
where
    S: Style,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FlushLine(next={}", self.next)?;

        if let Some(then_consume) = self.then_consume {
            write!(f, ", then_consume={}", then_consume)?;
        }

        write!(f, ")")
    }
}

#[wyst_copy]
pub enum InitializeBuffer<S>
where
    S: Style,
{
    Exterior(Text<S>),
    Interior(Text<S>),
}

impl<S> Display for InitializeBuffer<S>
where
    S: Style,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InitializeBuffer::Exterior(text) => write!(f, "exterior {}", text),
            InitializeBuffer::Interior(text) => write!(f, "interior {}", text),
        }
    }
}

#[wyst_copy]
pub(crate) enum NextStage<S>
where
    S: Style,
{
    /// Consume the text into the speculative buffer
    Buffer {
        next: LineStage,
        consume: SpeculativeHIR<S>,
    },
    /// Initialize the speculative buffer with a piece of exterior text.
    InitializeBuffer {
        next: LineStage,
        initialize: InitializeBuffer<S>,
    },
    /// Consume the text into the complete buffer
    Consume { consume: Text<S> },
    /// Ignore the text
    Ignore,
    /// Flush the current line, and optionally consume the text, if any.
    FlushLine(FlushLine<S>),
    /// Flush any speculative content as exterior content, and then transition.
    FlushExterior { next: LineStage },
    /// Flush any speculative content as exterior content, then flush the line, then transition.
    FlushExteriorAndLine(FlushLine<S>),
    /// Flush any speculative content as interior content, and then transition back into Buffer,
    /// initializing with the Text.
    PeekedExterior { next: LineStage, exterior: Text<S> },
    /// Flush any speculative content as interior content, transition into Interior, and then
    /// consume the Text.
    PeekedAnywhere { next: LineStage, consume: Text<S> },
    /// Transition to a new stage, and then optionally consume the Text.
    TransitionTo {
        next: LineStage,
        then_consume: Option<Text<S>>,
    },
    /// Nothing left to do
    EOF,
}

impl<S> Display for NextStage<S>
where
    S: Style,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NextStage::Buffer { next, consume } => write!(f, "Buffer({}) -> {}", consume, next),
            NextStage::InitializeBuffer { next, initialize } => {
                write!(f, "InitializeBuffer({}) -> {}", initialize, next)
            }
            NextStage::Consume { consume } => write!(f, "Consume({})", consume),
            NextStage::Ignore => write!(f, "Ignore"),
            NextStage::FlushLine(line) => write!(f, "{}", line),
            NextStage::FlushExterior { next } => write!(f, "FlushExterior({})", next),
            NextStage::FlushExteriorAndLine(flush) => {
                write!(f, "FlushExterior({}) and line", flush)
            }
            NextStage::PeekedExterior { next, exterior } => {
                write!(f, "PeekedExterior({}, next={})", exterior, next)
            }
            NextStage::PeekedAnywhere { next, consume } => {
                write!(f, "PeekedAnywhere({}, next={})", consume, next)
            }
            NextStage::TransitionTo { next, then_consume } => {
                write!(f, "TransitionTo({}", next)?;

                if let Some(then_consume) = then_consume {
                    write!(f, " and consume {}", then_consume)?;
                }

                write!(f, ")")
            }
            NextStage::EOF => write!(f, "EOF"),
        }
    }
}

/// We buffer operations that are only allowed in interior or exterior positions, but before we know
/// whether we're in an interior or exterior position.
///
/// The buffer is flushed when:
///
/// - content(exterior):
///   - Either this is the exterior op, or there are more ops after it. Either way, there will be
///     more content ops after the current buffer, so resolve the buffered ops with interior
///     placement.
/// - content(anywhere):
///   - There is more content after the current buffer, so resolve the buffered ops with interior
///     placement.
/// - break:
///   - There is no more content after the current buffer, so resolve the buffered ops with exterior
///     placement.
///
/// Resolving the buffer with interior placement:
/// - consume any ops in the buffer with interior placement
///
/// Resolving the buffer with exterior placement:
///
/// - If there is an op with exterior placement in the buffer:
///   - consume the op
///   - consume any ops in the buffer with exterior placement
#[wyst_data]
pub(crate) struct SpeculativeBuffer<S>
where
    S: Style,
{
    /// Optionally, a buffered exterior op. At any given time, this is the only buffered operation
    /// with Placement=Exterior, and it is always the first buffered op if it exists.
    exterior_op: Option<Text<S>>,
    /// A list of buffered that have not yet been resolved (either Placement=Interior or
    /// Placement=Indentation). If another piece of text is emitted on the same line, the buffer
    /// will be resolved with Placement=Interior. Otherwise, the buffer will be resolved with
    /// Placement=Indentation when the next break occurs.
    buffer: Vec<SpeculativeHIR<S>>,
}

impl<S> Default for SpeculativeBuffer<S>
where
    S: Style,
{
    fn default() -> Self {
        SpeculativeBuffer {
            exterior_op: None,
            buffer: vec![],
        }
    }
}

impl<S> SpeculativeBuffer<S>
where
    S: Style,
{
    pub(crate) fn push(&mut self, op: SpeculativeHIR<S>) {
        self.buffer.push(op);
    }

    pub(crate) fn initialize(&mut self, initialize: InitializeBuffer<S>) {
        match initialize {
            InitializeBuffer::Exterior(exterior) => self.exterior_op = Some(exterior),
            InitializeBuffer::Interior(interior) => {
                self.buffer.push(SpeculativeHIR::Interior(interior))
            }
        }
    }

    pub(crate) fn flush_exterior<'a>(
        &'a mut self,
    ) -> (Option<Text<S>>, impl Iterator<Item = IndentationHIR> + 'a) {
        let drained = self.buffer.drain(..).filter_map(|s| match s {
            SpeculativeHIR::Interior(_) => None,
            SpeculativeHIR::Indentation(i) => Some(i),
        });

        let text = self.exterior_op.take();

        (text, drained)
    }

    pub(crate) fn flush_interior<'a>(&'a mut self) -> impl Iterator<Item = Text<S>> + 'a {
        self.exterior_op = None;

        self.buffer.drain(..).filter_map(|s| match s {
            SpeculativeHIR::Interior(text) => Some(text),
            SpeculativeHIR::Indentation(_) => None,
        })
    }
}

/// HIR operations that do not have definite placement until we reach an operation that resolves
/// their placement.
#[wyst_copy]
pub(crate) enum SpeculativeHIR<S>
where
    S: Style,
{
    /// Text with interior placement. This operation will be emitted if other text is subsequently
    /// emitted on the same line.
    Interior(Text<S>),
    /// An indentation operation. This operation will be consumed if a break occurs before any other
    /// text is emitted.
    Indentation(IndentationHIR),
}

impl<S> Display for SpeculativeHIR<S>
where
    S: Style,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpeculativeHIR::Interior(text) => write!(f, "{}", text),
            SpeculativeHIR::Indentation(indent) => write!(f, "{}", indent),
        }
    }
}

impl<S> From<IndentationHIR> for SpeculativeHIR<S>
where
    S: Style,
{
    fn from(indent: IndentationHIR) -> Self {
        SpeculativeHIR::Indentation(indent)
    }
}

impl<S> From<Text<S>> for SpeculativeHIR<S>
where
    S: Style,
{
    fn from(text: Text<S>) -> Self {
        SpeculativeHIR::Interior(text)
    }
}
