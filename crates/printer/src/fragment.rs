// use wyst_core::wyst_data;

// use crate::{
//     ir::{Instruction, InstructionSlice, Instructions},
//     line::{Lines, Token, Tokens},
//     printer::{Nesting, PrinterConfig},
//     style::Style,
// };

// #[wyst_data]
// #[derive(Default)]
// struct MachineState {
//     break_level: usize,
//     cursor: usize,
//     nesting: Nesting,
// }

// pub struct Machine<S>
// where
//     S: Style,
// {
//     config: PrinterConfig,
//     state: MachineState,
//     lines: Lines<S>,
//     current_line: Tokens<S>,
// }

// impl<S> Machine<S>
// where
//     S: Style,
// {
//     // FIXME: config should have indentation string in it
//     pub fn new(config: impl Into<PrinterConfig>) -> Machine<S> {
//         Machine {
//             config: config.into(),
//             state: MachineState::default(),
//             lines: Lines::default(),
//             current_line: Tokens::build(Nesting::default()),
//         }
//     }

//     pub fn process(mut self, instructions: &Instructions<S>) -> Lines<S> {
//         process(&mut self, instructions);

//         let Machine {
//             mut lines,
//             current_line,
//             ..
//         } = self;

//         if current_line.has_content() {
//             lines.add(current_line.done());
//         }

//         lines
//     }

//     fn add(&mut self, text: Token<S>) {
//         self.current_line.add(text);
//     }

//     fn newline(&mut self) {
//         let mut line = Tokens::<S>::new(self.state.nesting, vec![]);
//         std::mem::swap(&mut self.current_line, &mut line);

//         self.lines.add(line.done());
//     }

//     fn line_width(&self) -> usize {
//         self.config.available(self.state.nesting)
//     }
// }

// // FIXME: normalize InstructionSlice into a chunk of instructions that doesn't have Start/Stop and only has unconditional Breaks
// fn compute_break_level(
//     instructions: &InstructionSlice<'_, impl Style>,
//     available_width: usize,
// ) -> usize {
//     let mut break_level = 0;

//     // This isn't an infinite loop because can_lay_out will eventually return true when no
//     // higher-level breaks are available.
//     loop {
//         if can_lay_out(instructions, break_level, available_width) {
//             return break_level;
//         }

//         break_level += 1;
//     }
// }

// fn can_lay_out<S>(
//     instructions: &InstructionSlice<'_, S>,
//     break_level: usize,
//     available_width: usize,
// ) -> bool
// where
//     S: Style,
// {
//     let mut started = 0;
//     let mut available = available_width;
//     let mut max_level = 0;

//     for instruction in instructions.iter() {
//         match instruction {
//             Instruction::Start { .. } => started += 1,
//             Instruction::End => {
//                 started -= 1;
//                 if started == 0 {
//                     // we made it all the way to the end of the current block of content
//                     // without overflowing, so layout will work.
//                     return true;
//                 }
//             }
//             Instruction::Bounded(bounded) => {
//                 if bounded.len() <= available {
//                     available -= bounded.len()
//                 } else if max_level > break_level {
//                     // If the text doesn't fit in the available space and there is a
//                     // previous higher-level break we could take, then we should take the
//                     // higher-level break.
//                     return false;
//                 } else {
//                     return true;
//                 }
//             }
//             Instruction::Interior(interior) => {
//                 if available == available_width {
//                     // If we're at the front the line, ignore it
//                     continue;
//                 }

//                 if interior.len() <= available {
//                     available -= interior.len()
//                 } else if max_level > break_level {
//                     // If the text doesn't fit in the available space and there is a
//                     // previous higher-level break we could take, then we should take the
//                     // higher-level break.
//                     return false;
//                 } else {
//                     return true;
//                 }
//             }
//             Instruction::BreakOpportunity { level } => {
//                 if *level > break_level {
//                     // Remember the max level that we've seen.
//                     max_level = max_level.max(*level);
//                 } else {
//                     // newline
//                     available = available_width;
//                 }
//             }
//         }
//     }

//     true
// }

// fn process<S>(machine: &mut Machine<S>, instructions: &Instructions<S>)
// where
//     S: Style,
// {
//     for slice in instructions.slices() {
//         let break_level = compute_break_level(&slice, machine.line_width());

