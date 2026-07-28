use front_end::ast::{self, GameRule};
use serde::{Deserialize, Serialize};

use crate::input_request::InputRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnginePayload {
    Condition {
        expr: ast::BoolExpr,
        negated: bool,
    },
    EndCondition {
        expr: ast::EndCondition,
        negated: bool,
        stage: String,
    },
    Action(GameRule),
    NeedsInput {
        request: InputRequest,
        original_action: GameRule,
    },
    StageRoundCounter(String),
    EndStage(String),
    Choice,
    Optional,
    Trigger,
}

impl EnginePayload {
    pub fn to_string(&self) -> String {
        match self {
            EnginePayload::Condition { expr: _, negated } => {
                if *negated {
                    String::from("Not Condition")
                } else {
                    String::from("Condition")
                }
            }
            EnginePayload::EndCondition {
                expr: _,
                negated,
                stage,
            } => {
                let stage_name = format!("{:?}", stage);
                if *negated {
                    format!("Not EndCondition ({})", stage_name)
                } else {
                    format!("EndCondition ({})", stage_name)
                }
            }
            EnginePayload::Action(_) => String::from("Action"),
            EnginePayload::NeedsInput { .. } => String::from("NeedsInput"),
            EnginePayload::StageRoundCounter(_) => format!("Stage Round Counter"),
            EnginePayload::EndStage(_) => format!("End Counter"),
            EnginePayload::Choice => String::from("Choice"),
            EnginePayload::Optional => String::from("Optional"),
            EnginePayload::Trigger => String::from("Trigger"),
        }
    }
}

impl From<front_end::ir::Payload<front_end::ir::LoweredCtx>> for EnginePayload {
    fn from(payload: front_end::ir::Payload<front_end::ir::LoweredCtx>) -> Self {
        use front_end::ir::Payload;
        match payload {
            Payload::Condition { expr, negated } => EnginePayload::Condition { expr, negated },
            Payload::EndCondition {
                expr,
                negated,
                stage,
            } => EnginePayload::EndCondition {
                expr,
                negated,
                stage,
            },
            Payload::Action(a) => EnginePayload::Action(a),
            Payload::StageRoundCounter(s) => EnginePayload::StageRoundCounter(s),
            Payload::EndStage(s) => EnginePayload::EndStage(s),
            Payload::Choice => EnginePayload::Choice,
            Payload::Optional => EnginePayload::Optional,
            Payload::Trigger => EnginePayload::Trigger,
        }
    }
}