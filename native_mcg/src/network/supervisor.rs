use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use axum::extract::ws::WebSocket;
use tokio::sync::{mpsc, oneshot};

use super::{
    run_websocket_actor, ConnectionId, ConnectionRole, FrontendConnectionCommand, NetworkCommand,
    NetworkEvent,
};

const DEFAULT_CONTROL_CHANNEL_CAPACITY: usize = 256;
const DEFAULT_CONNECTION_CHANNEL_CAPACITY: usize = 64;

/// Errors returned while interacting with the network supervisor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkError {
    SupervisorStopped,
    ConnectionIdExhausted,
    ConnectionNotFound(ConnectionId),
    ProtocolMismatch {
        connection_id: ConnectionId,
        expected: ConnectionRole,
        actual: ConnectionRole,
    },
    ConnectionBackpressured(ConnectionId),
    ConnectionActorStopped(ConnectionId),
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
        }
    }
}

impl Error for NetworkError {}

/// Cloneable interface used by transports and application logic.
#[derive(Clone)]
pub struct NetworkHandle {
    request_tx: mpsc::Sender<SupervisorRequest>,
}

impl NetworkHandle {
    /// Registers an upgraded frontend WebSocket with the supervisor.
    pub async fn register_websocket(
        &self,
        socket: WebSocket,
    ) -> Result<ConnectionId, NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::RegisterWebSocket {
                socket: Box::new(socket),
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Sends a targeted command to the supervisor and waits until it has been
    /// accepted by the destination connection queue.
    pub async fn send_command(&self, command: NetworkCommand) -> Result<(), NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::Execute {
                command,
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }
}

/// Owns active connection handles and routes commands and actor events.
pub struct NetworkSupervisor {
    /// Output for connection actors to report NetworkEvents.
    actor_event_tx: mpsc::Sender<NetworkEvent>,
    /// Input of NetworkEvents.
    actor_event_rx: mpsc::Receiver<NetworkEvent>,
    /// Output for forwarding NetworkEvents from actors towards Controller.
    application_event_tx: mpsc::Sender<NetworkEvent>,
    /// Requests from NetworkHandles.
    request_rx: mpsc::Receiver<SupervisorRequest>,
    /// Container with all connections with other peers or with frontends.
    connections: HashMap<ConnectionId, ManagedConnection>,
    next_connection_id: u64,
    connection_channel_capacity: usize,
}

impl NetworkSupervisor {
    /// Creates a supervisor and its cloneable external handle.
    pub fn new(application_event_tx: mpsc::Sender<NetworkEvent>) -> (Self, NetworkHandle) {
        Self::with_capacities(
            application_event_tx,
            DEFAULT_CONTROL_CHANNEL_CAPACITY,
            DEFAULT_CONNECTION_CHANNEL_CAPACITY,
        )
    }

    fn with_capacities(
        application_event_tx: mpsc::Sender<NetworkEvent>,
        control_channel_capacity: usize,
        connection_channel_capacity: usize,
    ) -> (Self, NetworkHandle) {
        assert!(control_channel_capacity > 0);
        assert!(connection_channel_capacity > 0);

        let (request_tx, request_rx) = mpsc::channel(control_channel_capacity);
        let (actor_event_tx, actor_event_rx) = mpsc::channel(control_channel_capacity);
        let handle = NetworkHandle { request_tx };
        let supervisor = Self {
            request_rx,
            actor_event_tx,
            actor_event_rx,
            application_event_tx,
            connections: HashMap::new(),
            next_connection_id: 0,
            connection_channel_capacity,
        };
        (supervisor, handle)
    }

    /// Runs until every [`NetworkHandle`] has been dropped or the application
    /// event receiver is closed.
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Listed for requests; What should NetworkSupervisor do?
                request = self.request_rx.recv() => {
                    let Some(request) = request else {
                        break;
                    };
                    self.handle_request(request);
                }
                // Forward NetworkEvents out of NetworkSupervisor
                event = self.actor_event_rx.recv() => {
                    let Some(event) = event else {
                        break;
                    };
                    self.handle_actor_event(&event);
                    if self.application_event_tx.send(event).await.is_err() {
                        break;
                    }
                }
            }
        }
        tracing::info!(
            connections = self.connections.len(),
            "network supervisor stopped"
        );
    }

    fn handle_request(&mut self, request: SupervisorRequest) {
        match request {
            SupervisorRequest::RegisterWebSocket {
                socket,
                response_tx,
            } => {
                let result = self.register_websocket(*socket);
                let _ = response_tx.send(result);
            }
            SupervisorRequest::Execute {
                command,
                response_tx,
            } => {
                let result = self.execute_command(command);
                let _ = response_tx.send(result);
            }
        }
    }

    fn register_websocket(&mut self, socket: WebSocket) -> Result<ConnectionId, NetworkError> {
        let connection_id = self.allocate_connection_id()?;
        let (command_tx, command_rx) = mpsc::channel(self.connection_channel_capacity);
        let actor_event_tx = self.actor_event_tx.clone();

        self.connections
            .insert(connection_id, ManagedConnection::Frontend { command_tx });
        tokio::spawn(run_websocket_actor(
            connection_id,
            socket,
            actor_event_tx,
            command_rx,
        ));

        Ok(connection_id)
    }

    fn allocate_connection_id(&mut self) -> Result<ConnectionId, NetworkError> {
        let next = self
            .next_connection_id
            .checked_add(1)
            .ok_or(NetworkError::ConnectionIdExhausted)?;
        let connection_id = ConnectionId::new(self.next_connection_id);
        self.next_connection_id = next;
        Ok(connection_id)
    }

    fn execute_command(&mut self, command: NetworkCommand) -> Result<(), NetworkError> {
        match command {
            NetworkCommand::SendFrontend {
                connection_id,
                message,
            } => self.send_frontend(connection_id, FrontendConnectionCommand::Send(message)),
            NetworkCommand::SendPeer { connection_id, .. } => {
                let connection = self
                    .connections
                    .get(&connection_id)
                    .ok_or(NetworkError::ConnectionNotFound(connection_id))?;
                Err(NetworkError::ProtocolMismatch {
                    connection_id,
                    expected: ConnectionRole::Peer,
                    actual: connection.role(),
                })
            }
            NetworkCommand::CloseConnection {
                connection_id,
                reason,
            } => self.send_frontend(connection_id, FrontendConnectionCommand::Close { reason }),
        }
    }

    fn send_frontend(
        &mut self,
        connection_id: ConnectionId,
        command: FrontendConnectionCommand,
    ) -> Result<(), NetworkError> {
        let connection = self
            .connections
            .get(&connection_id)
            .ok_or(NetworkError::ConnectionNotFound(connection_id))?;
        let command_tx = match connection {
            ManagedConnection::Frontend { command_tx } => command_tx.clone(),
        };

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

    fn handle_actor_event(&mut self, event: &NetworkEvent) {
        if let NetworkEvent::ConnectionClosed { connection_id, .. } = event {
            self.connections.remove(connection_id);
        }
    }
}

