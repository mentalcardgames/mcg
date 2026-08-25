use std::error::Error;
use std::fmt;

use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, Peer2PeerMsg, PlayerAction, PlayerId};
use tokio::sync::oneshot;

use crate::network::{ConnectionId, NetworkEvent};

/// Errors that can occur when communicating with the Controller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerError {
    /// The controller thread has stopped or closed its event receiver.
    ControllerStopped,
    /// The controller's incoming event queue is full (for bounded channels).
    QueueFull,
    /// An HTTP or RPC request to the controller timed out or was dropped.
    ResponseDropped,
}

impl fmt::Display for ControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ControllerStopped => write!(f, "controller thread has stopped"),
            Self::QueueFull => write!(f, "controller event queue is full"),
            Self::ResponseDropped => write!(f, "controller dropped the response channel"),
        }
    }
}

impl Error for ControllerError {}

/// Events received by the synchronous Controller from the async network shell,
/// HTTP API, bot drivers, or system lifecycle.
#[derive(Debug)]
pub enum ControllerEvent {
    /// Network-level event from the NetworkSupervisor (connections, incoming messages, closures).
    Network(NetworkEvent),
    /// HTTP API message request requiring a synchronous response via a oneshot channel.
    HttpRequest {
        message: Frontend2BackendMsg,
        reply_tx: oneshot::Sender<Backend2FrontendMsg>,
    },
    /// Bot action dispatched by an external bot driver.
    BotAction {
        player_id: PlayerId,
        action: PlayerAction,
    },
    /// Request to cleanly shut down the controller event loop.
    Shutdown,
}

/// Commands emitted by the Controller towards the async network shell.
#[derive(Clone, Debug)]
pub enum ControllerCommand {
    /// Broadcast a message to all active frontend connections.
    BroadcastFrontend(Backend2FrontendMsg),
    /// Send a message to a specific frontend connection.
    SendFrontend {
        connection_id: ConnectionId,
        message: Backend2FrontendMsg,
    },
    /// Send a message to a specific peer connection.
    SendPeer {
        connection_id: ConnectionId,
        message: Peer2PeerMsg,
    },
    /// Broadcast a message to all established peer connections.
    BroadcastPeer(Peer2PeerMsg),
    /// Close a network connection with a human-readable reason.
    CloseConnection {
        connection_id: ConnectionId,
        reason: String,
    },
    /// Request the network layer to initiate an outgoing peer connection.
    ConnectPeer { ticket: String },
}
