use wyst_core::{wyst_copy, wyst_data, wyst_display, WystEmpty};
use wyst_style::Style;
use wyst_utils::{WystMap, WystSet};

use crate::{
    ir::{
        hir::{
            process::{FlushLine, InitializeBuffer, LineStage, NextStage, SpeculativeBuffer},
            BreakId, IndentationHIR, NamedBreakLevel,
        },
        HIR, LIR,
    },
    text::Text,
    PrintConfig,
};

#[wyst_data]
struct Breaks {
    map: WystMap<usize, WystSet<BreakId>>,
}

impl WystEmpty for Breaks {
    fn empty() -> Self {
        Breaks {
            map: WystMap::empty(),
        }
    }
}

impl Breaks {
    fn skip(&mut self, NamedBreakLevel { level, id }: NamedBreakLevel) {
        self.map
            .entry_mut(level)
            .upsert(|| WystSet::empty(), |set| set.add(id));
    }

    /// The next break level that should be attempted. If there was no skipped break level in this
    /// line, return None.
    fn next_br(self) -> Option<NamedBreakLevel> {
        let (level, set) = self.map.iter().min_by_key(|(k, _)| *k)?;
        let id = set.iter().nth(0)?;

        Some(NamedBreakLevel { level, id })
    }
}

pub(crate) enum ProcessOp<S>
where
    S: Style,
{
    Flush(FlushLine<S>),
    Nothing,
    EOF,
}

/// A `LineBuffer` is responsible for processing [HIR] instructions, and remembering the break
/// opportunities that could have been taken. If a [LineBuffer] finishes without overflowing the
/// page width, it's turned into a [Line].
///
/// The [BreakOpportunities] struct is responsible for remembering break opportunities in previous
/// lines that weren't taken. If a [LineBuffer] overflows the page width, the break opportunities in
/// the current line will be used to determine which previous break opportunity should be taken.
#[wyst_data]
pub struct LineBuffer<S>
where
    S: Style,
{
    lir: Vec<Text<S>>,
    pub(crate) lineno: RewindableLine,
    pub(crate) stage: LineStage,
    config: PrintConfig,
    buffer: SpeculativeBuffer<S>,
    breaks: Breaks,
}

impl<S> LineBuffer<S>
where
    S: Style,
{
    pub(crate) fn first(config: PrintConfig) -> LineBuffer<S> {
        LineBuffer::start_line(RewindableLine::first(), config)
    }

    pub(crate) fn start_line(lineno: RewindableLine, config: PrintConfig) -> LineBuffer<S> {
        LineBuffer {
            lir: vec![],
            lineno,
            stage: lineno.line_start_stage(),
            config,
            buffer: SpeculativeBuffer::default(),
            breaks: Breaks::empty(),
        }
    }

    pub(crate) fn next_buf(&self, hir_offset: usize, config: PrintConfig) -> LineBuffer<S> {
        LineBuffer::start_line(
            self.lineno
                .next_lineno(hir_offset, self.stage.indent_level()),
            config,
        )
    }

    pub(crate) fn flush(self, flush: FlushLine<S>) -> FlushedLine<S> {
        let breaks = self.breaks;

        let next_line = Line {
            lir: self.lir,
            pre_indent: self.lineno.pre_indent,
            post_indent: flush.next.indent_level(),
            lineno: self.lineno,
        };

        if next_line.fits(self.config) {
            FlushedLine::Flushed(next_line)
        } else {
            FlushedLine::NoFit {
                line: next_line,
                try_skip: breaks.next_br(),
            }
        }
    }

    pub(crate) fn skip(&mut self, level: NamedBreakLevel) {
        self.breaks.skip(level);
    }

    pub(crate) fn push(&mut self, text: Text<S>) {
        self.lir.push(text);
    }

    fn flush_exterior(&mut self, mut next: LineStage) {
        let (exterior, indents) = self.buffer.flush_exterior();

        if let Some(exterior) = exterior {
            self.lir.push(exterior);
        }

        for op in indents {
            match op {
                IndentationHIR::Indent => {
                    next = next.indent();
                }
                IndentationHIR::Outdent => {
                    next = next.outdent();
                }
            }
        }

        self.stage = next;
    }

    fn flush_line(&mut self, flush: FlushLine<S>) -> ProcessOp<S> {
        printf_debug!("FLUSH {}", flush);
        ProcessOp::Flush(flush)
    }

    /// This method returns a [FlushLine] if the next stage is [FlushLine].
    pub(crate) fn process(&mut self, op: HIR<S>) -> ProcessOp<S> {
        let next = self.stage.do_next(op);

        printf_debug!("      DO {}", next);

        match next {
            NextStage::Buffer { next, consume } => {
                self.buffer.push(consume);
                self.stage = next;
            }
            NextStage::InitializeBuffer { next, initialize } => {
                self.buffer.initialize(initialize);
                self.stage = next;
            }
            NextStage::Consume { consume } => {
                self.lir.push(consume);
            }
            NextStage::Ignore => {
                // do nothing
            }
            NextStage::FlushExteriorAndLine(flush) => {
                self.flush_exterior(flush.next);
                return self.flush_line(flush);
            }
            NextStage::FlushLine(flush) => {
                return self.flush_line(flush);
            }
            NextStage::FlushExterior { next } => {
                self.flush_exterior(next);
            }
            NextStage::PeekedExterior { next, exterior } => {
                self.lir.extend(self.buffer.flush_interior());
                self.stage = next;
                self.buffer.initialize(InitializeBuffer::Exterior(exterior));
            }
            NextStage::PeekedAnywhere { next, consume } => {
                self.lir.extend(self.buffer.flush_interior());
                self.stage = next;
                self.lir.push(consume);
            }
            NextStage::TransitionTo { next, then_consume } => {
                self.stage = next;

                if let Some(then_consume) = then_consume {
                    self.lir.push(then_consume);
                }
            }
            NextStage::EOF => {
                printf_debug!("  EOF");
                return ProcessOp::EOF;
            }
        }

        printf_debug!("      -> {}", self.stage);

        ProcessOp::Nothing
    }
}

