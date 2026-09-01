use axum::extract::ws::WebSocket;
use mcg_shared::{Backend2FrontendMsg, Peer2PeerMsg};
use tokio::sync::mpsc;

use super::NetworkSupervisor;
use crate::network::iroh::{run_iroh_frontend_actor, run_iroh_peer_actor, IrohReader, IrohWriter};
use crate::network::websocket::{run_websocket_frontend_actor, run_websocket_pending_peer_actor};
use crate::network::{
    ConnectionId, FrontendConnectionCommand, NetworkCommand, NetworkError, NetworkEvent,
    PeerConnectionCommand, PeerConnectionDirection, PeerId, ProtocolRole, TransportKind,
};

#[derive(Clone)]
pub(crate) struct ManagedConnection {
    pub(crate) transport: TransportKind,
    pub(crate) target: ManagedTarget,
}

#[derive(Clone)]
pub(crate) enum ManagedTarget {
    Frontend {
        command_tx: mpsc::Sender<FrontendConnectionCommand>,
    },
    PendingPeer {
        command_tx: mpsc::Sender<PeerConnectionCommand>,
    },
    Peer {
        peer_id: PeerId,
        direction: PeerConnectionDirection,
        command_tx: mpsc::Sender<PeerConnectionCommand>,
    },
}

impl ManagedConnection {
    pub(crate) fn role(&self) -> ProtocolRole {
        match &self.target {
            ManagedTarget::Frontend { .. } => ProtocolRole::Frontend,
            ManagedTarget::PendingPeer { .. } => ProtocolRole::Peer,
            ManagedTarget::Peer { .. } => ProtocolRole::Peer,
        }
    }

    pub(crate) fn connected_event(&self, connection_id: ConnectionId) -> Option<NetworkEvent> {
        match &self.target {
            ManagedTarget::Frontend { .. } => Some(NetworkEvent::FrontendConnected {
                connection_id,
                transport: self.transport,
            }),
            ManagedTarget::PendingPeer { .. } => {
                tracing::warn!(%connection_id, "pending peer reported ready before identification");
                None
            }
            ManagedTarget::Peer {
                peer_id, direction, ..
            } => Some(NetworkEvent::PeerConnected {
                connection_id,
                peer_id: peer_id.clone(),
                transport: self.transport,
                direction: *direction,
            }),
        }
    }
}

impl NetworkSupervisor {
    pub(super) fn allocate_connection_id(&mut self) -> Result<ConnectionId, NetworkError> {
        let next = self
            .next_connection_id
            .checked_add(1)
            .ok_or(NetworkError::ConnectionIdExhausted)?;
        let connection_id = ConnectionId::new(self.next_connection_id);
        self.next_connection_id = next;
        Ok(connection_id)
    }

    pub(super) fn register_frontend_websocket(
        &mut self,
        socket: WebSocket,
    ) -> Result<ConnectionId, NetworkError> {
        let connection_id = self.allocate_connection_id()?;
        let (command_tx, command_rx) = mpsc::channel(self.connection_channel_capacity);
        let actor_event_tx = self.actor_event_tx.clone();

        self.connections.insert(
            connection_id,
            ManagedConnection {
                transport: TransportKind::WebSocket,
                target: ManagedTarget::Frontend { command_tx },
            },
        );
        self.tasks.spawn(run_websocket_frontend_actor(
            connection_id,
            socket,
            actor_event_tx,
            command_rx,
        ));

        Ok(connection_id)
    }

    pub(super) fn register_pending_peer_websocket(
        &mut self,
        socket: WebSocket,
    ) -> Result<ConnectionId, NetworkError> {
        let connection_id = self.allocate_connection_id()?;
        let (command_tx, command_rx) = mpsc::channel(self.connection_channel_capacity);
        let actor_event_tx = self.actor_event_tx.clone();

        self.connections.insert(
            connection_id,
            ManagedConnection {
                transport: TransportKind::WebSocket,
                target: ManagedTarget::PendingPeer { command_tx },
            },
        );
        self.tasks.spawn(run_websocket_pending_peer_actor(
            connection_id,
            socket,
            actor_event_tx,
            command_rx,
        ));

        Ok(connection_id)
    }

    pub(super) fn register_iroh_peer(
        &mut self,
        peer_id: PeerId,
        direction: PeerConnectionDirection,
        reader: IrohReader,
        writer: IrohWriter,
    ) -> Result<ConnectionId, NetworkError> {
        let connection_id = self.allocate_connection_id()?;
        let (command_tx, command_rx) = mpsc::channel(self.connection_channel_capacity);
        let actor_event_tx = self.actor_event_tx.clone();

        self.connections.insert(
            connection_id,
            ManagedConnection {
                transport: TransportKind::Iroh,
                target: ManagedTarget::Peer {
                    peer_id,
                    direction,
                    command_tx,
                },
            },
        );
        self.tasks.spawn(run_iroh_peer_actor(
            connection_id,
            reader,
            writer,
            actor_event_tx,
            command_rx,
        ));

        Ok(connection_id)
    }

