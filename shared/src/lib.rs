use serde::{Deserialize, Serialize};
use egui::Vec2;

pub mod communication;

pub const CARD_NATURAL_SIZE: Vec2 = Vec2::new(140.0, 190.0);

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlindKind {
    SmallBlind,
    BigBlind,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Card(pub u8);

impl From<usize> for PlayerId {
    fn from(v: usize) -> Self {
        PlayerId(v)
    }
}
impl From<PlayerId> for usize {
    fn from(pid: PlayerId) -> Self {
        pid.0
    }
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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
    pub player_id: PlayerId,
    pub rank: HandRank,
    pub best_five: [Card; 5],
}

/// LogEntry is a thin wrapper around the canonical ActionEvent. LogEntries
/// may later gain timestamps/ids for persistence, but the event itself should
/// be derived from ActionEvents (ActionEvent -> LogEntry), not the other way.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerPublic {
    pub id: PlayerId,
    pub name: String,
    pub stack: u32,
    pub cards: Option<[Card; 2]>, // only set for the viewer
    pub has_folded: bool,
    pub bet_this_round: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStatePublic {
    pub players: Vec<PlayerPublic>,
    pub community: Vec<Card>,
    pub pot: u32,
    #[serde(default)]
    pub sb: u32,
    #[serde(default)]
    pub bb: u32,
    pub to_act: PlayerId,
    pub stage: Stage,
    #[serde(default)]
    pub winner_ids: Vec<PlayerId>,
    #[serde(default)]
    pub action_log: Vec<ActionEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub id: PlayerId,
    pub name: String,
    pub is_bot: bool, // true if driven by bot mechanisms, false if waits for messages
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMsg {
    /// Player-initiated action: must specify which player is performing the action.
    Action {
        player_id: PlayerId,
        action: PlayerAction,
    },
    RequestState,
    NextHand,
    NewGame {
        players: Vec<PlayerConfig>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMsg {
    /// A simple welcome/acknowledgement sent by the server when a client
    /// first connects. The server no longer assigns or embeds a per-connection
    /// `PlayerId` in this message; transports or session managers may choose
    /// to emit their own client-specific information separately if needed.
    Welcome,
    State(GameStatePublic),
    Error(String),
}
