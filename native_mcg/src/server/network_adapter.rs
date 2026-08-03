use std::collections::{HashMap, HashSet};

use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, Peer2PeerMsg};
use tokio::sync::{broadcast, mpsc};

use crate::network::{
    ConnectionId, NetworkCommand, NetworkError, NetworkEvent, NetworkHandle, PeerId,
};

use super::peer_connections::{
    peer_id_from_ticket, EstablishedPeer, PeerConnectionError, PeerConnectionService,
};
use super::state::{current_state_public, dispatch_client_message, AppState, PeerInfo};

const PEER_CONNECT_RESULT_CHANNEL_CAPACITY: usize = 32;

#[derive(Clone, Copy)]
enum PeerConnectOrigin {
    Frontend(ConnectionId),
    Discovery,
}

struct PeerConnectResult {
    origin: PeerConnectOrigin,
    result: Result<EstablishedPeer, PeerConnectionError>,
}

/// Temporary bridge between the actor-based network layer and the legacy
/// lock-based backend state.
pub(super) struct LegacyBackendAdapter {
    state: AppState,
    network: NetworkHandle,
    peer_connections: PeerConnectionService,
    event_rx: mpsc::Receiver<NetworkEvent>,
    broadcast_rx: broadcast::Receiver<Backend2FrontendMsg>,
    peer_broadcast_rx: broadcast::Receiver<Peer2PeerMsg>,
    peer_connect_result_tx: mpsc::Sender<PeerConnectResult>,
    peer_connect_result_rx: mpsc::Receiver<PeerConnectResult>,
    subscribers: HashSet<ConnectionId>,
    peer_ids: HashMap<ConnectionId, PeerId>,
}

impl LegacyBackendAdapter {
    pub(super) fn new(
        state: AppState,
        network: NetworkHandle,
        peer_connections: PeerConnectionService,
        event_rx: mpsc::Receiver<NetworkEvent>,
    ) -> Self {
        let broadcast_rx = state.broadcaster.subscribe();
        let peer_broadcast_rx = state.peer_broadcaster.subscribe();
        let (peer_connect_result_tx, peer_connect_result_rx) =
            mpsc::channel(PEER_CONNECT_RESULT_CHANNEL_CAPACITY);
        Self {
            state,
            network,
            peer_connections,
            event_rx,
            broadcast_rx,
            peer_broadcast_rx,
            peer_connect_result_tx,
            peer_connect_result_rx,
            subscribers: HashSet::new(),
            peer_ids: HashMap::new(),
        }
    }

