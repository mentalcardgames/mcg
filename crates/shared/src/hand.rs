//! Hand evaluation types for poker hands in the Mental Card Game.

use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::player::PlayerId;

/// Categories of poker hands, ordered from weakest to strongest
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRankCategory {
    HighCard,
    Pair,
    TwoPair,
    ThreeKind,
    Straight,
    Flush,
    FullHouse,
    FourKind,
    StraightFlush,
}

impl HandRankCategory {
    pub fn to_str(&self) -> &'static str {
        match self {
            HandRankCategory::HighCard => "High Card",
            HandRankCategory::Pair => "Pair",
            HandRankCategory::TwoPair => "Two Pair",
            HandRankCategory::ThreeKind => "Three of a Kind",
            HandRankCategory::Straight => "Straight",
            HandRankCategory::Flush => "Flush",
            HandRankCategory::FullHouse => "Full House",
            HandRankCategory::FourKind => "Four of a Kind",
            HandRankCategory::StraightFlush => "Straight Flush",
        }
    }
}

/// Complete hand ranking including category and tiebreakers
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandRank {
    pub category: HandRankCategory,
    pub tiebreakers: Vec<u8>,
}

/// Result of hand evaluation for a player at showdown
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandResult {
    pub player_id: PlayerId,
    pub rank: HandRank,
    pub best_five: [Card; 5],
}
