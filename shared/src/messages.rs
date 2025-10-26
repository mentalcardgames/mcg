//! Client-server messaging protocol for the Mental Card Game.

use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::game::PlayerAction;
use crate::game::{ActionEvent, Stage};
use crate::player::{PlayerConfig, PlayerId, PlayerPublic};

/// Complete public view of the game state
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
    #[serde(default)]
    pub current_bet: u32,
    #[serde(default)]
    pub min_raise: u32,
}

/// Messages that clients can send to the server
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMsg {
    /// Player-initiated action: must specify which player is performing the action.
    Action {
        player_id: PlayerId,
        action: PlayerAction,
    },
    QrReq(String),
    Subscribe,
    RequestState,
    NextHand,
    NewGame {
        players: Vec<PlayerConfig>,
    },
}

/// Messages that the server can send to clients
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMsg {
    State(GameStatePublic),
    Error(String),
    QrRes(Box<[u8]>),
}