    pub(super) fn register_iroh_frontend(
        &mut self,
        reader: IrohReader,
        writer: IrohWriter,
    ) -> Result<ConnectionId, NetworkError> {
        let connection_id = self.allocate_connection_id()?;
        let (command_tx, command_rx) = mpsc::channel(self.connection_channel_capacity);
        let actor_event_tx = self.actor_event_tx.clone();

        self.connections.insert(
            connection_id,
            ManagedConnection {
                transport: TransportKind::Iroh,
                target: ManagedTarget::Frontend { command_tx },
            },
        );
        self.tasks.spawn(run_iroh_frontend_actor(
            connection_id,
            reader,
            writer,
            actor_event_tx,
            command_rx,
        ));

        Ok(connection_id)
    }

    pub(super) fn execute_command(&mut self, command: NetworkCommand) -> Result<(), NetworkError> {
        match command {
            NetworkCommand::BroadcastFrontend(message) => {
                self.broadcast_frontend(message);
                Ok(())
            }
            NetworkCommand::BroadcastPeer(message) => {
                self.broadcast_peer(message);
                Ok(())
            }
            NetworkCommand::SendFrontend {
                connection_id,
                message,
            } => self.send_frontend(connection_id, FrontendConnectionCommand::Send(message)),
            NetworkCommand::SendPeer {
                connection_id,
                message,
            } => self.send_peer(connection_id, PeerConnectionCommand::Send(message)),
            NetworkCommand::CloseConnection {
                connection_id,
                reason,
            } => self.close_connection(connection_id, reason),
        }
    }

    pub(super) fn broadcast_frontend(&mut self, message: Backend2FrontendMsg) {
        for (&connection_id, connection) in &self.connections {
            if let ManagedTarget::Frontend { command_tx } = &connection.target {
                if let Err(error) =
                    command_tx.try_send(FrontendConnectionCommand::Send(message.clone()))
                {
                    tracing::warn!(%connection_id, %error, "failed to broadcast frontend message to connection");
                }
            }
        }
    }

    pub(super) fn broadcast_peer(&mut self, message: Peer2PeerMsg) {
        for (&connection_id, connection) in &self.connections {
            if let ManagedTarget::Peer { command_tx, .. } = &connection.target {
                if let Err(error) =
                    command_tx.try_send(PeerConnectionCommand::Send(message.clone()))
                {
                    tracing::warn!(%connection_id, %error, "failed to broadcast peer message to connection");
                }
            }
        }
    }

    pub(super) fn send_frontend(
        &mut self,
        connection_id: ConnectionId,
        command: FrontendConnectionCommand,
    ) -> Result<(), NetworkError> {
        let connection = self
            .connections
            .get(&connection_id)
            .ok_or(NetworkError::ConnectionNotFound(connection_id))?;
        let command_tx = match &connection.target {
            ManagedTarget::Frontend { command_tx } => command_tx.clone(),
            ManagedTarget::PendingPeer { .. } | ManagedTarget::Peer { .. } => {
                return Err(NetworkError::ProtocolMismatch {
                    connection_id,
                    expected: ProtocolRole::Frontend,
                    actual: connection.role(),
                });
            }
        };

        self.try_send(connection_id, command_tx, command)
    }

    pub(super) fn send_peer(
        &mut self,
        connection_id: ConnectionId,
        command: PeerConnectionCommand,
    ) -> Result<(), NetworkError> {
        let connection = self
            .connections
            .get(&connection_id)
            .ok_or(NetworkError::ConnectionNotFound(connection_id))?;
        let command_tx = match &connection.target {
            ManagedTarget::Peer { command_tx, .. } => command_tx.clone(),
            ManagedTarget::Frontend { .. } => {
                return Err(NetworkError::ProtocolMismatch {
                    connection_id,
                    expected: ProtocolRole::Peer,
                    actual: connection.role(),
                });
            }
            ManagedTarget::PendingPeer { .. } => {
                return Err(NetworkError::PeerNotIdentified(connection_id));
            }
        };

        self.try_send(connection_id, command_tx, command)
    }

    pub(super) fn close_connection(
        &mut self,
        connection_id: ConnectionId,
        reason: String,
    ) -> Result<(), NetworkError> {
        let connection = self
            .connections
            .get(&connection_id)
            .cloned()
            .ok_or(NetworkError::ConnectionNotFound(connection_id))?;

        match connection.target {
            ManagedTarget::Frontend { command_tx } => self.try_send(
                connection_id,
                command_tx,
                FrontendConnectionCommand::Close { reason },
            ),
            ManagedTarget::Peer { command_tx, .. } => self.try_send(
                connection_id,
                command_tx,
                PeerConnectionCommand::Close { reason },
            ),
            ManagedTarget::PendingPeer { command_tx } => self.try_send(
                connection_id,
                command_tx,
                PeerConnectionCommand::Close { reason },
            ),
        }
    }

    pub(super) fn try_send<T>(
        &mut self,
        connection_id: ConnectionId,
        command_tx: mpsc::Sender<T>,
        command: T,
    ) -> Result<(), NetworkError> {
        match command_tx.try_send(command) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                Err(NetworkError::ConnectionBackpressured(connection_id))
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.connections.remove(&connection_id);
                Err(NetworkError::ConnectionActorStopped(connection_id))
            }
        }
    }
}