//         for instruction in slice.iter() {
//             match instruction {
//                 Instruction::Bounded(text) => machine.add(Token::Anywhere(*text)),
//                 Instruction::Interior(interior) => machine.add(Token::Interior(*interior)),
//                 Instruction::BreakOpportunity { level } => {
//                     if *level <= break_level {
//                         machine.newline()
//                     }
//                 }
//                 _ => {}
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::text::Printable;
//     use crate::texts::Texts;
//     use crate::{ir::InstructionBuilder, style::PlainStyle};

//     struct Program<S>
//     where
//         S: Style,
//     {
//         instructions: Instructions<S>,
//         texts: Texts,
//     }

//     impl<S> Program<S>
//     where
//         S: Style,
//     {
//         fn new(
//             build: impl for<'texts> FnOnce(
//                 InstructionBuilder<'texts, S>,
//             ) -> InstructionBuilder<'texts, S>,
//         ) -> Program<S> {
//             let mut texts = Texts::default();
//             let builder = InstructionBuilder::new(&mut texts);
//             let instructions = build(builder).done();

//             Program {
//                 instructions,
//                 texts,
//             }
//         }

//         fn run(&self, config: impl Into<PrinterConfig>) -> String {
//             let machine = Machine::<S>::new(config);
//             let lines = machine.process(&self.instructions);
//             format!("{}", lines.format(&self.texts))
//         }
//     }

//     #[test]
//     fn test_bounded() {
//         let program = Program::new(|b: InstructionBuilder<PlainStyle>| b.bounded("hello"));

//         assert_eq!(program.run(80), "hello\n");
//     }

//     #[test]
//     fn test_bounded_twice() {
//         let program = Program::<PlainStyle>::new(|b| b.bounded("hello").bounded("goodbye"));

//         assert_eq!(program.run(80), "hellogoodbye\n");
//     }

//     #[test]
//     fn test_allowed_break() {
//         let program = Program::<PlainStyle>::new(|b| {
//             b.start(1).bounded("hello").wbr(1).bounded("goodbye").end()
//         });

//         assert_eq!(program.run(80), "hellogoodbye\n");
//         assert_eq!(program.run(7), "hello\ngoodbye\n");
//     }

//     #[test]
//     fn test_different_layer_break() {
//         let program = Program::<PlainStyle>::new(|b| {
//             b.start(1)
//                 .bounded("hello")
//                 .bounded("(")
//                 .wbr(1)
//                 .bounded("this")
//                 .wbr(2)
//                 .bounded("is")
//                 .wbr(2)
//                 .bounded("inside")
//                 .wbr(1)
//                 .bounded(")")
//                 .end()
//         });

//         assert_eq!(program.run(80), "hello(thisisinside)\n");
//         assert_eq!(program.run(12), "hello(\nthisisinside\n)\n");
//         assert_eq!(program.run(7), "hello(\nthis\nis\ninside\n)\n");
//     }

//     #[test]
//     fn test_interior_content() {
//         let program = Program::<PlainStyle>::new(|b| {
//             b.start(1)
//                 .bounded("hello")
//                 .bounded("(")
//                 .wbr(1)
//                 .bounded("this")
//                 .wbr(2)
//                 .interior(" ", Style::invisible())
//                 .bounded("is")
//                 .wbr(2)
//                 .interior(" ", Style::invisible())
//                 .bounded("inside")
//                 .wbr(1)
//                 .bounded(")")
//                 .end()
//         });

//         assert_eq!(program.run(80), "hello(this is inside)\n");
//         assert_eq!(program.run(14), "hello(\nthis is inside\n)\n");
//         assert_eq!(program.run(7), "hello(\nthis\nis\ninside\n)\n");
//     }

//     #[test]
//     fn test_atomic_subcontent() {
//         let program = Program::<PlainStyle>::new(|b| {
//             // FIXME: handle nesting

//             b.start(1)
//                 .start(2)
//                 .bounded("hello")
//                 .wbr(2)
//                 .interior(" ", Style::invisible())
//                 .bounded("world")
//                 .end()
//                 .wbr(1)
//                 .interior(" ", Style::invisible())
//                 .start(2)
//                 .bounded("hellooooo")
//                 .wbr(2)
//                 .interior(" ", Style::invisible())
//                 .bounded("world")
//                 .end()
//         });

//         // assert_eq!(program.run(80), "hello world hellooooo world\n");
//         assert_eq!(program.run(11), "hello world\nhellooooo\nworld\n");
//     }
// }
