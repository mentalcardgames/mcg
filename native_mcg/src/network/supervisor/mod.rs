mod connections;
mod handle;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};
use tokio::task::{JoinHandle, JoinSet};

use self::connections::{preferred_direction, ManagedConnection, ManagedTarget};
pub use self::handle::NetworkHandle;
use iroh_tickets::endpoint::EndpointTicket;

use self::handle::{IrohConnectResult, SupervisorRequest};
use crate::controller::ControllerEvent;
use crate::network::iroh::{IrohConnectError, IrohConnector};
use crate::network::types::ActorEvent;
pub use crate::network::types::NetworkError;
use crate::network::{ConnectionId, NetworkEvent, PeerConnectionDirection, PeerId, TransportKind};

const DEFAULT_CONTROL_CHANNEL_CAPACITY: usize = 256;
const DEFAULT_CONNECTION_CHANNEL_CAPACITY: usize = 64;
const DEFAULT_IROH_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Owns active connection handles and routes commands and actor events.
pub struct NetworkSupervisor {
    /// Sender for supervisor requests, transferred to `NetworkHandle` when the supervisor task is started.
    request_tx: Option<mpsc::Sender<SupervisorRequest>>,
    /// Output for connection actors to report internal events.
    pub(crate) actor_event_tx: mpsc::Sender<ActorEvent>,
    /// Input of internal connection actor events.
    actor_event_rx: mpsc::Receiver<ActorEvent>,
    /// Output for forwarding NetworkEvents from actors towards Controller.
    application_event_tx: mpsc::Sender<ControllerEvent>,
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
    pub(crate) local_peer_id: Option<PeerId>,
    pub(crate) peers: HashMap<PeerId, ConnectionId>,
    pub(crate) pending_outgoing: HashSet<PeerId>,
}

impl NetworkSupervisor {
    /// Creates a supervisor with the provided application event sender.
    pub fn new(application_event_tx: mpsc::Sender<ControllerEvent>) -> Self {
        let (request_tx, request_rx) = mpsc::channel(DEFAULT_CONTROL_CHANNEL_CAPACITY);
        let (actor_event_tx, actor_event_rx) = mpsc::channel(DEFAULT_CONTROL_CHANNEL_CAPACITY);
        let (iroh_connect_result_tx, iroh_connect_result_rx) =
            mpsc::channel(DEFAULT_CONTROL_CHANNEL_CAPACITY);
        Self {
            request_tx: Some(request_tx),
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
            connection_channel_capacity: DEFAULT_CONNECTION_CHANNEL_CAPACITY,
            iroh_connect_timeout: DEFAULT_IROH_CONNECT_TIMEOUT,
            local_peer_id: None,
            peers: HashMap::new(),
            pending_outgoing: HashSet::new(),
        }
    }

    /// Explicitly sets the local peer identity for deduplication and testing.
    pub fn set_local_peer_id(&mut self, peer_id: PeerId) {
        self.local_peer_id = Some(peer_id);
    }

    /// Spawns the supervisor as an asynchronous Tokio task.
    ///
    /// Returns the [`NetworkHandle`] for interacting with the supervisor and the task's [`JoinHandle`].
    pub fn start(mut self) -> (NetworkHandle, JoinHandle<()>) {
        let request_tx = self
            .request_tx
            .take()
            .expect("network supervisor request sender already taken");
        let handle = NetworkHandle::new(request_tx);
        let task = tokio::spawn(self.run());
        (handle, task)
    }

