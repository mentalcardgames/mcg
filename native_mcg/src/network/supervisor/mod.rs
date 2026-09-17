mod connections;
mod handle;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use iroh::endpoint::Endpoint;
use tokio::sync::{mpsc, oneshot};
use tokio::task::{JoinHandle, JoinSet};

use self::connections::{preferred_direction, ManagedConnection, ManagedTarget};
pub use self::handle::NetworkHandle;
use iroh_tickets::endpoint::EndpointTicket;

use self::handle::{IrohConnectResult, SupervisorRequest};
use crate::config::Config;
use crate::controller::ControllerHandle;
use crate::network::iroh::{
    build_iroh_endpoint, load_or_generate_iroh_secret, run_iroh_accept_loop,
    run_iroh_listener_task, IrohConnectError, IrohConnector, IrohEndpointConnector,
};
use crate::network::types::ActorEvent;
pub use crate::network::types::NetworkError;
use crate::network::{ConnectionId, NetworkEvent, PeerConnectionDirection, PeerId, TransportKind};

const DEFAULT_CONTROL_CHANNEL_CAPACITY: usize = 256;
const DEFAULT_CONNECTION_CHANNEL_CAPACITY: usize = 64;
const DEFAULT_IROH_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Owns active connection handles and routes commands and actor events.
pub struct NetworkSupervisor {
    /// Weak sender for obtaining handles to this supervisor without preventing shutdown.
    weak_request_tx: mpsc::WeakSender<SupervisorRequest>,
    /// Output for connection actors to report internal events.
    pub(crate) actor_event_tx: mpsc::Sender<ActorEvent>,
    /// Input of internal connection actor events.
    actor_event_rx: mpsc::Receiver<ActorEvent>,
    /// Controller handle for forwarding network events.
    pub(crate) controller: ControllerHandle,
    /// Requests from NetworkHandles.
    request_rx: mpsc::Receiver<SupervisorRequest>,
    /// Completed outgoing Iroh connection attempts.
    iroh_connect_result_tx: mpsc::Sender<IrohConnectResult>,
    iroh_connect_result_rx: mpsc::Receiver<IrohConnectResult>,
    /// Endpoint-backed connector for outgoing Iroh connections.
    iroh_connector: Option<Arc<dyn IrohConnector>>,
    /// Active listener shutdown sender, signaled on supervisor shutdown or explicit stop.
    iroh_listener_shutdown_tx: Option<oneshot::Sender<()>>,
    /// Receiver awaiting clean completion of listener task.
    iroh_listener_stopped_rx: Option<oneshot::Receiver<()>>,
    /// Active Axum HTTP/WebSocket listener shutdown sender.
    axum_listener_shutdown_tx: Option<oneshot::Sender<()>>,
    /// Receiver awaiting clean completion of Axum listener task.
    axum_listener_stopped_rx: Option<oneshot::Receiver<()>>,
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

/// Builder for configuring and instantiating a [`NetworkSupervisor`] and [`NetworkHandle`].
pub struct NetworkBuilder {
    controller: ControllerHandle,
    control_channel_capacity: usize,
    connection_channel_capacity: usize,
    iroh_connect_timeout: Duration,
    local_peer_id: Option<PeerId>,
    iroh_connector: Option<Arc<dyn IrohConnector>>,
}

impl NetworkBuilder {
    /// Creates a new builder with the target controller handle.
    pub fn new(controller: ControllerHandle) -> Self {
        Self {
            controller,
            control_channel_capacity: DEFAULT_CONTROL_CHANNEL_CAPACITY,
            connection_channel_capacity: DEFAULT_CONNECTION_CHANNEL_CAPACITY,
            iroh_connect_timeout: DEFAULT_IROH_CONNECT_TIMEOUT,
            local_peer_id: None,
            iroh_connector: None,
        }
    }

    /// Sets the buffer capacity for the internal supervisor control channel.
    pub fn with_control_channel_capacity(mut self, capacity: usize) -> Self {
        self.control_channel_capacity = capacity;
        self
    }

    /// Sets the buffer capacity for per-connection frame channels.
    pub fn with_connection_channel_capacity(mut self, capacity: usize) -> Self {
        self.connection_channel_capacity = capacity;
        self
    }

    /// Sets the timeout for outgoing Iroh connection establishment.
    pub fn with_iroh_connect_timeout(mut self, timeout: Duration) -> Self {
        self.iroh_connect_timeout = timeout;
        self
    }

    /// Sets the local peer identity for deduplication and testing.
    pub fn with_local_peer_id(mut self, peer_id: PeerId) -> Self {
        self.local_peer_id = Some(peer_id);
        self
    }

    /// Sets the local peer identity for deduplication and testing (mutable reference helper).
    pub fn set_local_peer_id(&mut self, peer_id: PeerId) -> &mut Self {
        self.local_peer_id = Some(peer_id);
        self
    }

    /// Sets the timeout for outgoing Iroh connection establishment (mutable reference helper).
    pub fn set_iroh_connect_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.iroh_connect_timeout = timeout;
        self
    }

