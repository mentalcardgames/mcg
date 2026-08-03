use std::fmt;

use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, Peer2PeerMsg};

/// Process-local identifier for an open network connection.
///
/// A connection ID identifies a concrete transport connection. It is not part
/// of the wire protocol and must not be used as a persistent peer identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConnectionId(u64);

impl ConnectionId {
    /// Creates a connection ID from a process-local numeric value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the process-local numeric value.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Application protocol spoken by a connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionRole {
    Frontend,
    Peer,
}

/// Transport carrying a connection's application protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    WebSocket,
    Iroh,
}

/// Stable metadata associated with one open connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectionInfo {
    pub id: ConnectionId,
    pub role: ConnectionRole,
    pub transport: TransportKind,
}

/// Reason why a previously open connection actor stopped.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionCloseReason {
    RemoteClosed,
    TransportError(String),
    ProtocolError(String),
    OutboundChannelClosed,
    EventReceiverClosed,
    LocalRequest(String),
}

/// Commands accepted by a frontend connection actor.
#[derive(Clone, Debug)]
pub enum FrontendConnectionCommand {
    Send(Backend2FrontendMsg),
    Close { reason: String },
}

/// Commands accepted by an established peer connection actor.
#[derive(Clone, Debug)]
pub enum PeerConnectionCommand {
    Send(Peer2PeerMsg),
    Close { reason: String },
}

/// Commands sent from application logic to the network supervisor.
#[derive(Clone, Debug)]
pub enum NetworkCommand {
    SendFrontend {
        connection_id: ConnectionId,
        message: Backend2FrontendMsg,
    },
    SendPeer {
        connection_id: ConnectionId,
        message: Peer2PeerMsg,
    },
    CloseConnection {
        connection_id: ConnectionId,
        reason: String,
    },
}

/// Typed events emitted by network connection actors.
///
/// Application code consumes these events and decides how to respond. Network
/// actors do not access the lobby, game, or other application state directly.
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    /// A connection actor is ready to exchange typed messages.
    ConnectionOpened { connection: ConnectionInfo },
    /// A frontend message received on a concrete connection.
    FrontendMessage {
        connection_id: ConnectionId,
        message: Frontend2BackendMsg,
    },
    /// A peer message received on a concrete connection.
    PeerMessage {
        connection_id: ConnectionId,
        message: Peer2PeerMsg,
    },
    /// The transport connection ended and must be removed from its owner.
    ConnectionClosed {
        connection_id: ConnectionId,
        reason: ConnectionCloseReason,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_id_exposes_its_process_local_value() {
        let connection_id = ConnectionId::new(42);

        assert_eq!(connection_id.get(), 42);
        assert_eq!(connection_id.to_string(), "42");
    }

    #[test]
    fn frontend_event_keeps_source_connection_and_typed_message() {
        let event = NetworkEvent::FrontendMessage {
            connection_id: ConnectionId::new(7),
            message: Frontend2BackendMsg::Ping,
        };

        match event {
            NetworkEvent::FrontendMessage {
                connection_id,
                message,
            } => {
                assert_eq!(connection_id, ConnectionId::new(7));
                assert!(matches!(message, Frontend2BackendMsg::Ping));
            }
            _ => panic!("expected a frontend message event"),
        }
    }

    #[test]
    fn connection_metadata_keeps_protocol_role_separate_from_transport() {
        let connection = ConnectionInfo {
            id: ConnectionId::new(9),
            role: ConnectionRole::Frontend,
            transport: TransportKind::WebSocket,
        };

        assert_eq!(connection.id, ConnectionId::new(9));
        assert_eq!(connection.role, ConnectionRole::Frontend);
        assert_eq!(connection.transport, TransportKind::WebSocket);
    }
}