    #[cfg(test)]
    pub fn set_iroh_connect_timeout(&mut self, timeout: Duration) {
        self.iroh_connect_timeout = timeout;
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
                    if !self.handle_request(request).await {
                        break;
                    }
                }
                // Enrich internal actor events with supervisor-owned metadata.
                event = self.actor_event_rx.recv() => {
                    let Some(event) = event else {
                        break;
                    };
                    if let Some(event) = self.handle_actor_event(event) {
                        if self
                            .application_event_tx
                            .send(ControllerEvent::Network(event))
                            .await
                            .is_err()
                        {
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

    async fn handle_request(&mut self, request: SupervisorRequest) -> bool {
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
            SupervisorRequest::PublishLocalTicket {
                ticket,
                response_tx,
            } => {
                self.local_peer_id = Some(ticket.endpoint_addr().id);
                let send_res = self
                    .application_event_tx
                    .send(ControllerEvent::Network(NetworkEvent::LocalTicketReady(
                        ticket,
                    )))
                    .await;
                let _ = response_tx.send(send_res.map_err(|_| NetworkError::SupervisorStopped));
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
        if let Some(peer_id) = connector.local_peer_id() {
            self.local_peer_id = Some(peer_id);
        }
        self.iroh_connector = Some(connector);
        Ok(())
    }

    fn start_iroh_connect(
        &mut self,
        ticket: EndpointTicket,
        response_tx: oneshot::Sender<Result<ConnectionId, NetworkError>>,
    ) {
        let peer_id = ticket.endpoint_addr().id;
        if let Some(local_id) = self.local_peer_id {
            if local_id == peer_id {
                let _ = response_tx.send(Err(NetworkError::LocalEndpoint(peer_id)));
                return;
            }
        }
        if self.pending_outgoing.contains(&peer_id) || self.peers.contains_key(&peer_id) {
            let _ = response_tx.send(Err(NetworkError::DuplicatePeer(peer_id)));
            return;
        }

        let Some(connector) = self.iroh_connector.clone() else {
            let _ = response_tx.send(Err(NetworkError::TransportUnavailable(TransportKind::Iroh)));
            return;
        };
        self.pending_outgoing.insert(peer_id);
        let result_tx = self.iroh_connect_result_tx.clone();
        let timeout = self.iroh_connect_timeout;

        self.tasks.spawn(async move {
            let result = match tokio::time::timeout(timeout, connector.connect(ticket)).await {
                Ok(result) => result,
                Err(_) => {
                    let _ = response_tx.send(Err(NetworkError::ConnectionSetupTimedOut(
                        TransportKind::Iroh,
                    )));
                    let _ = result_tx
                        .send(IrohConnectResult {
                            peer_id,
                            result: Err(IrohConnectError::Connect(
                                "connection setup timed out".into(),
                            )),
                            response_tx: oneshot::channel().0,
                        })
                        .await;
                    return;
                }
            };
            let completion = IrohConnectResult {
                peer_id,
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
        self.pending_outgoing.remove(&result.peer_id);
        let connection = match result.result {
            Ok((peer_id, reader, writer)) => {
                self.register_iroh_peer(peer_id, PeerConnectionDirection::Outgoing, reader, writer)
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
                let Some(connection) = self.connections.get_mut(&connection_id) else {
                    tracing::warn!(%connection_id, "ready event belongs to an unknown connection");
                    return None;
                };
                match &connection.target {
                    ManagedTarget::Frontend { .. } => {
                        connection.reported = true;
                        connection.connected_event(connection_id)
                    }
                    ManagedTarget::Peer {
                        peer_id, direction, ..
                    } => {
                        let peer_id = *peer_id;
                        let direction = *direction;

                        if let Some(local_id) = self.local_peer_id {
                            if local_id == peer_id {
                                tracing::warn!(%peer_id, %connection_id, "closing self-connection to local endpoint");
                                let _ = self.close_connection(
                                    connection_id,
                                    "cannot connect to local endpoint".into(),
                                );
                                return None;
                            }
                        }

                        if let Some(&existing_id) = self.peers.get(&peer_id) {
                            if existing_id == connection_id {
                                return None;
                            }
                            let existing_direction =
                                self.connections.get(&existing_id).and_then(|c| {
                                    if let ManagedTarget::Peer { direction, .. } = c.target {
                                        Some(direction)
                                    } else {
                                        None
                                    }
                                });

                            let preferred = preferred_direction(&self.local_peer_id, &peer_id);
                            let new_is_preferred = existing_direction.is_some_and(|ex_dir| {
                                preferred == Some(direction) && preferred != Some(ex_dir)
                            });

                            if new_is_preferred {
                                tracing::info!(
                                    %peer_id,
                                    winner = %connection_id,
                                    loser = %existing_id,
                                    ?preferred,
                                    "replacing duplicate peer connection with preferred winner"
                                );
                                let _ = self.close_connection(
                                    existing_id,
                                    format!("duplicate peer connection; keeping {connection_id}"),
                                );
                                self.peers.insert(peer_id, connection_id);
                                let connection = self.connections.get_mut(&connection_id)?;
                                connection.reported = true;
                                connection.connected_event(connection_id)
                            } else {
                                tracing::info!(
                                    %peer_id,
                                    winner = %existing_id,
                                    loser = %connection_id,
                                    ?preferred,
                                    "closing duplicate peer connection; keeping winner"
                                );
                                let _ = self.close_connection(
                                    connection_id,
                                    format!("duplicate peer connection; keeping {existing_id}"),
                                );
                                None
                            }
                        } else {
                            self.peers.insert(peer_id, connection_id);
                            let connection = self.connections.get_mut(&connection_id)?;
                            connection.reported = true;
                            connection.connected_event(connection_id)
                        }
                    }
                }
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
                let connection = self.connections.remove(&connection_id);
                let was_reported = connection.as_ref().is_some_and(|c| c.reported);
                if let Some(connection) = &connection {
                    if let ManagedTarget::Peer { peer_id, .. } = &connection.target {
                        if self.peers.get(peer_id) == Some(&connection_id) {
                            self.peers.remove(peer_id);
                        }
                    }
                }
                if was_reported {
                    Some(NetworkEvent::ConnectionClosed {
                        connection_id,
                        reason,
                    })
                } else {
                    None
                }
            }
        }
    }
}
