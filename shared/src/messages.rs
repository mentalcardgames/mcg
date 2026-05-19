//! Client-server messaging protocol for the Mental Card Game.

use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::game::PlayerAction;
use crate::game::{ActionEvent, Stage};
use crate::player::{PlayerConfig, PlayerId, PlayerPublic};
use std::collections::HashMap;

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
pub enum Frontend2BackendMsg {
    /// Player-initiated action: gets applied to the game
    Action {
        player_id: PlayerId,
        action: PlayerAction,
    },
    QrReq(String),
    Subscribe,
    RequestState,
    Ping,
    NextHand,
    NewGame {
        players: Vec<PlayerConfig>,
    },
    /// Push a complete game state to the server (P2P state sync between backend nodes)
    /// The state is a serialized Game struct from native_mcg
    PushState {
        state: serde_json::Value,
    },
    QrValue(String),
    GetTicket,
    GetIP,
    PlayerCount(usize),
    LobbyOpen(String),
    PlayerName(String),
    GetOurName,
    Disconnect,
    ReadyUpdate(bool),
}

/// Messages that the server can send to clients
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Backend2FrontendMsg {
    State(GameStatePublic),
    Error(String),
    Pong,
    TicketValue(String),
    IPValue(String),
    QrRes(Box<[u8]>),
    NewPlayer(String),
    OurName(String),
    RemovePlayer(String),
    PlayerReady(String, bool),
}

//Messages two peers send between eachother
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Peer2PeerMsg {
    Ping,
    Pong,
    Connect(String, Option<String>), // Send our name and our endpointticket (if we have one) to the peer we're connecting to
    Disconnect(String), // Send our name to the peer we're disconnecting from (so they can remove us from their peer list)
    Reject(String), // Send a reason for rejecting the connection to the peer we're rejecting
    Payload(String),
    LobbyAccept(usize, String), // Number of max players in the lobby, gametype, and trigger to open lobby on the receiving peer
    Peers(HashMap<String, (String, String)>), // EndpointId (as string) -> Peer's Name and Ticket
    NewName(String), // New name for the peer (after a rename due to being a duplicated name)
    PeerReady(String, bool), // Peer name and ready status
}