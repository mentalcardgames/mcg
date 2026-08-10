//! Trace entry types for per-FSM-transition telemetry.
//!
//! Events carry **typed payloads** — the actual AST nodes — rather than
//! pre-rendered strings: a host can inspect `rule`/`expr` directly and render
//! whatever view it needs. The `Display`, [`TraceEvent::raw`] and
//! [`TraceEvent::summary`] impls are *derived views* over that data; the
//! trace-file lines are produced by `Display`, so the file format is stable.

use front_end::ast::{BoolExpr, EndCondition, GameRule};

use super::ir_ext::rule_signature;

#[derive(Clone, Debug)]
pub enum TraceEntry {
    Step {
        from: u32,
        to: u32,
        event: TraceEvent,
    },
}

#[derive(Clone, Debug)]
pub enum TraceEvent {
    /// The full action rule being executed. Render it with [`TraceEvent::pretty`]
    /// (DSL text), [`TraceEvent::raw`] (`Debug`), or [`TraceEvent::summary`]
    /// (compact structured form).
    Action {
        rule: GameRule,
    },
    Choice {
        chosen_idx: usize,
        options: Vec<String>,
    },
    OptionalAccept,
    OptionalDecline,
    /// The evaluated boolean expression — the full node, not pre-rendered text.
    Condition {
        expr: BoolExpr,
        result: bool,
        negated: bool,
        took_else: bool,
    },
    EndCondition {
        expr: EndCondition,
        result: bool,
        stage: String,
        exited: bool,
    },
    StageRoundCounter {
        stage: String,
        new_count: u32,
    },
    EndStage {
        stage: String,
    },
    Trigger,
    /// The current player is out of the game / out of the current stage:
    /// the instruction edge was advanced through without executing
    /// (2026-08-10, ineligible-player skip).
    Skipped {
        player: String,
        stage: String,
    },
    /// Free-form quantifier diagnostics ("5 players, awaiting card choice").
    Quantifier {
        kind: String,
        detail: String,
    },
}

impl TraceEvent {
    /// "Simplified" rendering: DSL-level text. `TraceEntry::Display` uses
    /// this form (so the `MCG_TRACE_LOG` file is readable too).
    pub fn pretty(&self) -> String {
        format!("{}", self)
    }

    /// "Raw" rendering: `Debug` representations where the engine has them
    /// (action rules, condition expressions); identical to [`pretty`] otherwise.
    pub fn raw(&self) -> String {
        match self {
            TraceEvent::Action { rule } => {
                let (subtype, _, raw_detail) = rule_signature(rule);
                format!("{} {}", subtype, raw_detail)
            }
            TraceEvent::Condition {
                expr,
                result,
                negated,
                took_else,
            } => {
                format!(
                    "Condition: {:?} = {} (neg={}, else={})",
                    expr, result, negated, took_else
                )
            }
            TraceEvent::EndCondition {
                expr,
                stage,
                result,
                exited,
            } => {
                format!(
                    "EndCondition({}): {:?} = {} (exited={})",
                    stage, expr, result, exited
                )
            }
            _ => format!("{}", self),
        }
    }

    /// Compact structured rendering for hosts that want the *semantic*
    /// content (which cards moved where, which memory was written) instead of
    /// DSL text. Falls back to [`pretty`] for events without a structured form.
    pub fn summary(&self) -> String {
        use front_end::ast::{ActionRule, GameRule};
        match self {
            TraceEvent::Action { rule } => match rule {
                GameRule::Action {
                    action: ActionRule::Move { move_type },
                } => move_summary(move_type),
                GameRule::Action {
                    action:
                        ActionRule::SetMemory {
                            memory,
                            memory_type,
                        },
                } => format!("set {memory} := {memory_type}"),
                GameRule::Action {
                    action: ActionRule::ResetMemory { memory },
                } => format!("reset {memory}"),
                GameRule::Action {
                    action: ActionRule::CycleAction { player },
                } => format!("cycle to {player}"),
                GameRule::Action {
                    action: ActionRule::OutAction { players, out_of },
                } => format!("out {players} of {out_of}"),
                _ => format!("{}", rule),
            },
            _ => format!("{}", self),
        }
    }
}

/// Structured one-liner for a Move action: which cards move where.
fn move_summary(move_type: &front_end::ast::MoveType) -> String {
    use front_end::ast::{ClassicMove, DealMove, ExchangeMove, MoveCardSet, MoveType};
    let mcs = match move_type {
        MoveType::Deal { deal } => match deal {
            DealMove::MoveCardSet { deal_cs } => Some(deal_cs),
        },
        MoveType::Exchange { exchange } => match exchange {
            ExchangeMove::MoveCardSet { exchange_cs } => Some(exchange_cs),
        },
        MoveType::Classic { classic } => match classic {
            ClassicMove::MoveCardSet { move_cs } => Some(move_cs),
        },
        MoveType::Place { .. } => None,
    };
    match mcs {
        Some(MoveCardSet::Move { from, status, to }) => {
            format!("move {} -> {} ({:?})", from, to, status)
        }
        Some(MoveCardSet::MoveQuantity {
            quantity,
            from,
            status,
            to,
        }) => format!("move {} {} -> {} ({:?})", quantity, from, to, status),
        None => format!("{}", move_type),
    }
}

impl std::fmt::Display for TraceEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceEntry::Step { from, to, event } => write!(f, "[{from}->{to}] {event}"),
        }
    }
}

impl std::fmt::Display for TraceEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceEvent::Action { rule } => {
                let (subtype, detail, _) = rule_signature(rule);
                write!(f, "{} {}", subtype, detail)
            }
            TraceEvent::Choice {
                chosen_idx,
                options,
            } => {
                write!(f, "Choice: chose {} (from {:?})", chosen_idx + 1, options)
            }
            TraceEvent::OptionalAccept => write!(f, "Optional: ACCEPTED"),
            TraceEvent::OptionalDecline => write!(f, "Optional: DECLINED"),
            TraceEvent::Condition {
                expr,
                result,
                negated,
                took_else,
            } => {
                write!(
                    f,
                    "Condition: {} = {} (neg={}, body={})",
                    expr, result, negated, took_else
                )
            }
            TraceEvent::EndCondition {
                expr,
                result,
                stage,
                exited,
            } => {
                write!(
                    f,
                    "EndCondition({}): {} = {} (exited={})",
                    stage, expr, result, exited
                )
            }
            TraceEvent::StageRoundCounter { stage, new_count } => {
                write!(f, "StageRoundCounter: {} -> {}", stage, new_count)
            }
            TraceEvent::EndStage { stage } => write!(f, "EndStage: {}", stage),
            TraceEvent::Trigger => write!(f, "Trigger"),
            TraceEvent::Skipped { player, stage } => {
                write!(f, "Skipped: {player} (out of {stage})")
            }
            TraceEvent::Quantifier { kind, detail } => {
                write!(f, "Quantifier:{} {}", kind, detail)
            }
        }
    }
}

#[cfg(test)]
#[path = "trace_tests.rs"]
mod tests;
