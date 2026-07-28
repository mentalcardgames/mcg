use front_end::ast::{CardSet, IntRange};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputRequest {
    PickItems {
        cardset: CardSet,
        min: usize,
        max: usize,
        context: String,
    },
    PickCount {
        int_range: IntRange,
        context: String,
    },
    PickPlayer {
        players: Vec<String>,
        min: usize,
        max: usize,
        context: String,
    },
}