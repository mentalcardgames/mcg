//! Client-server messaging protocol for the Mental Card Game.

use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::game::{ActionEvent, Stage};
use crate::player::{PlayerId, PlayerConfig, PlayerPublic};
use crate::game::PlayerAction;

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
    /// A simple welcome/acknowledgement sent by the server when a client
    /// first connects. The server no longer assigns or embeds a per-connection
    /// `PlayerId` in this message; transports or session managers may choose
    /// to emit their own client-specific information separately if needed.
    Welcome,
    State(GameStatePublic),
    Error(String),
}