    /// Sets a custom connector for Iroh endpoint connections.
    #[allow(dead_code)]
    pub(crate) fn with_iroh_connector(mut self, connector: Arc<dyn IrohConnector>) -> Self {
        self.iroh_connector = Some(connector);
        self
    }

    /// Builds the [`NetworkSupervisor`] and [`NetworkHandle`] without starting a background task.
    pub fn build(self) -> (NetworkSupervisor, NetworkHandle) {
        let (request_tx, request_rx) = mpsc::channel(self.control_channel_capacity);
        let (actor_event_tx, actor_event_rx) = mpsc::channel(self.control_channel_capacity);
        let (iroh_connect_result_tx, iroh_connect_result_rx) =
            mpsc::channel(self.control_channel_capacity);

        let weak_request_tx = request_tx.downgrade();
        let handle = NetworkHandle::new(request_tx);

        let supervisor = NetworkSupervisor {
            weak_request_tx,
            actor_event_tx,
            actor_event_rx,
            controller: self.controller,
            request_rx,
            iroh_connect_result_tx,
            iroh_connect_result_rx,
            iroh_connector: self.iroh_connector,
            iroh_listener_shutdown_tx: None,
            iroh_listener_stopped_rx: None,
            axum_listener_shutdown_tx: None,
            axum_listener_stopped_rx: None,
            tasks: JoinSet::new(),
            connections: HashMap::new(),
            next_connection_id: 0,
            connection_channel_capacity: self.connection_channel_capacity,
            iroh_connect_timeout: self.iroh_connect_timeout,
            local_peer_id: self.local_peer_id,
            peers: HashMap::new(),
            pending_outgoing: HashSet::new(),
        };

        (supervisor, handle)
    }

    /// Spawns the supervisor as an asynchronous Tokio task.
    ///
    /// Returns the [`NetworkHandle`] for interacting with the supervisor and the task's [`JoinHandle`].
    pub fn spawn(self) -> (NetworkHandle, JoinHandle<()>) {
        let (supervisor, handle) = self.build();
        let task = tokio::spawn(supervisor.run());
        (handle, task)
    }
}

