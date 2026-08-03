use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::WebSocket;
use iroh::endpoint::Endpoint;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;

use super::iroh::{
    run_iroh_peer_actor, IrohConnectError, IrohConnector, IrohEndpointConnector, PeerReader,
    PeerWriter,
};
use super::types::ActorEvent;
use super::websocket::run_websocket_actor;
use super::{
    ConnectionId, FrontendConnectionCommand, NetworkCommand, NetworkEvent, PeerConnectionCommand,
    PeerConnectionDirection, PeerId, ProtocolRole, TransportKind,
};

const DEFAULT_CONTROL_CHANNEL_CAPACITY: usize = 256;
const DEFAULT_CONNECTION_CHANNEL_CAPACITY: usize = 64;
const DEFAULT_IROH_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

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

/// Cloneable interface used by transports and application logic.
#[derive(Clone)]
pub struct NetworkHandle {
    request_tx: mpsc::Sender<SupervisorRequest>,
}

impl NetworkHandle {
    /// Requests an orderly shutdown of the supervisor and all owned child tasks.
    pub(crate) async fn shutdown(&self) -> Result<(), NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::Shutdown { response_tx })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)
    }

    /// Configures the Iroh endpoint used for outgoing peer connections.
    pub async fn configure_iroh_endpoint(&self, endpoint: Endpoint) -> Result<(), NetworkError> {
        self.configure_iroh_connector(Arc::new(IrohEndpointConnector::new(endpoint)))
            .await
    }

    async fn configure_iroh_connector(
        &self,
        connector: Arc<dyn IrohConnector>,
    ) -> Result<(), NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::ConfigureIroh {
                connector,
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

    /// Establishes and registers an outgoing Iroh peer connection.
    pub async fn connect_iroh_peer(
        &self,
        ticket: impl Into<String>,
    ) -> Result<ConnectionId, NetworkError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::ConnectIrohPeer {
                ticket: ticket.into(),
                response_tx,
            })
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?;
        response_rx
            .await
            .map_err(|_| NetworkError::SupervisorStopped)?
    }

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

    /// Registers an established Iroh peer stream with the supervisor.
    pub async fn register_incoming_iroh_peer<R, W>(
        &self,
        peer_id: PeerId,
        reader: R,
        writer: W,
    ) -> Result<ConnectionId, NetworkError>
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (response_tx, response_rx) = oneshot::channel();
        self.request_tx
            .send(SupervisorRequest::RegisterIncomingIrohPeer {
                peer_id,
                reader: Box::new(reader),
                writer: Box::new(writer),
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
    /// Output for connection actors to report internal events.
    actor_event_tx: mpsc::Sender<ActorEvent>,
    /// Input of internal connection actor events.
    actor_event_rx: mpsc::Receiver<ActorEvent>,
    /// Output for forwarding NetworkEvents from actors towards Controller.
    application_event_tx: mpsc::Sender<NetworkEvent>,
    /// Requests from NetworkHandles.
    request_rx: mpsc::Receiver<SupervisorRequest>,
    /// Completed outgoing Iroh connection attempts.
    iroh_connect_result_tx: mpsc::Sender<IrohConnectResult>,
    iroh_connect_result_rx: mpsc::Receiver<IrohConnectResult>,
    /// Endpoint-backed connector for outgoing Iroh connections.
    iroh_connector: Option<Arc<dyn IrohConnector>>,
    /// Connection actors and in-progress outgoing connection attempts.
    tasks: JoinSet<()>,
    /// Container with all connections with other peers or with frontends.
    connections: HashMap<ConnectionId, ManagedConnection>,
    next_connection_id: u64,
    connection_channel_capacity: usize,
    iroh_connect_timeout: Duration,
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
        Self::with_settings(
            application_event_tx,
            control_channel_capacity,
            connection_channel_capacity,
            DEFAULT_IROH_CONNECT_TIMEOUT,
        )
    }

    fn with_settings(
        application_event_tx: mpsc::Sender<NetworkEvent>,
        control_channel_capacity: usize,
        connection_channel_capacity: usize,
        iroh_connect_timeout: Duration,
    ) -> (Self, NetworkHandle) {
        assert!(control_channel_capacity > 0);
        assert!(connection_channel_capacity > 0);
        assert!(!iroh_connect_timeout.is_zero());

        let (request_tx, request_rx) = mpsc::channel(control_channel_capacity);
        let (actor_event_tx, actor_event_rx) = mpsc::channel(control_channel_capacity);
        let (iroh_connect_result_tx, iroh_connect_result_rx) =
            mpsc::channel(control_channel_capacity);
        let handle = NetworkHandle { request_tx };
        let supervisor = Self {
            request_rx,
            actor_event_tx,
            actor_event_rx,
            iroh_connect_result_tx,
            iroh_connect_result_rx,
            iroh_connector: None,
            tasks: JoinSet::new(),
            application_event_tx,
            connections: HashMap::new(),
            next_connection_id: 0,
            connection_channel_capacity,
            iroh_connect_timeout,
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
                    if !self.handle_request(request) {
                        break;
                    }
                }
                // Enrich internal actor events with supervisor-owned metadata.
                event = self.actor_event_rx.recv() => {
                    let Some(event) = event else {
                        break;
                    };
                    if let Some(event) = self.handle_actor_event(event) {
                        if self.application_event_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                }
                result = self.iroh_connect_result_rx.recv() => {
                    let Some(result) = result else {
                        break;
                    };
                    self.handle_iroh_connect_result(result);
                }
                task = self.tasks.join_next(), if !self.tasks.is_empty() => {
                    if let Some(Err(error)) = task {
                        tracing::error!(%error, "network child task failed");
                    }
                }
            }
        }
        self.tasks.shutdown().await;
        tracing::info!(
            connections = self.connections.len(),
            "network supervisor stopped"
        );
    }

    fn handle_request(&mut self, request: SupervisorRequest) -> bool {
        match request {
            SupervisorRequest::Shutdown { response_tx } => {
                let _ = response_tx.send(());
                return false;
            }
            SupervisorRequest::ConfigureIroh {
                connector,
                response_tx,
            } => {
                let result = self.configure_iroh_connector(connector);
                let _ = response_tx.send(result);
            }
            SupervisorRequest::ConnectIrohPeer {
                ticket,
                response_tx,
            } => self.start_iroh_connect(ticket, response_tx),
            SupervisorRequest::RegisterWebSocket {
                socket,
                response_tx,
            } => {
                let result = self.register_websocket(*socket);
                let _ = response_tx.send(result);
            }
            SupervisorRequest::RegisterIncomingIrohPeer {
                peer_id,
                reader,
                writer,
                response_tx,
            } => {
                let result = self.register_iroh_peer(
                    peer_id,
                    PeerConnectionDirection::Incoming,
                    reader,
                    writer,
                );
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
        true
    }

    fn configure_iroh_connector(
        &mut self,
        connector: Arc<dyn IrohConnector>,
    ) -> Result<(), NetworkError> {
        if self.iroh_connector.is_some() {
            return Err(NetworkError::TransportAlreadyConfigured(
                TransportKind::Iroh,
            ));
        }
        self.iroh_connector = Some(connector);
        Ok(())
    }

    fn start_iroh_connect(
        &mut self,
        ticket: String,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    ) {
        let Some(connector) = self.iroh_connector.clone() else {
            let _ = response_tx.send(Err(NetworkError::TransportUnavailable(TransportKind::Iroh)));
            return;
        };
        let result_tx = self.iroh_connect_result_tx.clone();
        let timeout = self.iroh_connect_timeout;

        self.tasks.spawn(async move {
            let result = match tokio::time::timeout(timeout, connector.connect(ticket)).await {
                Ok(result) => result,
                Err(_) => {
                    let _ = response_tx.send(Err(NetworkError::ConnectionSetupTimedOut(
                        TransportKind::Iroh,
                    )));
                    return;
                }
            };
            let completion = IrohConnectResult {
                result,
                response_tx,
            };
            if let Err(error) = result_tx.send(completion).await {
                let _ = error
                    .0
                    .response_tx
                    .send(Err(NetworkError::SupervisorStopped));
            }
        });
    }

    fn handle_iroh_connect_result(&mut self, result: IrohConnectResult) {
        let connection = match result.result {
            Ok((peer_id, reader, writer)) => {
                self.register_iroh_peer(peer_id, PeerConnectionDirection::Outgoing, reader, writer)
            }
            Err(IrohConnectError::InvalidTicket(message)) => {
                Err(NetworkError::InvalidPeerTicket(message))
            }
            Err(IrohConnectError::Connect(message)) => Err(NetworkError::ConnectionSetupFailed {
                transport: TransportKind::Iroh,
                message,
            }),
            Err(IrohConnectError::OpenStream(message)) => {
                Err(NetworkError::ConnectionSetupFailed {
                    transport: TransportKind::Iroh,
                    message: format!("opening bidirectional stream: {message}"),
                })
            }
        };
        let _ = result.response_tx.send(connection);
    }

    fn register_websocket(&mut self, socket: WebSocket) -> Result<ConnectionId, NetworkError> {
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
        self.tasks.spawn(run_websocket_actor(
            connection_id,
            socket,
            actor_event_tx,
            command_rx,
        ));

        Ok(connection_id)
    }

    fn register_iroh_peer(
        &mut self,
        peer_id: PeerId,
        direction: PeerConnectionDirection,
        reader: PeerReader,
        writer: PeerWriter,
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

    fn send_frontend(
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
            ManagedTarget::Peer { .. } => {
                return Err(NetworkError::ProtocolMismatch {
                    connection_id,
                    expected: ProtocolRole::Frontend,
                    actual: connection.role(),
                });
            }
        };

        self.try_send(connection_id, command_tx, command)
    }

    fn send_peer(
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
        };

        self.try_send(connection_id, command_tx, command)
    }

    fn close_connection(
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
        }
    }

    fn try_send<T>(
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

    fn handle_actor_event(&mut self, event: ActorEvent) -> Option<NetworkEvent> {
        match event {
            ActorEvent::Ready { connection_id } => {
                let Some(connection) = self.connections.get(&connection_id) else {
                    tracing::warn!(%connection_id, "ready event belongs to an unknown connection");
                    return None;
                };
                Some(connection.connected_event(connection_id))
            }
            ActorEvent::FrontendMessage {
                connection_id,
                message,
            } => Some(NetworkEvent::FrontendMessage {
                connection_id,
                message,
            }),
            ActorEvent::PeerMessage {
                connection_id,
                message,
            } => Some(NetworkEvent::PeerMessage {
                connection_id,
                message,
            }),
            ActorEvent::Closed {
                connection_id,
                reason,
            } => {
                self.connections.remove(&connection_id);
                Some(NetworkEvent::ConnectionClosed {
                    connection_id,
                    reason,
                })
            }
        }
    }
}

