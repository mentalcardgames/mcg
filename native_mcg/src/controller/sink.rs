use mcg_shared::{Backend2FrontendMsg, Peer2PeerMsg};
use tokio::sync::mpsc;

use crate::network::ConnectionId;

use super::types::ControllerCommand;

/// Sink trait representing an outbound destination for Controller commands.
///
/// This trait decouples the Controller's sequential business logic from the
/// concrete network dispatching mechanism, enabling deterministic in-memory testing.
pub trait ControllerSink: Send {
    /// Send a command to the outbound destination.
    fn send_command(&mut self, command: ControllerCommand);

    /// Convenience helper to broadcast a message to all connected frontends.
    fn broadcast_frontend(&mut self, message: Backend2FrontendMsg) {
        self.send_command(ControllerCommand::BroadcastFrontend(message));
    }

    /// Convenience helper to send a message to a specific frontend.
    fn send_frontend(&mut self, connection_id: ConnectionId, message: Backend2FrontendMsg) {
        self.send_command(ControllerCommand::SendFrontend {
            connection_id,
            message,
        });
    }

    /// Convenience helper to broadcast a message to all connected peers.
    fn broadcast_peer(&mut self, message: Peer2PeerMsg) {
        self.send_command(ControllerCommand::BroadcastPeer(message));
    }

    /// Convenience helper to send a message to a specific peer.
    fn send_peer(&mut self, connection_id: ConnectionId, message: Peer2PeerMsg) {
        self.send_command(ControllerCommand::SendPeer {
            connection_id,
            message,
        });
    }

    /// Convenience helper to request closing a connection.
    fn close_connection(&mut self, connection_id: ConnectionId, reason: String) {
        self.send_command(ControllerCommand::CloseConnection {
            connection_id,
            reason,
        });
    }

    /// Convenience helper to request initiating an outgoing peer connection.
    fn connect_peer(&mut self, ticket: String) {
        self.send_command(ControllerCommand::ConnectPeer { ticket });
    }
}

/// In-memory sink recording all dispatched commands for unit testing.
#[derive(Clone, Debug, Default)]
pub struct InMemoryControllerSink {
    pub commands: Vec<ControllerCommand>,
}

impl InMemoryControllerSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

impl ControllerSink for InMemoryControllerSink {
    fn send_command(&mut self, command: ControllerCommand) {
        self.commands.push(command);
    }
}

/// Channel-backed sink forwarding commands from the Controller thread to the async network shell.
pub struct ChannelControllerSink {
    command_tx: mpsc::UnboundedSender<ControllerCommand>,
}

impl ChannelControllerSink {
    pub fn new(command_tx: mpsc::UnboundedSender<ControllerCommand>) -> Self {
        Self { command_tx }
    }
}

impl ControllerSink for ChannelControllerSink {
    fn send_command(&mut self, command: ControllerCommand) {
        if let Err(error) = self.command_tx.send(command) {
            tracing::warn!(
                ?error,
                "failed to send controller command; receiver dropped"
            );
        }
    }
}
