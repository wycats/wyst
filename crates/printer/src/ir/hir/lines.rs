use std::{
    iter::{Enumerate, Skip},
    slice::Iter,
};

use wyst_core::prelude::*;
use wyst_style::Style;
use wyst_utils::{MapEntryMut, WystCopyMap, WystMap};

use crate::{
    ir::{
        hir::{
            line::{FlushedLine, Line, LineBuffer, ProcessOp, RewindableLine},
            process::FlushLine,
            BreakId, BreakLevel, NamedBreakLevel,
        },
        HIR, LIR,
    },
    PrintConfig,
};

#[wyst_display("Rewind(break={}, line={})", "self.named_level", "self.line")]
#[wyst_copy]
pub struct RewindState {
    /// The break level for the break opportunity that was skipped.
    pub(crate) named_level: NamedBreakLevel,
    /// The line that contains the skipped break opportunity.
    pub(crate) line: RewindableLine,
}

impl RewindState {
    fn id(self) -> BreakId {
        self.named_level.id
    }

    fn level(self) -> usize {
        self.named_level.level
    }
}

/// `SkippedOpportunitiesForId` remembers the first line number
#[wyst_data]
pub struct SkippedOpportunitiesForId {
    lines: WystCopyMap<usize, RewindableLine>,
}

#[wyst_copy]
pub enum HandleOp<S>
where
    S: Style,
{
    Process(HIR<S>),
    Skip(NamedBreakLevel),
}

#[wyst_data]
struct Breaks {
    map: WystMap<usize, WystCopyMap<BreakId, RewindState>>,
}

impl WystEmpty for Breaks {
    fn empty() -> Self {
        Breaks {
            map: WystMap::empty(),
        }
    }
}

impl Breaks {
    /// Rewind to the next available break. This happens when a line doesn't fit into the page
    /// width, but there were no breaks in the line. As a result, the only thing we can do is rewind
    /// to the next available skipped break.
    ///
    /// The "next available skipped break" means the first skipped break for the lowest break level
    /// in the skipped breaks.
    fn take_next(&mut self) -> Option<RewindState> {
        let (_, map) = self.map.iter_mut().min_by_key(|(level, _)| *level)?;
        let (id, state) = map.iter().nth(0)?;
        map.delete(id);

        Some(state)
    }

    /// Rewind to a skipped break for a named break level. This happens when a line doesn't fit into
    /// the page width, and there were breaks in the line.
    ///
    /// When this happens, we select the first skipped break in the current line at the lowest break
    /// level in its skipped breaks. If that break is covered by a previously skipped break, we
    /// rewind back to that point and try again.
    fn take_break(&mut self, br: NamedBreakLevel) -> RewindState {
        match self.map.entry_mut(br.level) {
            MapEntryMut::Occupied(o) => {
                match o.get().delete(br.id) {
                    Some(rewind) => rewind,
                    None => panic!(
                        "Cannot take RewindState with id={:?} level={:?} (there were no skipped breaks with that id in level={:?})",
                        br.id,
                        br.level,
                        br.level
                    ),
                }
            }
            MapEntryMut::Vacant(_) => {
                panic!("Cannot take RewindState with level {} (there were no skipped breaks with that level)", br.level)
            }
        }
    }

    fn skip(&mut self, rewind: RewindState) {
        self.map.entry_mut(rewind.level()).upsert(
            || WystCopyMap::empty(),
            |map| {
                map.entry_mut(rewind.id()).upsert(
                    // If this is the first time we're seeing a [RewindState] for this level, add it.
                    || rewind,
                    // Otherwise, check to make sure that this [RewindState] is compatible with the
                    // last one we saw.
                    |state| {
                        if state.named_level.implies_skip(rewind.named_level) {
                            // Do nothing (`self.skipped` already implies that this break is skipped)
                        } else {
                            // If the existing skipped entry does not imply that this is skipped, update it.
                            // This means that we are about to skip a break opportunity that has a lower
                            // level than the ones we've already skipped.
                            todo!("Is this acceptable? I don't think so");
                        }

                        state
                    },
                )
            },
        )
    }
}

