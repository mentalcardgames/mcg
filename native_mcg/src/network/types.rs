use std::fmt;

use mcg_shared::Frontend2BackendMsg;

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

/// Typed events emitted by network connection actors.
///
/// Application code consumes these events and decides how to respond. Network
/// actors do not access the lobby, game, or other application state directly.
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    /// A frontend message received on a concrete connection.
    FrontendMessage {
        connection_id: ConnectionId,
        message: Frontend2BackendMsg,
    },
    /// The transport connection ended and must be removed from its owner.
    ConnectionClosed { connection_id: ConnectionId },
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
            NetworkEvent::ConnectionClosed { .. } => {
                panic!("expected a frontend message event")
            }
        }
    }
}
