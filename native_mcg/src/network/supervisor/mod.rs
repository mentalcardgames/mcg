mod connections;
mod handle;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;

use self::connections::ManagedConnection;
pub use self::handle::NetworkHandle;
use self::handle::{IrohConnectResult, SupervisorRequest};
use crate::network::iroh::{IrohConnectError, IrohConnector};
use crate::network::types::ActorEvent;
pub use crate::network::types::NetworkError;
use crate::network::{ConnectionId, NetworkEvent, PeerConnectionDirection, TransportKind};

const DEFAULT_CONTROL_CHANNEL_CAPACITY: usize = 256;
const DEFAULT_CONNECTION_CHANNEL_CAPACITY: usize = 64;
const DEFAULT_IROH_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Owns active connection handles and routes commands and actor events.
pub struct NetworkSupervisor {
    /// Output for connection actors to report internal events.
    pub(crate) actor_event_tx: mpsc::Sender<ActorEvent>,
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
    pub(crate) tasks: JoinSet<()>,
    /// Container with all connections with other peers or with frontends.
    pub(crate) connections: HashMap<ConnectionId, ManagedConnection>,
    pub(crate) next_connection_id: u64,
    pub(crate) connection_channel_capacity: usize,
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

    pub fn with_capacities(
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

    pub fn with_settings(
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
        let handle = NetworkHandle::new(request_tx);
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
            SupervisorRequest::EstablishIrohPeerConnection {
                ticket,
                response_tx,
            } => self.start_iroh_connect(ticket, response_tx),
            SupervisorRequest::RegisterFrontendWebSocket {
                socket,
                response_tx,
            } => {
                let result = self.register_frontend_websocket(*socket);
                let _ = response_tx.send(result);
            }
            SupervisorRequest::RegisterIrohPeer {
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
            SupervisorRequest::RegisterIrohFrontend {
                reader,
                writer,
                response_tx,
            } => {
                let result = self.register_iroh_frontend(reader, writer);
                let _ = response_tx.send(result);
            }
            SupervisorRequest::UnicastFrontend {
                connection_id,
                message,
                response_tx,
            } => {
                let result = self.unicast_frontend(connection_id, message);
                let _ = response_tx.send(result);
            }
            SupervisorRequest::UnicastPeer {
                connection_id,
                message,
                response_tx,
            } => {
                let result = self.unicast_peer(connection_id, message);
                let _ = response_tx.send(result);
            }
            SupervisorRequest::BroadcastFrontend {
                message,
                response_tx,
            } => {
                self.broadcast_frontend(message);
                let _ = response_tx.send(Ok(()));
            }
            SupervisorRequest::BroadcastPeer {
                message,
                response_tx,
            } => {
                self.broadcast_peer(message);
                let _ = response_tx.send(Ok(()));
            }
            SupervisorRequest::CloseConnection {
                connection_id,
                reason,
                response_tx,
            } => {
                let result = self.close_connection(connection_id, reason);
                let _ = response_tx.send(result);
            }
        }
        true
    }

    pub(crate) fn configure_iroh_connector(
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

    fn handle_actor_event(&mut self, event: ActorEvent) -> Option<NetworkEvent> {
        match event {
            ActorEvent::Ready { connection_id } => {
                let Some(connection) = self.connections.get(&connection_id) else {
                    tracing::warn!(%connection_id, "ready event belongs to an unknown connection");
                    return None;
                };
                connection.connected_event(connection_id)
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
