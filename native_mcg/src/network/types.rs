use std::error::Error;
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

/// Transport-independent identity of a remote peer.
///
/// Unlike [`ConnectionId`], a peer ID may remain stable across multiple
/// transport connections. Iroh connections use the remote endpoint ID.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PeerId(String);

impl PeerId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Application protocol spoken by a connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolRole {
    Frontend,
    Peer,
}

/// Transport carrying a connection's application protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    WebSocket,
    Iroh,
}

/// Side that initiated an established peer connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerConnectionDirection {
    Incoming,
    Outgoing,
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
    BroadcastFrontend(Backend2FrontendMsg),
    BroadcastPeer(Peer2PeerMsg),
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

/// Typed events emitted by the network supervisor towards application code.
///
/// Application code consumes these events and decides how to respond. Network
/// actors do not access the lobby, game, or other application state directly.
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    /// A frontend connection actor is ready to exchange typed messages.
    FrontendConnected {
        connection_id: ConnectionId,
        transport: TransportKind,
    },
    /// A peer connection actor is ready to exchange typed messages.
    PeerConnected {
        connection_id: ConnectionId,
        peer_id: PeerId,
        transport: TransportKind,
        direction: PeerConnectionDirection,
    },
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

/// Internal events emitted by connection actors towards the supervisor.
#[derive(Debug)]
pub(crate) enum ActorEvent {
    Ready {
        connection_id: ConnectionId,
    },
    FrontendMessage {
        connection_id: ConnectionId,
        message: Frontend2BackendMsg,
    },
    PeerIdentified {
        connection_id: ConnectionId,
        peer_id: PeerId,
    },
    PeerMessage {
        connection_id: ConnectionId,
        message: Peer2PeerMsg,
    },
    Closed {
        connection_id: ConnectionId,
        reason: ConnectionCloseReason,
    },
}

/// Errors returned while interacting with the network supervisor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkError {
    SupervisorStopped,
    ConnectionIdExhausted,
    ConnectionNotFound(ConnectionId),
    ProtocolMismatch {
        connection_id: ConnectionId,
        expected: ProtocolRole,
        actual: ProtocolRole,
    },
    ConnectionBackpressured(ConnectionId),
    ConnectionActorStopped(ConnectionId),
    PeerNotIdentified(ConnectionId),
    TransportAlreadyConfigured(TransportKind),
    TransportUnavailable(TransportKind),
    InvalidPeerTicket(String),
    ConnectionSetupFailed {
        transport: TransportKind,
        message: String,
    },
    ConnectionSetupTimedOut(TransportKind),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SupervisorStopped => formatter.write_str("network supervisor stopped"),
            Self::ConnectionIdExhausted => formatter.write_str("connection ID space exhausted"),
            Self::ConnectionNotFound(connection_id) => {
                write!(formatter, "connection {connection_id} not found")
            }
            Self::ProtocolMismatch {
                connection_id,
                expected,
                actual,
            } => write!(
                formatter,
                "connection {connection_id} has role {actual:?}, expected {expected:?}"
            ),
            Self::ConnectionBackpressured(connection_id) => {
                write!(
                    formatter,
                    "connection {connection_id} outbound queue is full"
                )
            }
            Self::ConnectionActorStopped(connection_id) => {
                write!(formatter, "connection actor {connection_id} stopped")
            }
            Self::PeerNotIdentified(connection_id) => {
                write!(
                    formatter,
                    "peer connection {connection_id} has not identified itself"
                )
            }
            Self::TransportAlreadyConfigured(transport) => {
                write!(formatter, "{transport:?} transport is already configured")
            }
            Self::TransportUnavailable(transport) => {
                write!(formatter, "{transport:?} transport is unavailable")
            }
            Self::InvalidPeerTicket(message) => write!(formatter, "invalid peer ticket: {message}"),
            Self::ConnectionSetupFailed { transport, message } => {
                write!(
                    formatter,
                    "failed to establish {transport:?} connection: {message}"
                )
            }
            Self::ConnectionSetupTimedOut(transport) => {
                write!(
                    formatter,
                    "timed out while establishing {transport:?} connection"
                )
            }
        }
    }
}

impl Error for NetworkError {}

/// Errors returned while establishing peer connections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PeerConnectionError {
    Network(NetworkError),
    DuplicatePeer(PeerId),
    LocalEndpoint(PeerId),
}

impl fmt::Display for PeerConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => error.fmt(formatter),
            Self::DuplicatePeer(peer_id) => {
                write!(
                    formatter,
                    "peer {peer_id} is already connected or connecting"
                )
            }
            Self::LocalEndpoint(peer_id) => {
                write!(formatter, "cannot connect to local endpoint {peer_id}")
            }
        }
    }
}

impl Error for PeerConnectionError {}

impl From<NetworkError> for PeerConnectionError {
    fn from(error: NetworkError) -> Self {
        Self::Network(error)
    }
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
    fn peer_connected_event_requires_a_peer_id() {
        let event = NetworkEvent::PeerConnected {
            connection_id: ConnectionId::new(9),
            peer_id: PeerId::new("peer-9"),
            transport: TransportKind::Iroh,
            direction: PeerConnectionDirection::Incoming,
        };

        assert!(matches!(
            event,
            NetworkEvent::PeerConnected {
                connection_id,
                peer_id,
                transport: TransportKind::Iroh,
                direction: PeerConnectionDirection::Incoming,
            } if connection_id == ConnectionId::new(9) && peer_id == PeerId::new("peer-9")
        ));
    }

    #[test]
    fn network_error_display_formatting() {
        let err = NetworkError::ConnectionNotFound(ConnectionId::new(12));
        assert_eq!(err.to_string(), "connection 12 not found");

        let err = NetworkError::ProtocolMismatch {
            connection_id: ConnectionId::new(12),
            expected: ProtocolRole::Frontend,
            actual: ProtocolRole::Peer,
        };
        assert_eq!(
            err.to_string(),
            "connection 12 has role Peer, expected Frontend"
        );
    }

    #[test]
    fn peer_connection_error_from_network_error() {
        let net_err = NetworkError::SupervisorStopped;
        let peer_err: PeerConnectionError = net_err.clone().into();
        assert_eq!(peer_err, PeerConnectionError::Network(net_err));
        assert_eq!(peer_err.to_string(), "network supervisor stopped");

        let dup_err = PeerConnectionError::DuplicatePeer(PeerId::new("p1"));
        assert_eq!(
            dup_err.to_string(),
            "peer p1 is already connected or connecting"
        );

        let local_err = PeerConnectionError::LocalEndpoint(PeerId::new("p1"));
        assert_eq!(local_err.to_string(), "cannot connect to local endpoint p1");
    }
}
