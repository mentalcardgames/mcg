use front_end::ir::{Ir, LoweredPayLoad, Payload, StateID};

pub trait IrExt {
    fn edge_labels(&self, state: StateID) -> Vec<String>;
}

impl IrExt for Ir<LoweredPayLoad> {
    fn edge_labels(&self, state: StateID) -> Vec<String> {
        let edges = match self.states.get(&state) {
            Some(e) => e,
            None => return Vec::new(),
        };
        edges
            .iter()
            .enumerate()
            .map(|(i, edge)| {
                let label = self
                    .states
                    .get(&edge.to)
                    .and_then(|target_edges| target_edges.first())
                    .map(|first_edge| payload_label(&first_edge.payload));
                label.unwrap_or_else(|| format!("Option {}", i + 1))
            })
            .collect()
    }
}

/// `(subtype, detail, raw_detail)`: a stable category label ("Action:Move"),
/// the rule rendered as DSL text (`deal 1 from Deck private to Hand of P:P1`),
/// and the `Debug` representation (for the TUI's raw trace mode).
pub(super) fn rule_signature(rule: &front_end::ast::GameRule) -> (String, String, String) {
    match rule {
        front_end::ast::GameRule::Action { action } => {
            let subtype = match action {
                front_end::ast::ActionRule::FlipAction { .. } => "Action:FlipAction".to_string(),
                front_end::ast::ActionRule::ShuffleAction { .. } => {
                    "Action:ShuffleAction".to_string()
                }
                front_end::ast::ActionRule::OutAction { .. } => "Action:OutAction".to_string(),
                front_end::ast::ActionRule::SetMemory { .. } => "Action:SetMemory".to_string(),
                front_end::ast::ActionRule::ResetMemory { .. } => "Action:ResetMemory".to_string(),
                front_end::ast::ActionRule::CycleAction { .. } => "Action:CycleAction".to_string(),
                front_end::ast::ActionRule::BidAction { .. } => "Action:BidAction".to_string(),
                front_end::ast::ActionRule::BidMemoryAction { .. } => {
                    "Action:BidMemoryAction".to_string()
                }
                front_end::ast::ActionRule::EndAction { .. } => "Action:EndAction".to_string(),
                front_end::ast::ActionRule::DemandAction { .. } => {
                    "Action:DemandAction".to_string()
                }
                front_end::ast::ActionRule::DemandMemoryAction { .. } => {
                    "Action:DemandMemoryAction".to_string()
                }
                front_end::ast::ActionRule::Move { .. } => "Action:Move".to_string(),
            };
            (subtype, format!("{}", action), format!("{:?}", action))
        }
        front_end::ast::GameRule::SetUp { setup } => {
            let subtype = match setup {
                front_end::ast::SetUpRule::CreatePlayer { .. } => "SetUp:CreatePlayer".to_string(),
                front_end::ast::SetUpRule::CreateTeams { .. } => "SetUp:CreateTeams".to_string(),
                front_end::ast::SetUpRule::CreateTurnorder { .. } => {
                    "SetUp:CreateTurnorder".to_string()
                }
                front_end::ast::SetUpRule::CreateTurnorderRandom { .. } => {
                    "SetUp:CreateTurnorderRandom".to_string()
                }
                front_end::ast::SetUpRule::CreateLocation { .. } => {
                    "SetUp:CreateLocation".to_string()
                }
                front_end::ast::SetUpRule::CreateCardOnLocation { .. } => {
                    "SetUp:CreateCardOnLocation".to_string()
                }
                front_end::ast::SetUpRule::CreateTokenOnLocation { .. } => {
                    "SetUp:CreateTokenOnLocation".to_string()
                }
                front_end::ast::SetUpRule::CreateCombo { .. } => "SetUp:CreateCombo".to_string(),
                front_end::ast::SetUpRule::CreateMemory { .. } => "SetUp:CreateMemory".to_string(),
                front_end::ast::SetUpRule::CreateMemoryWithMemoryType { .. } => {
                    "SetUp:CreateMemoryWithMemoryType".to_string()
                }
                front_end::ast::SetUpRule::CreatePrecedence { .. } => {
                    "SetUp:CreatePrecedence".to_string()
                }
                front_end::ast::SetUpRule::CreatePointMap { .. } => {
                    "SetUp:CreatePointMap".to_string()
                }
            };
            (subtype, format!("{}", setup), format!("{:?}", setup))
        }
        front_end::ast::GameRule::Scoring { scoring } => {
            let subtype = match scoring {
                front_end::ast::ScoringRule::ScoreRule { score_rule } => match score_rule {
                    front_end::ast::ScoreRule::Score { .. } => "Scoring:Score".to_string(),
                    front_end::ast::ScoreRule::ScoreMemory { .. } => {
                        "Scoring:ScoreMemory".to_string()
                    }
                },
                front_end::ast::ScoringRule::WinnerRule { winner_rule } => match winner_rule {
                    front_end::ast::WinnerRule::Winner { .. } => "Scoring:Winner".to_string(),
                    front_end::ast::WinnerRule::WinnerWith { .. } => {
                        "Scoring:WinnerWith".to_string()
                    }
                },
            };
            (subtype, format!("{}", scoring), format!("{:?}", scoring))
        }
    }
}

pub(super) fn payload_label(payload: &LoweredPayLoad) -> String {
    match payload {
        Payload::Action(gr) => format!("{}", gr),
        Payload::Condition { expr, negated } => {
            if *negated {
                format!("unless {}", expr)
            } else {
                format!("if {}", expr)
            }
        }
        Payload::EndCondition {
            expr,
            negated,
            stage,
        } => {
            if *negated {
                format!("end stage {} unless {}", stage, expr)
            } else {
                format!("end stage {} when {}", stage, expr)
            }
        }
        Payload::StageRoundCounter(stage) => format!("round of {}", stage),
        Payload::EndStage(stage) => format!("end stage {}", stage),
        Payload::Choice => "choose".to_string(),
        Payload::Optional => "optional".to_string(),
        Payload::Trigger => "trigger".to_string(),
    }
}

#[cfg(test)]
#[path = "ir_ext_tests.rs"]
mod tests;
