use serde::{Deserialize, Serialize};

pub mod communication;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

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
    DealtHole { player_id: usize },
    DealtCommunity { cards: Vec<u8> },
    Showdown { hand_results: Vec<HandResult> },
    PotAwarded { winners: Vec<usize>, amount: u32 },
}

/// A single recorded action/event in the game. This is now the canonical,
/// typed source-of-truth for both UI and logs. Use ActionEvent::PlayerAction
/// for player-initiated actions and ActionEvent::GameAction for dealer/stage/etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionEvent {
    PlayerAction {
        player_id: usize,
        action: ActionKind,
    },
    GameAction(GameAction),
}

impl ActionEvent {
    /// Helper to create a PlayerAction event from a player id + ActionKind
    pub fn player(player_id: usize, action: ActionKind) -> Self {
        ActionEvent::PlayerAction { player_id, action }
    }

    /// Helper to create a GameAction event
    pub fn game(action: GameAction) -> Self {
        ActionEvent::GameAction(action)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlindKind {
    SmallBlind,
    BigBlind,
}

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandRank {
    pub category: HandRankCategory,
    pub tiebreakers: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandResult {
    pub player_id: usize,
    pub rank: HandRank,
    pub best_five: [u8; 5],
}

/// LogEntry is a thin wrapper around the canonical ActionEvent. LogEntries
/// may later gain timestamps/ids for persistence, but the event itself should
/// be derived from ActionEvents (ActionEvent -> LogEntry), not the other way.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerPublic {
    pub id: usize,
    pub name: String,
    pub stack: u32,
    pub cards: Option<[u8; 2]>, // only set for the viewer
    pub has_folded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStatePublic {
    pub players: Vec<PlayerPublic>,
    pub community: Vec<u8>,
    pub pot: u32,
    #[serde(default)]
    pub sb: u32,
    #[serde(default)]
    pub bb: u32,
    pub to_act: usize,
    pub stage: Stage,
    pub you_id: usize,
    #[serde(default)]
    pub winner_ids: Vec<usize>,
    #[serde(default)]
    pub action_log: Vec<ActionEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub id: usize,
    pub name: String,
    pub is_bot: bool, // true if driven by bot mechanisms, false if waits for messages
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMsg {
    /// Player-initiated action: must specify which player is performing the action.
    Action {
        player_id: usize,
        action: PlayerAction,
    },
    /// Request the current state for a particular player (you_id in State will reflect this).
    RequestState {
        player_id: usize,
    },
    /// Request to advance to the next hand on behalf of a specific player.
    NextHand {
        player_id: usize,
    },
    /// Start a new game with the specified players. Each player has an ID, name, and bot flag.
    NewGame {
        players: Vec<PlayerConfig>,
    },

}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMsg {
    Welcome { you: usize },
    State(GameStatePublic),
    Error(String),
}