#[wyst_data]
pub(crate) enum FlushedLine<S>
where
    S: Style,
{
    Flushed(Line<S>),
    NoFit {
        try_skip: Option<NamedBreakLevel>,
        /// In case there are no further break opportunities, use this line.
        line: Line<S>,
    },
}

#[wyst_display(
    "RewindableLine({}, hir_offset={}, pre_indent={})",
    "self.lineno",
    "self.hir_offset",
    "self.pre_indent"
)]
#[wyst_copy]
pub struct RewindableLine {
    /// The offset in the Vec of lines that corresponds to this line. When this line is rewound,
    /// this line and everything after it will be truncated.
    pub(crate) lineno: usize,
    /// The offset in the HIR stream that corresponded to the start of this line. When this line is
    /// rewound, the HIR stream will be re-set to this offset.
    pub(crate) hir_offset: usize,
    /// The indentation size at the beginning of this line. When this line is rewound, the
    /// [LineBuffer] will be initialized with this indentation level.
    pre_indent: usize,
}

impl RewindableLine {
    fn first() -> RewindableLine {
        RewindableLine {
            lineno: 0,
            hir_offset: 0,
            pre_indent: 0,
        }
    }

    fn next_lineno(self, hir_offset: usize, pre_indent: usize) -> RewindableLine {
        RewindableLine {
            lineno: self.lineno + 1,
            hir_offset,
            pre_indent,
        }
    }

    fn line_start_stage(self) -> LineStage {
        if self.lineno == 0 && self.hir_offset == 0 {
            LineStage::Start {
                indent: self.pre_indent,
            }
        } else {
            LineStage::Indentation {
                indent: self.pre_indent,
            }
        }
    }
}

#[wyst_data]
pub struct Line<S>
where
    S: Style,
{
    /// The list of LIR ops that were accumulated for this line.
    lir: Vec<Text<S>>,
    /// The number of indents at the beginning of this line.
    pre_indent: usize,
    /// The number of indents in the line immediately following this line.
    post_indent: usize,
    /// The line number of this line as well as the offset of the first op in this line (in case we
    /// need to backtrack).
    lineno: RewindableLine,
}

impl<S> Line<S>
where
    S: Style,
{
    pub(crate) fn fits(&self, config: PrintConfig) -> bool {
        let width = self.width(config);
        width <= config.page_width
    }

    fn width(&self, config: PrintConfig) -> usize {
        let indent_width = config.indent_width(self.pre_indent);
        let text_width: usize = self.lir.iter().map(|t| t.len()).sum();
        indent_width + text_width
    }

    pub(crate) fn into_lir(self) -> impl Iterator<Item = LIR<S>> {
        self.lir
            .into_iter()
            .map(LIR::Bounded)
            .chain(Some(LIR::Break(self.post_indent)))
    }
}