impl NetworkSupervisor {
    /// Returns a [`NetworkHandle`] for interacting with this supervisor.
    pub fn handle(&self) -> NetworkHandle {
        let tx = self
            .weak_request_tx
            .upgrade()
            .expect("network supervisor handle cannot be upgraded because channel closed");
        NetworkHandle::new(tx)
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
                        if self.controller.send_network_event(event).await.is_err() {
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
                    if let Some(shutdown_tx) = &self.iroh_listener_shutdown_tx {
                        if shutdown_tx.is_closed() {
                            tracing::warn!("supervised Iroh listener task terminated");
                            self.iroh_listener_shutdown_tx = None;
                            self.iroh_listener_stopped_rx = None;
                            self.iroh_connector = None;
                        }
                    }
                    if let Some(shutdown_tx) = &self.axum_listener_shutdown_tx {
                        if shutdown_tx.is_closed() {
                            tracing::warn!("supervised Axum listener task terminated");
                            self.axum_listener_shutdown_tx = None;
                            self.axum_listener_stopped_rx = None;
                        }
                    }
                }
            }
        }
        if let Some(shutdown_tx) = self.axum_listener_shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        if let Some(shutdown_tx) = self.iroh_listener_shutdown_tx.take() {
            let _ = shutdown_tx.send(());
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
            SupervisorRequest::StartAxumListener { addr, response_tx } => {
                let result = self.start_axum_listener(addr).await;
                let _ = response_tx.send(result);
            }
            SupervisorRequest::StartIrohListener {
                config,
                config_path,
                response_tx,
            } => {
                let result = self.start_iroh_listener(config, config_path).await;
                let _ = response_tx.send(result);
            }
            SupervisorRequest::StartIrohEndpointListener {
                endpoint,
                response_tx,
            } => {
                let result = self.start_iroh_endpoint_listener(endpoint).await;
                let _ = response_tx.send(result);
            }
            SupervisorRequest::StopListener {
                transport,
                response_tx,
            } => {
                let result = self.stop_listener(transport).await;
                let _ = response_tx.send(result);
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
                    .controller
                    .send_network_event(NetworkEvent::LocalTicketReady(ticket))
                    .await;
                let _ = response_tx.send(send_res.map_err(|_| NetworkError::SupervisorStopped));
            }
        }
        true
    }

    /// Starts an Iroh QUIC listener supervised by this supervisor.
    pub async fn start_iroh_listener(
        &mut self,
        config: Config,
        config_path: Option<PathBuf>,
    ) -> Result<(), NetworkError> {
        if self.iroh_connector.is_some() || self.iroh_listener_shutdown_tx.is_some() {
            return Err(NetworkError::ListenerAlreadyRunning(TransportKind::Iroh));
        }

        let secret_key = load_or_generate_iroh_secret(&config, config_path.as_deref()).await;
        let endpoint = build_iroh_endpoint(secret_key).await.map_err(|error| {
            NetworkError::ListenerSetupFailed {
                transport: TransportKind::Iroh,
                message: error.to_string(),
            }
        })?;

        self.configure_iroh_connector(Arc::new(IrohEndpointConnector::new(endpoint.clone())))?;

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (stopped_tx, stopped_rx) = oneshot::channel();
        self.iroh_listener_shutdown_tx = Some(shutdown_tx);
        self.iroh_listener_stopped_rx = Some(stopped_rx);
        let network = self.handle();

        self.tasks.spawn(async move {
            run_iroh_listener_task(
                endpoint,
                config_path,
                network,
                shutdown_rx,
                Some(stopped_tx),
            )
            .await;
        });

        Ok(())
    }

    /// Starts a listener for an already-instantiated Iroh endpoint, supervised by this supervisor.
    pub async fn start_iroh_endpoint_listener(
        &mut self,
        endpoint: Endpoint,
    ) -> Result<(), NetworkError> {
        if self.iroh_connector.is_some() || self.iroh_listener_shutdown_tx.is_some() {
            return Err(NetworkError::ListenerAlreadyRunning(TransportKind::Iroh));
        }

        self.configure_iroh_connector(Arc::new(IrohEndpointConnector::new(endpoint.clone())))?;

        let ticket = EndpointTicket::new(endpoint.addr());
        self.local_peer_id = Some(ticket.endpoint_addr().id);
        let _ = self
            .controller
            .send_network_event(NetworkEvent::LocalTicketReady(ticket))
            .await;

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (stopped_tx, stopped_rx) = oneshot::channel();
        self.iroh_listener_shutdown_tx = Some(shutdown_tx);
        self.iroh_listener_stopped_rx = Some(stopped_rx);
        let network = self.handle();

        self.tasks.spawn(async move {
            run_iroh_accept_loop(endpoint, network, shutdown_rx, Some(stopped_tx)).await;
        });

        Ok(())
    }

    /// Starts an Axum HTTP/WebSocket listener on the given address, supervised by this supervisor.
    pub async fn start_axum_listener(
        &mut self,
        addr: SocketAddr,
    ) -> Result<SocketAddr, NetworkError> {
        if self.axum_listener_shutdown_tx.is_some() {
            return Err(NetworkError::ListenerAlreadyRunning(
                TransportKind::WebSocket,
            ));
        }

        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|error| {
            NetworkError::ListenerSetupFailed {
                transport: TransportKind::WebSocket,
                message: error.to_string(),
            }
        })?;

        let bound_addr =
            listener
                .local_addr()
                .map_err(|error| NetworkError::ListenerSetupFailed {
                    transport: TransportKind::WebSocket,
                    message: error.to_string(),
                })?;

        let app = crate::network::build_router(self.handle());

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let (stopped_tx, stopped_rx) = oneshot::channel();
        self.axum_listener_shutdown_tx = Some(shutdown_tx);
        self.axum_listener_stopped_rx = Some(stopped_rx);

        self.tasks.spawn(async move {
            let server = axum::serve(listener, app).with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            });
            if let Err(error) = server.await {
                tracing::error!(%error, "Axum HTTP/WebSocket server error");
            }
            let _ = stopped_tx.send(());
        });

        Ok(bound_addr)
    }

    /// Stops the Axum HTTP/WebSocket listener if one is currently running.
    pub async fn stop_axum_listener(&mut self) -> Result<(), NetworkError> {
        self.stop_listener(TransportKind::WebSocket).await
    }

    /// Stops the Iroh listener if one is currently running.
    pub async fn stop_iroh_listener(&mut self) -> Result<(), NetworkError> {
        self.stop_listener(TransportKind::Iroh).await
    }

    /// Stops the listener for the specified transport kind.
    pub async fn stop_listener(&mut self, transport: TransportKind) -> Result<(), NetworkError> {
        match transport {
            TransportKind::Iroh => {
                let Some(shutdown_tx) = self.iroh_listener_shutdown_tx.take() else {
                    return Err(NetworkError::ListenerNotRunning(TransportKind::Iroh));
                };
                if shutdown_tx.is_closed() {
                    self.iroh_listener_stopped_rx = None;
                    self.iroh_connector = None;
                    return Err(NetworkError::ListenerNotRunning(TransportKind::Iroh));
                }
                let _ = shutdown_tx.send(());
                if let Some(stopped_rx) = self.iroh_listener_stopped_rx.take() {
                    let _ = stopped_rx.await;
                }
                self.iroh_connector = None;
                Ok(())
            }
            TransportKind::WebSocket => {
                let Some(shutdown_tx) = self.axum_listener_shutdown_tx.take() else {
                    return Err(NetworkError::ListenerNotRunning(TransportKind::WebSocket));
                };
                if shutdown_tx.is_closed() {
                    self.axum_listener_stopped_rx = None;
                    return Err(NetworkError::ListenerNotRunning(TransportKind::WebSocket));
                }
                let _ = shutdown_tx.send(());
                if let Some(stopped_rx) = self.axum_listener_stopped_rx.take() {
                    let _ = stopped_rx.await;
                }
                Ok(())
            }
        }
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