/// `BreakDecisions` remembers the line numbers of each [BreakId] that wasn't taken, and the highest
/// break level of each [BreakId] that was taken. If a line overflows the page width, the skipped
/// break opportunity with the lowest level will cause a rollback.
#[wyst_data]
pub struct BreakDecisions {
    /// All of the breaks that were skipped
    skipped: Breaks,
    /// The highest break level that was taken for a given [BreakId].
    taken: WystMap<BreakId, usize>,
}

impl BreakDecisions {
    fn empty() -> BreakDecisions {
        BreakDecisions {
            skipped: Breaks::empty(),
            taken: WystMap::empty(),
        }
    }

    /// Add the break to the list of taken breaks.
    fn take_break(&mut self, state: NamedBreakLevel) {
        self.taken
            .entry_mut(state.id)
            .upsert(|| 0, |level| *level = state.level)
    }

    fn rewind_for(&mut self, skipped_br: Option<NamedBreakLevel>) -> Option<RewindState> {
        match skipped_br {
            Some(skipped) => {
                printf_debug!(" TAKE {}", skipped);

                let next = self.skipped.take_break(skipped);
                self.take_break(next.named_level);

                printf_debug!("      -> {}", next);
                Some(next)
            }
            None => {
                printf_debug!(" TAKE NEXT");
                let next = self.skipped.take_next();

                match next {
                    None => {
                        printf_debug!("      -> None")
                    }
                    Some(state) => {
                        self.take_break(state.named_level);
                        printf_debug!("      -> {}", state);
                    }
                }

                next
            }
        }
    }

    /// Filter out the operations that correspond to [HIR::BreakOpportunity]s that we have decided
    /// to skip.
    fn handle_op<S>(&self, op: HIR<S>) -> HandleOp<S>
    where
        S: Style,
    {
        match op {
            HIR::BreakOpportunity(wbr) => match wbr.level {
                BreakLevel::Unconditional => HandleOp::Process(wbr.into()),
                BreakLevel::Level(NamedBreakLevel { id, level }) => match self.taken.get(id) {
                    // Skip [HIR::BreakOpportunities] if their level is higher than the level we
                    // already marked as taken for this ID.
                    Some(taken_level) if *taken_level >= level => HandleOp::Process(wbr.into()),
                    _ => HandleOp::Skip(NamedBreakLevel { id, level }),
                },
            },
            op => HandleOp::Process(op),
        }
    }

    fn skip(&mut self, rewind: RewindState) {
        self.skipped.skip(rewind);
    }
}

/// The `LinesBuffer` takes in [HIR] opcodes. Its job is to accumulate [LIR] opcodes, while
/// remembering which break opportunities were chosen for which lines. If a [Line] cannot fit within
/// the page width, the `LinesBuffer` will identify a break opportunity this way:
///
/// - Find the lowest break opportunity in the current [Line] that was skipped
/// - Update [BreakDecisions] to take that break opportunity
/// - Find the previous [LineNumber] that skipped that break opportunity
///   - if no such [LineNumber] exists, repeat the current line
///   - otherwise, rewind
#[wyst_data]
pub struct LinesBuffer<S>
where
    S: Style,
{
    breaks: BreakDecisions,
    lines: Vec<Line<S>>,
    current_line: LineBuffer<S>,
    config: PrintConfig,
}

#[wyst_data]
#[must_use]
pub(crate) enum ProcessResult<S>
where
    S: Style,
{
    EOF,
    Next,
    Flush(FlushResult<S>),
}

#[wyst_data]
#[must_use]
pub(crate) enum FlushResult<S>
where
    S: Style,
{
    Success(Line<S>),
    Rewind(RewindableLine),
}

