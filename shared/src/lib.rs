use egui::Vec2;
use serde::{Deserialize, Serialize};

pub mod communication;

pub const CARD_NATURAL_SIZE: Vec2 = Vec2::new(140.0, 190.0);

/// Card rank values (0=Ace, 1=2, ..., 12=King)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardRank {
    Ace = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
    Eight = 7,
    Nine = 8,
    Ten = 9,
    Jack = 10,
    Queen = 11,
    King = 12,
}

impl CardRank {
    /// Convert from u8 to CardRank. Panics if value > 12.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CardRank::Ace,
            1 => CardRank::Two,
            2 => CardRank::Three,
            3 => CardRank::Four,
            4 => CardRank::Five,
            5 => CardRank::Six,
            6 => CardRank::Seven,
            7 => CardRank::Eight,
            8 => CardRank::Nine,
            9 => CardRank::Ten,
            10 => CardRank::Jack,
            11 => CardRank::Queen,
            12 => CardRank::King,
            _ => panic!("Invalid card rank: {}", value),
        }
    }

    /// Convert to usize for array indexing.
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

/// Card suit values (0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardSuit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

impl CardSuit {
    /// Convert from u8 to CardSuit. Panics if value > 3.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CardSuit::Clubs,
            1 => CardSuit::Diamonds,
            2 => CardSuit::Hearts,
            3 => CardSuit::Spades,
            _ => panic!("Invalid card suit: {}", value),
        }
    }

    /// Convert to usize for array indexing.
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

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

impl Card {
    /// Create a new card from rank and suit
    pub fn new(rank: CardRank, suit: CardSuit) -> Self {
        Card((suit as u8) * 13 + (rank as u8))
    }

    /// Get the rank of this card
    pub fn rank(self) -> CardRank {
        CardRank::from_u8(self.0 % 13)
    }

    /// Get the suit of this card
    pub fn suit(self) -> CardSuit {
        CardSuit::from_u8(self.0 / 13)
    }
}

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