    pub(super) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.event_rx.recv() => {
                    let Some(event) = event else {
                        break;
                    };
                    if !self.handle_network_event(event).await {
                        break;
                    }
                }
                broadcast = self.broadcast_rx.recv() => {
                    match broadcast {
                        Ok(message) => {
                            if !self.forward_broadcast(message).await {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(skipped, "legacy network adapter missed backend broadcasts");
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                broadcast = self.peer_broadcast_rx.recv() => {
                    match broadcast {
                        Ok(message) => {
                            if !self.forward_peer_broadcast(message).await {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(skipped, "legacy network adapter missed peer broadcasts");
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                result = self.peer_connect_result_rx.recv() => {
                    let Some(result) = result else {
                        break;
                    };
                    if !self.handle_peer_connect_result(result).await {
                        break;
                    }
                }
            }
        }
        tracing::info!("legacy backend network adapter stopped");
    }

    async fn handle_network_event(&mut self, event: NetworkEvent) -> bool {
        match event {
            NetworkEvent::FrontendConnected {
                connection_id,
                transport,
            } => {
                tracing::debug!(
                    %connection_id,
                    ?transport,
                    "frontend connection opened"
                );
                true
            }
            NetworkEvent::PeerConnected {
                connection_id,
                peer_id,
                transport,
                direction,
            } => {
                let winner = self
                    .peer_connections
                    .connection_opened(connection_id, peer_id.clone(), direction)
                    .await;
                if winner == connection_id {
                    tracing::debug!(%connection_id, %peer_id, ?transport, ?direction, "peer connection opened");
                    self.peer_ids.insert(connection_id, peer_id);
                } else {
                    tracing::debug!(%connection_id, %peer_id, %winner, ?transport, ?direction, "duplicate peer connection rejected");
                }
                true
            }
            NetworkEvent::ConnectionClosed {
                connection_id,
                reason,
            } => {
                self.subscribers.remove(&connection_id);
                self.remove_peer_connection(connection_id).await;
                tracing::debug!(%connection_id, ?reason, "network connection closed");
                true
            }
            NetworkEvent::FrontendMessage {
                connection_id,
                message: Frontend2BackendMsg::Subscribe,
            } => self.subscribe(connection_id).await,
            NetworkEvent::FrontendMessage {
                connection_id,
                message: Frontend2BackendMsg::QrValue(ticket),
            } => {
                self.start_peer_connect(ticket, PeerConnectOrigin::Frontend(connection_id))
                    .await
            }
            NetworkEvent::FrontendMessage {
                connection_id,
                message,
            } => {
                let response = dispatch_client_message(&self.state, message).await;
                self.send_frontend(connection_id, response).await
            }
            NetworkEvent::PeerMessage {
                connection_id,
                message,
            } => {
                let Some(peer_id) = self.peer_ids.get(&connection_id).cloned() else {
                    tracing::error!(%connection_id, "peer message has no registered peer identity");
                    return self
                        .close_connection(connection_id, "missing peer identity".into())
                        .await;
                };
                self.handle_peer_message(connection_id, peer_id, message)
                    .await
            }
        }
    }

    async fn subscribe(&mut self, connection_id: ConnectionId) -> bool {
        if !self.subscribers.insert(connection_id) {
            return self
                .send_frontend(
                    connection_id,
                    Backend2FrontendMsg::Error("already subscribed".into()),
                )
                .await;
        }

        if let Some(state) = current_state_public(&self.state).await {
            self.send_frontend(connection_id, Backend2FrontendMsg::UpdatePokerState(state))
                .await
        } else {
            true
        }
    }

    async fn handle_peer_message(
        &mut self,
        connection_id: ConnectionId,
        peer_id: PeerId,
        message: Peer2PeerMsg,
    ) -> bool {
        match message {
            Peer2PeerMsg::Connect(name, ticket) => {
                let (rejection, max_players, game_type) = {
                    let lobby = self.state.lobby.read().await;
                    let peers = self.state.peers.read().await;
                    let rejection = if !lobby.lobby_open {
                        Some("Lobby is closed")
                    } else if lobby.game_running {
                        Some("Game is already running, wait until it finishes and try again")
                    } else if peers.len() >= lobby.max_players {
                        Some("Lobby is full")
                    } else {
                        None
                    };
                    (rejection, lobby.max_players, lobby.game_type.clone())
                };

                if let Some(reason) = rejection {
                    if !self
                        .send_peer(connection_id, Peer2PeerMsg::Reject(reason.into()))
                        .await
                    {
                        return false;
                    }
                    return self
                        .close_connection(connection_id, "peer connection rejected".into())
                        .await;
                }

                let Some(endpoint_id) = Self::parse_peer_endpoint_id(&peer_id) else {
                    return self
                        .close_connection(connection_id, "invalid peer identity".into())
                        .await;
                };

                if !self
                    .send_peer(
                        connection_id,
                        Peer2PeerMsg::LobbyAccept(max_players, game_type),
                    )
                    .await
                {
                    return false;
                }

                let assigned_name = {
                    let peers = self.state.peers.read().await;
                    if peers.values().any(|peer| peer.name == name) {
                        let mut counter = 2;
                        loop {
                            let candidate = format!("{name} {counter}");
                            if !peers.values().any(|peer| peer.name == candidate) {
                                break candidate;
                            }
                            counter += 1;
                        }
                    } else {
                        name.clone()
                    }
                };

                if assigned_name != name
                    && !self
                        .send_peer(connection_id, Peer2PeerMsg::NewName(assigned_name.clone()))
                        .await
                {
                    return false;
                }

                let peers = {
                    self.state
                        .peers
                        .read()
                        .await
                        .iter()
                        .map(|(id, info)| {
                            (id.to_string(), (info.name.clone(), info.ticket.clone()))
                        })
                        .collect()
                };
                if !self
                    .send_peer(connection_id, Peer2PeerMsg::Peers(peers))
                    .await
                {
                    return false;
                }

                self.state.peers.write().await.insert(
                    endpoint_id,
                    PeerInfo {
                        name: assigned_name.clone(),
                        ticket: ticket.unwrap_or_default(),
                    },
                );
                let _ = self
                    .state
                    .broadcaster
                    .send(Backend2FrontendMsg::NewPlayer(assigned_name));
                true
            }
            Peer2PeerMsg::Disconnect(name) => {
                self.remove_peer_from_state(&peer_id).await;
                let _ = self
                    .state
                    .broadcaster
                    .send(Backend2FrontendMsg::RemovePlayer(name));
                self.close_connection(connection_id, "peer requested disconnect".into())
                    .await
            }
            Peer2PeerMsg::Peers(peers) => {
                let mut discovered_tickets = Vec::new();
                {
                    let mut known_peers = self.state.peers.write().await;
                    for (id, (name, ticket)) in peers {
                        let Ok(endpoint_id) = id.parse::<iroh::EndpointId>() else {
                            tracing::warn!(peer_id = %id, "received invalid advertised peer identity");
                            continue;
                        };
                        let Ok(ticket_peer_id) = peer_id_from_ticket(&ticket) else {
                            tracing::warn!(peer_id = %id, "received invalid advertised peer ticket");
                            continue;
                        };
                        if ticket_peer_id.as_str() != id {
                            tracing::warn!(
                                peer_id = %id,
                                ticket_peer_id = %ticket_peer_id,
                                "advertised peer identity does not match its ticket"
                            );
                            continue;
                        }
                        if known_peers.contains_key(&endpoint_id) {
                            continue;
                        }

                        known_peers.insert(
                            endpoint_id,
                            PeerInfo {
                                name: name.clone(),
                                ticket: ticket.clone(),
                            },
                        );
                        let _ = self
                            .state
                            .broadcaster
                            .send(Backend2FrontendMsg::NewPlayer(name));
                        if id != peer_id.as_str() {
                            discovered_tickets.push(ticket);
                        }
                    }
                }
                for ticket in discovered_tickets {
                    if !self
                        .start_peer_connect(ticket, PeerConnectOrigin::Discovery)
                        .await
                    {
                        return false;
                    }
                }
                true
            }
            Peer2PeerMsg::NewName(name) => {
                self.state.lobby.write().await.our_name = name.clone();
                let own_ticket = self.state.ticket.read().await.clone().unwrap_or_default();
                if let Some(own_peer) = self
                    .state
                    .peers
                    .write()
                    .await
                    .values_mut()
                    .find(|peer| peer.ticket == own_ticket)
                {
                    own_peer.name = name.clone();
                }
                let _ = self
                    .state
                    .broadcaster
                    .send(Backend2FrontendMsg::OurName(name));
                true
            }
            Peer2PeerMsg::LobbyAccept(max_players, game_type) => {
                {
                    let mut lobby = self.state.lobby.write().await;
                    lobby.lobby_open = true;
                    lobby.max_players = max_players;
                    lobby.game_type = game_type;
                }
                let _ = self.state.broadcaster.send(Backend2FrontendMsg::Pong);
                true
            }
            Peer2PeerMsg::RequestReady => {
                let message = {
                    let lobby = self.state.lobby.read().await;
                    Peer2PeerMsg::PeerReady(lobby.our_name.clone(), lobby.ready)
                };
                self.send_peer(connection_id, message).await
            }
            Peer2PeerMsg::PeerReady(name, ready) => {
                let _ = self
                    .state
                    .broadcaster
                    .send(Backend2FrontendMsg::PlayerReady(name, ready));
                true
            }
            Peer2PeerMsg::Reject(reason) => {
                let _ = self
                    .state
                    .broadcaster
                    .send(Backend2FrontendMsg::Error(format!(
                        "Peer rejected connection: {reason}"
                    )));
                self.close_connection(connection_id, "peer rejected connection".into())
                    .await
            }
            other => {
                tracing::debug!(%connection_id, %peer_id, ?other, "peer message has no legacy handler");
                true
            }
        }
    }

    fn parse_peer_endpoint_id(peer_id: &PeerId) -> Option<iroh::EndpointId> {
        match peer_id.as_str().parse() {
            Ok(endpoint_id) => Some(endpoint_id),
            Err(error) => {
                tracing::error!(%peer_id, %error, "peer identity is not a valid Iroh endpoint ID");
                None
            }
        }
    }

    async fn remove_peer_from_state(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        let endpoint_id = Self::parse_peer_endpoint_id(peer_id)?;
        self.state.peers.write().await.remove(&endpoint_id)
    }

    async fn remove_peer_connection(&mut self, connection_id: ConnectionId) {
        self.peer_connections.connection_closed(connection_id).await;
        let Some(peer_id) = self.peer_ids.remove(&connection_id) else {
            return;
        };
        if self.peer_ids.values().any(|known| known == &peer_id) {
            return;
        }
        if let Some(peer) = self.remove_peer_from_state(&peer_id).await {
            if !peer.name.is_empty() {
                let _ = self
                    .state
                    .broadcaster
                    .send(Backend2FrontendMsg::RemovePlayer(peer.name));
            }
        }
    }

    async fn start_peer_connect(&mut self, ticket: String, origin: PeerConnectOrigin) -> bool {
        let peer_connections = self.peer_connections.clone();
        let result_tx = self.peer_connect_result_tx.clone();
        tokio::spawn(async move {
            let result = peer_connections.connect(ticket).await;
            let completion = PeerConnectResult { origin, result };
            if result_tx.send(completion).await.is_err() {
                tracing::debug!("network adapter stopped before peer connect completed");
            }
        });
        true
    }

    async fn handle_peer_connect_result(&mut self, result: PeerConnectResult) -> bool {
        match result.result {
            Ok(peer) => {
                tracing::info!(connection_id = %peer.connection_id, peer_id = %peer.peer_id, "outgoing Iroh peer connected and introduced");
                true
            }
            Err(error) => self.report_peer_connect_error(result.origin, error).await,
        }
    }

    async fn report_peer_connect_error(
        &mut self,
        origin: PeerConnectOrigin,
        error: PeerConnectionError,
    ) -> bool {
        tracing::warn!(%error, "failed to establish outgoing Iroh peer connection");
        match origin {
            PeerConnectOrigin::Frontend(connection_id) => {
                self.send_frontend(
                    connection_id,
                    Backend2FrontendMsg::Error(format!("Failed to connect to peer: {error}")),
                )
                .await
            }
            PeerConnectOrigin::Discovery => !matches!(
                error,
                PeerConnectionError::Network(NetworkError::SupervisorStopped)
            ),
        }
    }

    async fn forward_broadcast(&mut self, message: Backend2FrontendMsg) -> bool {
        let subscribers: Vec<_> = self.subscribers.iter().copied().collect();
        for connection_id in subscribers {
            match self
                .network
                .send_command(NetworkCommand::SendFrontend {
                    connection_id,
                    message: message.clone(),
                })
                .await
            {
                Ok(()) => {}
                Err(
                    NetworkError::ConnectionNotFound(_) | NetworkError::ConnectionActorStopped(_),
                ) => {
                    self.subscribers.remove(&connection_id);
                }
                Err(NetworkError::ConnectionBackpressured(_)) => {
                    tracing::warn!(%connection_id, "dropping broadcast for backpressured frontend");
                }
                Err(NetworkError::SupervisorStopped) => return false,
                Err(error) => {
                    tracing::error!(%connection_id, %error, "failed to forward backend broadcast");
                }
            }
        }
        true
    }

    async fn forward_peer_broadcast(&mut self, message: Peer2PeerMsg) -> bool {
        let connections: Vec<_> = self.peer_ids.keys().copied().collect();
        for connection_id in connections {
            if !self.send_peer(connection_id, message.clone()).await {
                return false;
            }
        }
        true
    }

    async fn send_frontend(
        &mut self,
        connection_id: ConnectionId,
        message: Backend2FrontendMsg,
    ) -> bool {
        match self
            .network
            .send_command(NetworkCommand::SendFrontend {
                connection_id,
                message,
            })
            .await
        {
            Ok(()) => true,
            Err(NetworkError::ConnectionNotFound(_) | NetworkError::ConnectionActorStopped(_)) => {
                self.subscribers.remove(&connection_id);
                true
            }
            Err(NetworkError::SupervisorStopped) => false,
            Err(error) => {
                tracing::error!(%connection_id, %error, "failed to send direct frontend response");
                true
            }
        }
    }

    async fn send_peer(&mut self, connection_id: ConnectionId, message: Peer2PeerMsg) -> bool {
        match self
            .network
            .send_command(NetworkCommand::SendPeer {
                connection_id,
                message,
            })
            .await
        {
            Ok(()) => true,
            Err(NetworkError::ConnectionNotFound(_) | NetworkError::ConnectionActorStopped(_)) => {
                self.remove_peer_connection(connection_id).await;
                true
            }
            Err(NetworkError::ConnectionBackpressured(_)) => {
                tracing::warn!(%connection_id, "dropping direct message for backpressured peer");
                true
            }
            Err(NetworkError::SupervisorStopped) => false,
            Err(error) => {
                tracing::error!(%connection_id, %error, "failed to send direct peer message");
                true
            }
        }
    }

    async fn close_connection(&mut self, connection_id: ConnectionId, reason: String) -> bool {
        match self
            .network
            .send_command(NetworkCommand::CloseConnection {
                connection_id,
                reason,
            })
            .await
        {
            Ok(()) => true,
            Err(NetworkError::ConnectionNotFound(_) | NetworkError::ConnectionActorStopped(_)) => {
                self.subscribers.remove(&connection_id);
                self.remove_peer_connection(connection_id).await;
                true
            }
            Err(NetworkError::SupervisorStopped) => false,
            Err(error) => {
                tracing::error!(%connection_id, %error, "failed to close network connection");
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Result;
    use tokio::io::{duplex, split, AsyncBufReadExt, AsyncWriteExt, BufReader};

    use super::*;
    use crate::config::Config;
    use crate::network::NetworkSupervisor;

    #[tokio::test]
    async fn adapter_handles_peer_lobby_messages_through_network_actor() -> Result<()> {
        let state = AppState::new(Config::default(), None);
        {
            let mut lobby = state.lobby.write().await;
            lobby.lobby_open = true;
            lobby.max_players = 3;
            lobby.game_type = "poker".into();
        }
        let mut frontend_events = state.broadcaster.subscribe();
        let (event_tx, event_rx) = mpsc::channel(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let peer_connections = PeerConnectionService::new(state.clone(), network.clone());
        let adapter_task = tokio::spawn(
            LegacyBackendAdapter::new(state.clone(), network.clone(), peer_connections, event_rx)
                .run(),
        );
        let (actor_stream, remote_stream) = duplex(4096);
        let (actor_reader, actor_writer) = split(actor_stream);
        let (remote_reader, mut remote_writer) = split(remote_stream);
        let mut remote_reader = BufReader::new(remote_reader);
        let endpoint_id = iroh::SecretKey::from_bytes(&[7; 32]).public();
        let peer_id = PeerId::new(endpoint_id.to_string());

        network
            .register_incoming_iroh_peer(peer_id, actor_reader, actor_writer)
            .await?;
        remote_writer
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&Peer2PeerMsg::Connect(
                        "Alice".into(),
                        Some("alice-ticket".into()),
                    ))?
                )
                .as_bytes(),
            )
            .await?;

        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::LobbyAccept(3, game_type) if game_type == "poker"
        ));
        line.clear();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::Peers(peers) if peers.is_empty()
        ));
        let new_player =
            tokio::time::timeout(Duration::from_secs(1), frontend_events.recv()).await??;
        assert!(matches!(
            new_player,
            Backend2FrontendMsg::NewPlayer(name) if name == "Alice"
        ));
        let peer = state
            .peers
            .read()
            .await
            .get(&endpoint_id)
            .cloned()
            .expect("accepted peer should be stored");
        assert_eq!(peer.name, "Alice");
        assert_eq!(peer.ticket, "alice-ticket");

        state
            .peer_broadcaster
            .send(Peer2PeerMsg::Payload("broadcast payload".into()))?;
        line.clear();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::Payload(payload) if payload == "broadcast payload"
        ));

        remote_writer
            .write_all(
                format!("{}\n", serde_json::to_string(&Peer2PeerMsg::RequestReady)?).as_bytes(),
            )
            .await?;
        line.clear();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::PeerReady(name, false) if name.is_empty()
        ));

        remote_writer
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&Peer2PeerMsg::Disconnect("Alice".into()))?
                )
                .as_bytes(),
            )
            .await?;
        let removed_player =
            tokio::time::timeout(Duration::from_secs(1), frontend_events.recv()).await??;
        assert!(matches!(
            removed_player,
            Backend2FrontendMsg::RemovePlayer(name) if name == "Alice"
        ));
        line.clear();
        let bytes =
            tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line))
                .await??;
        assert_eq!(bytes, 0, "peer actor should close its writer");
        assert!(!state.peers.read().await.contains_key(&endpoint_id));

        adapter_task.abort();
        supervisor_task.abort();
        let _ = adapter_task.await;
        let _ = supervisor_task.await;
        Ok(())
    }
}
