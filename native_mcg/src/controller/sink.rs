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

/// Sink backed directly by [`NetworkHandle`] sending commands synchronously via its blocking API.
impl ControllerSink for crate::network::NetworkHandle {
    fn send_command(&mut self, command: ControllerCommand) {
        match command {
            ControllerCommand::BroadcastFrontend(message) => {
                if let Err(error) = self.blocking_broadcast_frontend(message) {
                    tracing::warn!(%error, "failed to broadcast frontend message from controller");
                }
            }
            ControllerCommand::BroadcastPeer(message) => {
                if let Err(error) = self.blocking_broadcast_peer(message) {
                    tracing::warn!(%error, "failed to broadcast peer message from controller");
                }
            }
            ControllerCommand::SendFrontend {
                connection_id,
                message,
            } => {
                if let Err(error) = self.blocking_unicast_frontend(connection_id, message) {
                    tracing::warn!(%connection_id, %error, "failed to send frontend message from controller");
                }
            }
            ControllerCommand::SendPeer {
                connection_id,
                message,
            } => {
                if let Err(error) = self.blocking_unicast_peer(connection_id, message) {
                    tracing::warn!(%connection_id, %error, "failed to send peer message from controller");
                }
            }
            ControllerCommand::CloseConnection {
                connection_id,
                reason,
            } => {
                if let Err(error) = self.blocking_close_connection(connection_id, reason) {
                    tracing::warn!(%connection_id, %error, "failed to close connection from controller");
                }
            }
            ControllerCommand::ConnectPeer { ticket } => {
                if let Err(error) = self.blocking_establish_iroh_peer_connection(ticket) {
                    tracing::warn!(%error, "failed to connect to iroh peer from controller");
                }
            }
        }
    }
}

/// Controller sink combining a [`crate::network::NetworkHandle`] with an optional public state watch sender for bot observation.
pub struct NetworkControllerSink {
    network: crate::network::NetworkHandle,
    state_watch_tx: Option<tokio::sync::watch::Sender<Option<mcg_shared::PokerStatePublic>>>,
}

impl NetworkControllerSink {
    pub fn new(network: crate::network::NetworkHandle) -> Self {
        Self {
            network,
            state_watch_tx: None,
        }
    }

    pub fn with_state_watch(
        network: crate::network::NetworkHandle,
        state_watch_tx: tokio::sync::watch::Sender<Option<mcg_shared::PokerStatePublic>>,
    ) -> Self {
        Self {
            network,
            state_watch_tx: Some(state_watch_tx),
        }
    }
}

impl ControllerSink for NetworkControllerSink {
    fn send_command(&mut self, command: ControllerCommand) {
        if let ControllerCommand::BroadcastFrontend(
            mcg_shared::Backend2FrontendMsg::UpdatePokerState(ref gs),
        ) = command
        {
            if let Some(ref tx) = self.state_watch_tx {
                let _ = tx.send_replace(Some(gs.clone()));
            }
        }
        self.network.send_command(command);
    }
}
