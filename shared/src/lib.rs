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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMsg {
    Join { name: String },
    Action(PlayerAction),
    RequestState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMsg {
    Welcome { you: usize },
    State(GameStatePublic),
    Error(String),
}
