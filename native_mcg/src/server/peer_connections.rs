use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use iroh_tickets::{endpoint::EndpointTicket, Ticket};
use mcg_shared::Peer2PeerMsg;
use tokio::sync::Mutex;

use crate::network::{ConnectionId, NetworkCommand, NetworkError, NetworkHandle, PeerId};

use super::state::AppState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum PeerConnectionError {
    Network(NetworkError),
    DuplicatePeer(PeerId),
    LocalEndpoint(PeerId),
}

impl fmt::Display for PeerConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => error.fmt(formatter),
            Self::DuplicatePeer(peer_id) => {
                write!(
                    formatter,
                    "peer {peer_id} is already connected or connecting"
                )
            }
            Self::LocalEndpoint(peer_id) => {
                write!(formatter, "cannot connect to local endpoint {peer_id}")
            }
        }
    }
}

impl Error for PeerConnectionError {}

impl From<NetworkError> for PeerConnectionError {
    fn from(error: NetworkError) -> Self {
        Self::Network(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EstablishedPeer {
    pub connection_id: ConnectionId,
    pub peer_id: PeerId,
}

#[derive(Default)]
struct PeerConnectionState {
    pending: HashSet<PeerId>,
    active: HashMap<ConnectionId, PeerId>,
}

/// Coordinates application-level outgoing peer connections for every caller.
///
/// Ticket validation, duplicate suppression, transport establishment, and the
/// initial peer introduction all pass through this service. Network events are
/// fed back through [`Self::connection_opened`] and [`Self::connection_closed`]
/// so HTTP requests and the legacy adapter share one view of connection state.
#[derive(Clone)]
pub(super) struct PeerConnectionService {
    state: AppState,
    network: NetworkHandle,
    registry: Arc<Mutex<PeerConnectionState>>,
}

impl PeerConnectionService {
    pub(super) fn new(state: AppState, network: NetworkHandle) -> Self {
        Self {
            state,
            network,
            registry: Arc::new(Mutex::new(PeerConnectionState::default())),
        }
    }

    pub(super) async fn connect(
        &self,
        ticket: String,
    ) -> Result<EstablishedPeer, PeerConnectionError> {
        let peer_id = peer_id_from_ticket(&ticket)?;
        if self.local_peer_id().await.as_ref() == Some(&peer_id) {
            return Err(PeerConnectionError::LocalEndpoint(peer_id));
        }

        {
            let mut registry = self.registry.lock().await;
            if registry.pending.contains(&peer_id)
                || registry.active.values().any(|known| known == &peer_id)
            {
                return Err(PeerConnectionError::DuplicatePeer(peer_id));
            }
            registry.pending.insert(peer_id.clone());
        }

        let result = self.establish_and_introduce(ticket).await;
        match result {
            Ok(connection_id) => {
                self.connection_opened(connection_id, peer_id.clone()).await;
                Ok(EstablishedPeer {
                    connection_id,
                    peer_id,
                })
            }
            Err(error) => {
                self.registry.lock().await.pending.remove(&peer_id);
                Err(error.into())
            }
        }
    }

    pub(super) async fn connection_opened(&self, connection_id: ConnectionId, peer_id: PeerId) {
        let mut registry = self.registry.lock().await;
        registry.pending.remove(&peer_id);
        registry.active.insert(connection_id, peer_id);
    }

    pub(super) async fn connection_closed(&self, connection_id: ConnectionId) {
        self.registry.lock().await.active.remove(&connection_id);
    }

    async fn local_peer_id(&self) -> Option<PeerId> {
        self.state
            .ticket
            .read()
            .await
            .as_deref()
            .and_then(|ticket| peer_id_from_ticket(ticket).ok())
    }

    async fn establish_and_introduce(&self, ticket: String) -> Result<ConnectionId, NetworkError> {
        let connection_id = self.network.connect_iroh_peer(ticket).await?;
        self.introduce(connection_id).await?;
        Ok(connection_id)
    }

    async fn introduce(&self, connection_id: ConnectionId) -> Result<(), NetworkError> {
        let name = self.state.lobby.read().await.our_name.clone();
        let own_ticket = self.state.ticket.read().await.clone();

        if let Err(error) = self
            .network
            .send_command(NetworkCommand::SendPeer {
                connection_id,
                message: Peer2PeerMsg::Connect(name, own_ticket),
            })
            .await
        {
            let _ = self
                .network
                .send_command(NetworkCommand::CloseConnection {
                    connection_id,
                    reason: "failed to send peer introduction".into(),
                })
                .await;
            return Err(error);
        }

        Ok(())
    }
}

pub(super) fn peer_id_from_ticket(ticket: &str) -> Result<PeerId, NetworkError> {
    let ticket = EndpointTicket::deserialize(ticket)
        .map_err(|error| NetworkError::InvalidPeerTicket(error.to_string()))?;
    Ok(PeerId::new(ticket.endpoint_addr().id.to_string()))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Result;
    use tokio::io::{duplex, split, AsyncBufReadExt, BufReader};
    use tokio::sync::mpsc;

    use super::*;
    use crate::config::Config;
    use crate::network::{NetworkEvent, NetworkSupervisor, TransportKind};

    #[tokio::test]
    async fn service_introduces_peer_through_network_actor() -> Result<()> {
        let state = AppState::new(Config::default(), None);
        state.lobby.write().await.our_name = "Bob".into();
        *state.ticket.write().await = Some("bob-ticket".into());
        let (event_tx, _event_rx) = mpsc::channel::<NetworkEvent>(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let service = PeerConnectionService::new(state, network.clone());
        let (actor_stream, remote_stream) = duplex(4096);
        let (actor_reader, actor_writer) = split(actor_stream);
        let (remote_reader, _remote_writer) = split(remote_stream);
        let mut remote_reader = BufReader::new(remote_reader);
        let peer_id = PeerId::new(iroh::SecretKey::from_bytes(&[9; 32]).public().to_string());
        let connection_id = network
            .register_iroh_peer(peer_id, actor_reader, actor_writer)
            .await?;

        service.introduce(connection_id).await?;

        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::Connect(name, Some(ticket))
                if name == "Bob" && ticket == "bob-ticket"
        ));

        supervisor_task.abort();
        let _ = supervisor_task.await;
        Ok(())
    }

    #[tokio::test]
    async fn service_deduplicates_connections_for_all_callers() -> Result<()> {
        let state = AppState::new(Config::default(), None);
        let (event_tx, _event_rx) = mpsc::channel::<NetworkEvent>(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let service = PeerConnectionService::new(state, network);
        let endpoint_id = iroh::SecretKey::from_bytes(&[10; 32]).public();
        let peer_id = PeerId::new(endpoint_id.to_string());
        let ticket = EndpointTicket::new(iroh::EndpointAddr::new(endpoint_id)).serialize();
        let connection_id = ConnectionId::new(41);
        service
            .connection_opened(connection_id, peer_id.clone())
            .await;

        assert_eq!(
            service.connect(ticket.clone()).await,
            Err(PeerConnectionError::DuplicatePeer(peer_id))
        );

        service.connection_closed(connection_id).await;
        assert_eq!(
            service.connect(ticket).await,
            Err(PeerConnectionError::Network(
                NetworkError::TransportUnavailable(TransportKind::Iroh)
            ))
        );

        supervisor_task.abort();
        let _ = supervisor_task.await;
        Ok(())
    }
}
