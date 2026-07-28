//! TraceEntry relay types for the TUI

use cgdsl_engine::TraceEntry;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraceDetail {
    Choices,
    Evaluations,
    Verbose,
    Last5,
}

impl TraceDetail {
    pub fn cycle(&mut self) {
        *self = match self {
            TraceDetail::Choices => TraceDetail::Evaluations,
            TraceDetail::Evaluations => TraceDetail::Verbose,
            TraceDetail::Verbose => TraceDetail::Last5,
            TraceDetail::Last5 => TraceDetail::Choices,
        };
    }

    pub fn passes(&self, entry: &TraceEntry) -> bool {
        match self {
            TraceDetail::Choices => {
                matches!(
                    entry,
                    TraceEntry::Step {
                        event: cgdsl_engine::TraceEvent::Choice { .. },
                        ..
                    }
                )
            }
            TraceDetail::Evaluations => {
                matches!(
                    entry,
                    TraceEntry::Step {
                        event: cgdsl_engine::TraceEvent::OptionalAccept,
                        ..
                    }
                ) || matches!(
                    entry,
                    TraceEntry::Step {
                        event: cgdsl_engine::TraceEvent::OptionalDecline,
                        ..
                    }
                ) || matches!(
                    entry,
                    TraceEntry::Step {
                        event: cgdsl_engine::TraceEvent::Condition { .. },
                        ..
                    }
                ) || matches!(
                    entry,
                    TraceEntry::Step {
                        event: cgdsl_engine::TraceEvent::EndCondition { .. },
                        ..
                    }
                )
            }
            TraceDetail::Verbose => true,
            TraceDetail::Last5 => true,
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
        Self {
            step_number: step,
            entry,
        }
    }
}
