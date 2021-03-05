#[macro_use]
mod dev;

mod algorithm;
mod builder;
mod fragment;
pub mod ir;
mod printer;
mod range;
mod text;
mod texts;

#[cfg(test)]
mod tests;

pub use self::builder::FragmentBuilder;
pub use self::ir::{to_lines, HirBuilder, PrintConfig, Printer};
pub use self::range::{RangeBound, ResolvedRange, WystRange};
