//! Player-related types and identifiers for the Mental Card Game.

use serde::{Deserialize, Serialize};

use crate::cards::Card;

/// Unique identifier for a player in the game
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

impl From<usize> for PlayerId {
    fn from(v: usize) -> Self {
        PlayerId(v)
    }
}

impl From<PlayerId> for usize {
    fn from(player_id: PlayerId) -> Self {
        player_id.0
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Public view of a player's state (what other players can see)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerPublic {
    pub id: PlayerId,
    pub name: String,
    pub stack: u32,
    pub cards: Option<[Card; 2]>, // only set for the viewer
    pub has_folded: bool,
    pub bet_this_round: u32,
}

/// Configuration for setting up a player in a new game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub id: PlayerId,
    pub name: String,
    pub is_bot: bool, // true if driven by bot mechanisms, false if waits for messages
}
