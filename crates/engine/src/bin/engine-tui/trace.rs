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
        // The GameOver winner-set line is the game result — always visible,
        // whatever the detail filter (2026-08-10).
        let is_game_over = matches!(
            entry,
            TraceEntry::Step {
                event: cgdsl_engine::TraceEvent::GameOver { .. },
                ..
            }
        );
        if is_game_over {
            return true;
        }
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
#[allow(clippy::large_enum_variant)] // Entry carries the full typed TraceEvent (AST nodes); the TUI stores these per-run
pub enum DisplayTraceEntry {
    Entry {
        step_number: usize,
        entry: TraceEntry,
    },
    TurnSeparator {
        player_name: String,
    },
}
