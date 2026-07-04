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
        detail: String,
    },
    Choice {
        chosen_idx: usize,
        options: Vec<String>,
    },
    OptionalAccept,
    OptionalDecline,
    Condition {
        expr: String,
        result: bool,
        negated: bool,
        took_else: bool,
    },
    EndCondition {
        expr: String,
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
    Quantifier {
        kind: String,
        detail: String,
    },
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
            TraceEvent::Action { subtype, detail } => write!(f, "Action:{} {}", subtype, detail),
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
                    "Condition: {} = {} (neg={}, else={})",
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
            TraceEvent::Quantifier { kind, detail } => {
                write!(f, "Quantifier:{} {}", kind, detail)
            }
        }
    }
}
