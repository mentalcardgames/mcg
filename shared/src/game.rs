//! Game state, stages, and action types for the Mental Card Game.

use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::hand::HandResult;
use crate::player::PlayerId;

/// The current stage of a poker hand
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

/// Simple player action types that can be taken during a hand
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerAction {
    Fold,
    CheckCall,
    Bet(u32),
}

/// Player-side action kinds used in logs/history (keeps richer semantics for history)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionKind {
    Fold,
    Check,
    Call(u32),
    Bet(u32),
    Raise { to: u32, by: u32 },
    PostBlind { kind: BlindKind, amount: u32 },
}

/// Game-level actions/events (formerly folded into LogEvent)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameAction {
    StageChanged(Stage),
    DealtHole { player_id: PlayerId },
    DealtCommunity { cards: Vec<Card> },
    Showdown { hand_results: Vec<HandResult> },
    PotAwarded { winners: Vec<PlayerId>, amount: u32 },
}

/// A single recorded action/event in the game. This is now the canonical,
/// typed source-of-truth for both UI and logs. Use ActionEvent::PlayerAction
/// for player-initiated actions and ActionEvent::GameAction for dealer/stage/etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionEvent {
    PlayerAction {
        player_id: PlayerId,
        action: ActionKind,
    },
    GameAction(GameAction),
}

impl ActionEvent {
    /// Helper to create a PlayerAction event from a player id + ActionKind
    pub fn player(player_id: PlayerId, action: ActionKind) -> Self {
        ActionEvent::PlayerAction { player_id, action }
    }

    /// Helper to create a GameAction event
    pub fn game(action: GameAction) -> Self {
        ActionEvent::GameAction(action)
    }
}

/// Types of blinds that can be posted
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlindKind {
    SmallBlind,
    BigBlind,
}