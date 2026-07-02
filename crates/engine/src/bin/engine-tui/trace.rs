//! TraceEntry relay types for the TUI

use cgdsl_engine::TraceEntry;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraceDetail {
    Choices,
    Evaluations,
    Verbose,
}

impl TraceDetail {
    pub fn cycle(&mut self) {
        *self = match self {
            TraceDetail::Choices => TraceDetail::Evaluations,
            TraceDetail::Evaluations => TraceDetail::Verbose,
            TraceDetail::Verbose => TraceDetail::Choices,
        };
    }

    pub fn passes(&self, entry: &TraceEntry) -> bool {
        match self {
            TraceDetail::Choices => {
                matches!(entry, TraceEntry::ChoiceMade { .. } | TraceEntry::OptionalAccepted | TraceEntry::OptionalDeclined)
            }
            TraceDetail::Evaluations => {
                !matches!(entry, TraceEntry::Step { .. })
            }
            TraceDetail::Verbose => true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisplayTraceEntry {
    pub step_number: usize,
    pub entry: TraceEntry,
}

impl DisplayTraceEntry {
    pub fn from_trace_entry(step: usize, entry: TraceEntry) -> Self {
        Self { step_number: step, entry }
    }
}