/// Type that represents the internal contract between NetworkSupervisor and NetworkHandle for message calls on the handle.
enum SupervisorRequest {
    Shutdown {
        response_tx: oneshot::Sender<()>,
    },
    ConfigureIroh {
        connector: Arc<dyn IrohConnector>,
        response_tx: oneshot::Sender<Result<(), NetworkError>>,
    },
    ConnectIrohPeer {
        ticket: String,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    RegisterWebSocket {
        socket: Box<WebSocket>,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    RegisterIncomingIrohPeer {
        peer_id: PeerId,
        reader: Box<dyn AsyncRead + Unpin + Send>,
        writer: Box<dyn AsyncWrite + Unpin + Send>,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    },
    Execute {
        command: NetworkCommand,
        response_tx: oneshot::Sender<Result<(), NetworkError>>,
    },
}

struct IrohConnectResult {
    result: Result<(PeerId, PeerReader, PeerWriter), IrohConnectError>,
    response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
}

#[derive(Clone)]
struct ManagedConnection {
    transport: TransportKind,
    target: ManagedTarget,
}

#[derive(Clone)]
enum ManagedTarget {
    Frontend {
        command_tx: mpsc::Sender<FrontendConnectionCommand>,
    },
    Peer {
        peer_id: PeerId,
        direction: PeerConnectionDirection,
        command_tx: mpsc::Sender<PeerConnectionCommand>,
    },
}

impl ManagedConnection {
    fn role(&self) -> ProtocolRole {
        match &self.target {
            ManagedTarget::Frontend { .. } => ProtocolRole::Frontend,
            ManagedTarget::Peer { .. } => ProtocolRole::Peer,
        }
    }

    fn connected_event(&self, connection_id: ConnectionId) -> NetworkEvent {
        match &self.target {
            ManagedTarget::Frontend { .. } => NetworkEvent::FrontendConnected {
                connection_id,
                transport: self.transport,
            },
            ManagedTarget::Peer {
                peer_id, direction, ..
            } => NetworkEvent::PeerConnected {
                connection_id,
                peer_id: peer_id.clone(),
                transport: self.transport,
                direction: *direction,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Result;
    use async_trait::async_trait;
    use axum::{
        extract::{ws::WebSocketUpgrade, State},
        response::IntoResponse,
        routing::get,
        Router,
    };
    use futures::{SinkExt, StreamExt};
    use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, Peer2PeerMsg};
    use tokio::io::{duplex, split, AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::sync::Mutex;
    use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

    use super::*;
    use crate::network::{ConnectionCloseReason, TransportKind};

    struct OneShotIrohConnector {
        peer_id: PeerId,
        stream: Mutex<Option<(PeerReader, PeerWriter)>>,
    }

    struct PendingIrohConnector;

    struct SignalingPendingIrohConnector {
        started_tx: Mutex<Option<oneshot::Sender<()>>>,
    }

    #[async_trait]
    impl IrohConnector for OneShotIrohConnector {
        async fn connect(
            &self,
            _ticket: String,
        ) -> Result<(PeerId, PeerReader, PeerWriter), IrohConnectError> {
            let (reader, writer) =
                self.stream.lock().await.take().ok_or_else(|| {
                    IrohConnectError::Connect("test stream already consumed".into())
                })?;
            Ok((self.peer_id.clone(), reader, writer))
        }
    }

    #[async_trait]
    impl IrohConnector for PendingIrohConnector {
        async fn connect(
            &self,
            _ticket: String,
        ) -> Result<(PeerId, PeerReader, PeerWriter), IrohConnectError> {
            std::future::pending().await
        }
    }

    #[async_trait]
    impl IrohConnector for SignalingPendingIrohConnector {
        async fn connect(
            &self,
            _ticket: String,
        ) -> Result<(PeerId, PeerReader, PeerWriter), IrohConnectError> {
            if let Some(started_tx) = self.started_tx.lock().await.take() {
                let _ = started_tx.send(());
            }
            std::future::pending().await
        }
    }

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
            NetworkEvent::FrontendConnected {
                connection_id,
                transport: TransportKind::WebSocket,
            } => connection_id,
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
                expected: ProtocolRole::Peer,
                actual: ProtocolRole::Frontend,
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
        network.shutdown().await?;
        tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
        Ok(())
    }

    #[tokio::test]
    async fn supervisor_registers_routes_closes_and_removes_iroh_peer() -> Result<()> {
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let (actor_stream, remote_stream) = duplex(4096);
        let (actor_reader, actor_writer) = split(actor_stream);
        let (remote_reader, mut remote_writer) = split(remote_stream);
        let mut remote_reader = BufReader::new(remote_reader);
        let peer_id = PeerId::new("test-peer-supervisor");

        let unavailable = network.connect_iroh_peer("test-ticket").await;
        assert_eq!(
            unavailable,
            Err(NetworkError::TransportUnavailable(TransportKind::Iroh))
        );
        network
            .configure_iroh_connector(Arc::new(OneShotIrohConnector {
                peer_id: peer_id.clone(),
                stream: Mutex::new(Some((Box::new(actor_reader), Box::new(actor_writer)))),
            }))
            .await?;
        let duplicate_configuration = network
            .configure_iroh_connector(Arc::new(OneShotIrohConnector {
                peer_id: PeerId::new("unused-test-peer"),
                stream: Mutex::new(None),
            }))
            .await;
        assert_eq!(
            duplicate_configuration,
            Err(NetworkError::TransportAlreadyConfigured(
                TransportKind::Iroh
            ))
        );
        let connection_id = network.connect_iroh_peer("test-ticket").await?;
        let opened = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward the open event");
        assert!(matches!(
            opened,
            NetworkEvent::PeerConnected {
                connection_id: opened_id,
                peer_id: opened_peer_id,
                transport: TransportKind::Iroh,
                direction: PeerConnectionDirection::Outgoing,
            } if opened_id == connection_id && opened_peer_id == peer_id
        ));

        remote_writer
            .write_all(format!("{}\n", serde_json::to_string(&Peer2PeerMsg::Ping)?).as_bytes())
            .await?;
        let incoming = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward the peer event");
        assert!(matches!(
            incoming,
            NetworkEvent::PeerMessage {
                connection_id: source,
                message: Peer2PeerMsg::Ping,
            } if source == connection_id
        ));

        network
            .send_command(NetworkCommand::SendPeer {
                connection_id,
                message: Peer2PeerMsg::Pong,
            })
            .await?;
        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::Pong
        ));

        let mismatch = network
            .send_command(NetworkCommand::SendFrontend {
                connection_id,
                message: Backend2FrontendMsg::Pong,
            })
            .await;
        assert_eq!(
            mismatch,
            Err(NetworkError::ProtocolMismatch {
                connection_id,
                expected: ProtocolRole::Frontend,
                actual: ProtocolRole::Peer,
            })
        );

        network
            .send_command(NetworkCommand::CloseConnection {
                connection_id,
                reason: "supervisor peer test shutdown".into(),
            })
            .await?;
        let closed = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward the close event");
        assert!(matches!(
            closed,
            NetworkEvent::ConnectionClosed {
                connection_id: closed_id,
                reason: ConnectionCloseReason::LocalRequest(reason),
            } if closed_id == connection_id && reason == "supervisor peer test shutdown"
        ));

        let after_close = network
            .send_command(NetworkCommand::SendPeer {
                connection_id,
                message: Peer2PeerMsg::Ping,
            })
            .await;
        assert_eq!(
            after_close,
            Err(NetworkError::ConnectionNotFound(connection_id))
        );

        drop(network);
        tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
        Ok(())
    }

    #[tokio::test]
    async fn supervisor_times_out_pending_iroh_connections() -> Result<()> {
        let (event_tx, _event_rx) = mpsc::channel(16);
        let (supervisor, network) =
            NetworkSupervisor::with_settings(event_tx, 16, 8, Duration::from_millis(10));
        let supervisor_task = tokio::spawn(supervisor.run());
        network
            .configure_iroh_connector(Arc::new(PendingIrohConnector))
            .await?;

        assert_eq!(
            network.connect_iroh_peer("test-ticket").await,
            Err(NetworkError::ConnectionSetupTimedOut(TransportKind::Iroh))
        );

        network.shutdown().await?;
        tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
        Ok(())
    }

    #[tokio::test]
    async fn supervisor_shutdown_cancels_pending_iroh_connections() -> Result<()> {
        let (event_tx, _event_rx) = mpsc::channel(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let (started_tx, started_rx) = oneshot::channel();
        network
            .configure_iroh_connector(Arc::new(SignalingPendingIrohConnector {
                started_tx: Mutex::new(Some(started_tx)),
            }))
            .await?;
        let connect_task = tokio::spawn({
            let network = network.clone();
            async move { network.connect_iroh_peer("test-ticket").await }
        });
        tokio::time::timeout(Duration::from_secs(1), started_rx).await??;

        network.shutdown().await?;

        tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
        assert_eq!(connect_task.await?, Err(NetworkError::SupervisorStopped));
        Ok(())
    }
}