impl<S> LinesBuffer<S>
where
    S: Style,
{
    fn new(config: PrintConfig) -> LinesBuffer<S> {
        LinesBuffer {
            breaks: BreakDecisions::empty(),
            lines: vec![],
            current_line: LineBuffer::first(config),
            config: config,
        }
    }

    fn done(self) -> Lines<S> {
        Lines::new(self.lines)
    }

    fn rewind(&mut self, line: RewindableLine) {
        self.lines.truncate(line.lineno);
        self.current_line = LineBuffer::start_line(line, self.config);
    }

    /// Returns `true` if EOF.
    pub(crate) fn process(&mut self, hir_offset: usize, op: HIR<S>) -> ProcessResult<S> {
        printf_debug!(" INSN {} @ {}", op.name(), self.current_line.stage);
        printf_debug!("      {} :: {}", hir_offset, op);

        match self.breaks.handle_op(op) {
            HandleOp::Process(op) => match self.current_line.process(op) {
                ProcessOp::Flush(flush) => ProcessResult::Flush(self.flush(flush, hir_offset)),
                ProcessOp::Nothing => ProcessResult::Next,
                ProcessOp::EOF => ProcessResult::EOF,
            },
            HandleOp::Skip(level) => {
                printf_debug!("      SKIP {}", level);
                self.skip(level);
                ProcessResult::Next
            }
        }
    }

    fn flush(&mut self, flush: FlushLine<S>, hir_offset: usize) -> FlushResult<S> {
        let mut buffer = self.current_line.next_buf(hir_offset, self.config);
        std::mem::swap(&mut buffer, &mut self.current_line);
        match buffer.flush(flush) {
            FlushedLine::Flushed(line) => {
                if let Some(then_consume) = flush.then_consume {
                    self.current_line.push(then_consume);
                }
                self.current_line.stage = flush.next;
                FlushResult::Success(line)
            }
            FlushedLine::NoFit { try_skip, line } => {
                match self.breaks.rewind_for(try_skip) {
                    // There are no more breaks to take, so just accumulate the line anyway.
                    None => FlushResult::Success(line),
                    Some(state) => FlushResult::Rewind(state.line),
                }
            }
        }
    }

    fn skip(&mut self, named_level: NamedBreakLevel) {
        self.breaks.skip(RewindState {
            named_level,
            line: self.current_line.lineno,
        });
        self.current_line.skip(named_level);
    }
}

#[wyst_data]
#[derive(new)]
pub struct Lines<S>
where
    S: Style,
{
    lines: Vec<Line<S>>,
}

impl<S> Lines<S>
where
    S: Style,
{
    pub(crate) fn to_lir(self) -> Vec<LIR<S>> {
        self.lines
            .into_iter()
            .flat_map(|line| line.into_lir())
            .collect()
    }
}

struct ToLines<'ops, S>
where
    S: Style,
{
    buffer: LinesBuffer<S>,
    ops: &'ops [HIR<S>],
    iterator: Skip<Enumerate<Iter<'ops, HIR<S>>>>,
}

impl<'ops, S> ToLines<'ops, S>
where
    S: Style,
{
    fn new(ops: &'ops [HIR<S>], config: PrintConfig) -> ToLines<'ops, S> {
        ToLines {
            buffer: LinesBuffer::new(config),
            ops,
            iterator: ops.iter().enumerate().skip(0),
        }
    }

    fn done(self) -> Lines<S> {
        self.buffer.done()
    }

    /// Returns `true` if the buffer is finished, and ready to finalize.
    fn tick(&mut self) -> bool {
        let (hir_offset, op) = match self.iterator.next() {
            // If we got to the end of the iteration, we got past the [HIR::EOF] operation and we're
            // ready to return the buffer.
            None => {
                return true;
            }
            Some((op, hir)) => (op, *hir),
        };

        match self.buffer.process(hir_offset, op) {
            ProcessResult::EOF => {
                // Do nothing. The next iteration will be [None] and will be handled above.
            }
            ProcessResult::Next => {
                // Do nothing.
            }
            ProcessResult::Flush(FlushResult::Success(line)) => {
                self.buffer.lines.push(line);
            }
            ProcessResult::Flush(FlushResult::Rewind(rewind)) => {
                printf_debug!("REWND {:?}", rewind);
                self.buffer.rewind(rewind);
                self.iterator = self.ops.iter().enumerate().skip(rewind.hir_offset);
            }
        }

        false
    }
}

pub fn to_lines<S>(config: PrintConfig, ops: &[HIR<S>]) -> Lines<S>
where
    S: Style,
{
    let mut to_lines = ToLines::new(ops, config);

    loop {
        if to_lines.tick() {
            return to_lines.done();
        }
    }
}
