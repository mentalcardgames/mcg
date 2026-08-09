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
    Action {
        subtype: String,
        /// Human-readable DSL text, e.g. `deal 1 from Deck private to Hand of P:P1`.
        detail: String,
        /// `Debug` representation of the rule, e.g. `Move { move_type: ... }`.
        raw_detail: String,
    },
    Choice {
        chosen_idx: usize,
        options: Vec<String>,
    },
    OptionalAccept,
    OptionalDecline,
    Condition {
        /// Human-readable boolean expression, e.g. `(sum of Hand of current using BJ > 21)`.
        expr: String,
        /// `Debug` representation of the expression.
        raw_expr: String,
        result: bool,
        negated: bool,
        took_else: bool,
    },
    EndCondition {
        /// Human-readable boolean expression.
        expr: String,
        /// `Debug` representation of the expression.
        raw_expr: String,
        stage: String,
        result: bool,
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
    /// (action rules, condition expressions); identical to [`pretty`]
    /// otherwise.
    pub fn raw(&self) -> String {
        match self {
            TraceEvent::Action {
                subtype,
                raw_detail,
                ..
            } => format!("Action:{} {}", subtype, raw_detail),
            TraceEvent::Condition {
                raw_expr,
                result,
                negated,
                took_else,
                ..
            } => {
                format!(
                    "Condition: {} = {} (neg={}, else={})",
                    raw_expr, result, negated, took_else
                )
            }
            TraceEvent::EndCondition {
                raw_expr,
                stage,
                result,
                exited,
                ..
            } => {
                format!(
                    "EndCondition({}): {} = {} (exited={})",
                    stage, raw_expr, result, exited
                )
            }
            _ => format!("{}", self),
        }
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
            TraceEvent::Action {
                subtype, detail, ..
            } => write!(f, "{} {}", subtype, detail),
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
                ..
            } => {
                write!(
                    f,
                    "Condition: {} = {} (neg={}, else={})",
                    expr, result, negated, took_else
                )
            }
            TraceEvent::EndCondition {
                expr,
                result,
                stage,
                exited,
                ..
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
            TraceEvent::Quantifier { kind, detail } => {
                write!(f, "Quantifier:{} {}", kind, detail)
            }
        }
    }
}

#[cfg(test)]
#[path = "trace_tests.rs"]
mod tests;
