//! Client-server messaging protocol for the Mental Card Game.

use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::game::PlayerAction;
use crate::game::{ActionEvent, Stage};
use crate::player::{PlayerConfig, PlayerId, PlayerPublic};
use std::collections::HashMap;

/// Complete public view of the game state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PokerStatePublic {
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

/// Messages that the frontend sends to the backend
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Frontend2BackendMsg {
    /// Player-initiated action: gets applied to the game
    Action {
        player_id: PlayerId,
        action: PlayerAction,
    },
    QrReq(String),
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
    GetPlayers,
    Disconnect,
    ReadyUpdate(bool),
    UpdatePlayerSetup(Vec<PlayerConfig>),
    RequestPlayerSetup,
}

/// Messages that the backend sends to the frontend
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Backend2FrontendMsg {
    UpdatePokerState(PokerStatePublic),
    PlayerSetup(Vec<PlayerConfig>),
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

/// Messages that are send between two peers
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Peer2PeerMsg {
    Ping,
    Pong,
    Connect(String, String), // Send our name and our endpoint ticket to the peer we're connecting to
    Disconnect(String), // Send our name to the peer we're disconnecting from (so they can remove us from their peer list)
    Reject(String),     // Send a reason for rejecting the connection to the peer we're rejecting
    Payload(String),
    LobbyAccept(usize, String), // Number of max players in the lobby, gametype, and trigger to open lobby on the receiving peer
    Peers(HashMap<String, (String, String)>), // EndpointId (as string) -> Peer's Name and Ticket
    NewName(String), // New name for the peer (after a rename due to being a duplicated name)
    PeerReady(String, bool), // Peer name and ready status
    RequestReady, // Request the peer's ready status for the anti-race condition redundancy code when we scan a QR code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend2frontend_roundtrip() {
        let msgs = vec![
            Backend2FrontendMsg::Pong,
            Backend2FrontendMsg::Error("error msg".into()),
            Backend2FrontendMsg::TicketValue("ticket123".into()),
            Backend2FrontendMsg::IPValue("127.0.0.1".into()),
            Backend2FrontendMsg::QrRes(vec![1, 2, 3].into_boxed_slice()),
            Backend2FrontendMsg::NewPlayer("Alice".into()),
            Backend2FrontendMsg::OurName("Bob".into()),
            Backend2FrontendMsg::RemovePlayer("Charlie".into()),
            Backend2FrontendMsg::PlayerReady("Dave".into(), true),
            Backend2FrontendMsg::PlayerSetup(vec![PlayerConfig {
                id: PlayerId(0),
                name: "Alice".into(),
                is_bot: false,
            }]),
            Backend2FrontendMsg::UpdatePokerState(PokerStatePublic {
                players: vec![],
                community: vec![],
                pot: 100,
                sb: 5,
                bb: 10,
                to_act: PlayerId(0),
                stage: Stage::Preflop,
                winner_ids: vec![],
                action_log: vec![],
                current_bet: 10,
                min_raise: 20,
            }),
        ];

        for msg in msgs {
            let json = serde_json::to_string(&msg).expect("serialize failed");
            let deserialized: Backend2FrontendMsg =
                serde_json::from_str(&json).expect("deserialize failed");
            let json2 = serde_json::to_string(&deserialized).expect("reserialize failed");
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn test_frontend2backend_roundtrip() {
        let msgs = vec![
            Frontend2BackendMsg::GetOurName,
            Frontend2BackendMsg::GetPlayers,
            Frontend2BackendMsg::GetTicket,
            Frontend2BackendMsg::GetIP,
            Frontend2BackendMsg::Ping,
            Frontend2BackendMsg::NextHand,
            Frontend2BackendMsg::Disconnect,
            Frontend2BackendMsg::ReadyUpdate(true),
            Frontend2BackendMsg::PlayerCount(4),
            Frontend2BackendMsg::PlayerName("Alice".into()),
            Frontend2BackendMsg::LobbyOpen("Poker".into()),
            Frontend2BackendMsg::QrValue("ticket".into()),
            Frontend2BackendMsg::QrReq("req".into()),
            Frontend2BackendMsg::RequestState,
            Frontend2BackendMsg::RequestPlayerSetup,
            Frontend2BackendMsg::UpdatePlayerSetup(vec![PlayerConfig {
                id: PlayerId(0),
                name: "Alice".into(),
                is_bot: false,
            }]),
            Frontend2BackendMsg::NewGame { players: vec![] },
            Frontend2BackendMsg::Action {
                player_id: PlayerId(0),
                action: PlayerAction::Fold,
            },
        ];

        for msg in msgs {
            let json = serde_json::to_string(&msg).expect("serialize failed");
            let deserialized: Frontend2BackendMsg =
                serde_json::from_str(&json).expect("deserialize failed");
            let json2 = serde_json::to_string(&deserialized).expect("reserialize failed");
            assert_eq!(json, json2);
        }
    }
}
