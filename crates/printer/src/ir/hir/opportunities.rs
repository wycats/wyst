use wyst_core::{wyst_copy, wyst_data};

use crate::ir::hir::BreakOpportunity;

#[wyst_data]
pub struct BreakOpportunities {}

/// A `BreakOpportunityPoint` is a break opportunity in a previous line.
#[wyst_copy]
pub struct BreakOpportunityPoint {
    line_number: usize,
    opportunity: BreakOpportunity,
}
