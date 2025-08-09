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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionEvent {
    pub player_id: usize,
    pub action: PlayerAction,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlindKind {
    SmallBlind,
    BigBlind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionKind {
    Fold,
    Check,
    Call(u32),
    Bet(u32),
    Raise { to: u32, by: u32 },
    PostBlind { kind: BlindKind, amount: u32 },
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogEvent {
    Action(ActionKind),
    StageChanged(Stage),
    DealtHole { player_id: usize },
    DealtCommunity { cards: Vec<u8> },
    Showdown { hand_results: Vec<HandResult> },
    PotAwarded { winners: Vec<usize>, amount: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub player_id: Option<usize>,
    pub event: LogEvent,
}

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
    pub to_act: usize,
    pub stage: Stage,
    pub you_id: usize,
    #[serde(default)]
    pub bot_count: usize,
    #[serde(default)]
    pub recent_actions: Vec<ActionEvent>,
    #[serde(default)]
    pub winner_ids: Vec<usize>,
    #[serde(default)]
    pub action_log: Vec<LogEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMsg {
    Join { name: String },
    Action(PlayerAction),
    RequestState,
    NextHand,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMsg {
    Welcome { you: usize },
    State(GameStatePublic),
    Error(String),
}