/// Type that represents the internal contract between NetworkSupervisor and NetworkHandle for message calls on the handle.
enum SupervisorRequest {
    RegisterWebSocket {
        socket: Box<WebSocket>,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    Execute {
        command: NetworkCommand,
        response_tx: oneshot::Sender<Result<(), NetworkError>>,
    },
}

enum ManagedConnection {
    Frontend {
        command_tx: mpsc::Sender<FrontendConnectionCommand>,
    },
}

impl ManagedConnection {
    fn role(&self) -> ConnectionRole {
        match self {
            Self::Frontend { .. } => ConnectionRole::Frontend,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Result;
    use axum::{
        extract::{ws::WebSocketUpgrade, State},
        response::IntoResponse,
        routing::get,
        Router,
    };
    use futures::{SinkExt, StreamExt};
    use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, Peer2PeerMsg};
    use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

    use super::*;
    use crate::network::{ConnectionCloseReason, ConnectionInfo, TransportKind};

    async fn test_ws_handler(
        ws: WebSocketUpgrade,
        State(network): State<NetworkHandle>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| async move {
            network
                .register_websocket(socket)
                .await
                .expect("test supervisor should be running");
        })
    }

    #[tokio::test]
    async fn supervisor_registers_routes_closes_and_removes_websocket() -> Result<()> {
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let app = Router::new()
            .route("/ws", get(test_ws_handler))
            .with_state(network.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server_task = tokio::spawn(async move { axum::serve(listener, app).await });

        let (mut client, _) =
            tokio_tungstenite::connect_async(format!("ws://{address}/ws")).await?;
        let opened = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward the open event");
        let connection_id = match opened {
            NetworkEvent::ConnectionOpened {
                connection:
                    ConnectionInfo {
                        id,
                        role: ConnectionRole::Frontend,
                        transport: TransportKind::WebSocket,
                    },
            } => id,
            other => panic!("unexpected event: {other:?}"),
        };

        client
            .send(TungsteniteMessage::Text(serde_json::to_string(
                &Frontend2BackendMsg::Ping,
            )?))
            .await?;
        let incoming = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward the frontend event");
        assert!(matches!(
            incoming,
            NetworkEvent::FrontendMessage {
                connection_id: source,
                message: Frontend2BackendMsg::Ping,
            } if source == connection_id
        ));

        network
            .send_command(NetworkCommand::SendFrontend {
                connection_id,
                message: Backend2FrontendMsg::Pong,
            })
            .await?;
        let response = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("WebSocket should remain open")?;
        let TungsteniteMessage::Text(response) = response else {
            panic!("expected a targeted text response");
        };
        assert!(matches!(
            serde_json::from_str::<Backend2FrontendMsg>(&response)?,
            Backend2FrontendMsg::Pong
        ));

        let mismatch = network
            .send_command(NetworkCommand::SendPeer {
                connection_id,
                message: Peer2PeerMsg::Ping,
            })
            .await;
        assert_eq!(
            mismatch,
            Err(NetworkError::ProtocolMismatch {
                connection_id,
                expected: ConnectionRole::Peer,
                actual: ConnectionRole::Frontend,
            })
        );

        network
            .send_command(NetworkCommand::CloseConnection {
                connection_id,
                reason: "supervisor test shutdown".into(),
            })
            .await?;
        let close_frame = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("WebSocket should receive a close frame")?;
        assert!(matches!(close_frame, TungsteniteMessage::Close(_)));

        let closed = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward the close event");
        assert!(matches!(
            closed,
            NetworkEvent::ConnectionClosed {
                connection_id: closed_id,
                reason: ConnectionCloseReason::LocalRequest(reason),
            } if closed_id == connection_id && reason == "supervisor test shutdown"
        ));

        let after_close = network
            .send_command(NetworkCommand::SendFrontend {
                connection_id,
                message: Backend2FrontendMsg::Pong,
            })
            .await;
        assert_eq!(
            after_close,
            Err(NetworkError::ConnectionNotFound(connection_id))
        );

        server_task.abort();
        let _ = server_task.await;
        drop(network);
        tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
        Ok(())
    }
}
