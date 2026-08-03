use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

use iroh_tickets::{endpoint::EndpointTicket, Ticket};
use mcg_shared::Peer2PeerMsg;

use crate::network::{
    ConnectionId, NetworkCommand, NetworkError, NetworkHandle, PeerConnectionDirection, PeerId,
};

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
    active: HashMap<ConnectionId, ActivePeerConnection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActivePeerConnection {
    peer_id: PeerId,
    direction: PeerConnectionDirection,
}

struct PendingPeerReservation {
    registry: Arc<Mutex<PeerConnectionState>>,
    peer_id: PeerId,
}

impl PendingPeerReservation {
    fn reserve(
        registry: Arc<Mutex<PeerConnectionState>>,
        peer_id: PeerId,
    ) -> Result<Self, PeerConnectionError> {
        {
            let mut state = registry.lock().expect("peer connection registry poisoned");
            if state.pending.contains(&peer_id)
                || state.active.values().any(|known| known.peer_id == peer_id)
            {
                return Err(PeerConnectionError::DuplicatePeer(peer_id));
            }
            state.pending.insert(peer_id.clone());
        }

        Ok(Self { registry, peer_id })
    }
}

impl Drop for PendingPeerReservation {
    fn drop(&mut self) {
        self.registry
            .lock()
            .expect("peer connection registry poisoned")
            .pending
            .remove(&self.peer_id);
    }
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

        let _pending = PendingPeerReservation::reserve(self.registry.clone(), peer_id.clone())?;

        let result = self.network.connect_iroh_peer(ticket).await;
        match result {
            Ok(opened_connection_id) => {
                let connection_id = self
                    .connection_opened(
                        opened_connection_id,
                        peer_id.clone(),
                        PeerConnectionDirection::Outgoing,
                    )
                    .await;
                if connection_id == opened_connection_id {
                    if let Err(error) = self.introduce(connection_id).await {
                        self.connection_closed(connection_id).await;
                        return Err(error.into());
                    }
                }
                Ok(EstablishedPeer {
                    connection_id,
                    peer_id,
                })
            }
            Err(error) => Err(error.into()),
        }
    }

    pub(super) async fn connection_opened(
        &self,
        connection_id: ConnectionId,
        peer_id: PeerId,
        direction: PeerConnectionDirection,
    ) -> ConnectionId {
        let preferred_direction = self
            .local_peer_id()
            .await
            .and_then(|local_peer_id| preferred_direction(&local_peer_id, &peer_id));
        let (winner, loser) = {
            let mut registry = self
                .registry
                .lock()
                .expect("peer connection registry poisoned");
            registry.pending.remove(&peer_id);

            if registry.active.contains_key(&connection_id) {
                return connection_id;
            }

            let existing = registry
                .active
                .iter()
                .find(|(_, connection)| connection.peer_id == peer_id)
                .map(|(connection_id, connection)| (*connection_id, connection.clone()));

            match existing {
                None => {
                    registry.active.insert(
                        connection_id,
                        ActivePeerConnection {
                            peer_id: peer_id.clone(),
                            direction,
                        },
                    );
                    (connection_id, None)
                }
                Some((existing_id, existing_connection)) => {
                    let new_is_preferred = preferred_direction == Some(direction)
                        && preferred_direction != Some(existing_connection.direction);
                    if new_is_preferred {
                        registry.active.remove(&existing_id);
                        registry.active.insert(
                            connection_id,
                            ActivePeerConnection {
                                peer_id: peer_id.clone(),
                                direction,
                            },
                        );
                        (connection_id, Some(existing_id))
                    } else {
                        (existing_id, Some(connection_id))
                    }
                }
            }
        };

        if let Some(loser) = loser {
            tracing::info!(%peer_id, %winner, %loser, ?preferred_direction, "closing duplicate peer connection");
            if let Err(error) = self
                .network
                .send_command(NetworkCommand::CloseConnection {
                    connection_id: loser,
                    reason: format!("duplicate peer connection; keeping {winner}"),
                })
                .await
            {
                tracing::warn!(%peer_id, connection_id = %loser, %error, "failed to close duplicate peer connection");
            }
        }

        winner
    }

    pub(super) async fn connection_closed(&self, connection_id: ConnectionId) {
        self.registry
            .lock()
            .expect("peer connection registry poisoned")
            .active
            .remove(&connection_id);
    }

    async fn local_peer_id(&self) -> Option<PeerId> {
        self.state
            .ticket
            .read()
            .await
            .as_deref()
            .and_then(|ticket| peer_id_from_ticket(ticket).ok())
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

fn preferred_direction(
    local_peer_id: &PeerId,
    remote_peer_id: &PeerId,
) -> Option<PeerConnectionDirection> {
    use std::cmp::Ordering;

    match local_peer_id.cmp(remote_peer_id) {
        Ordering::Less => Some(PeerConnectionDirection::Outgoing),
        Ordering::Greater => Some(PeerConnectionDirection::Incoming),
        Ordering::Equal => None,
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

    #[test]
    fn pending_peer_reservation_is_removed_when_connect_future_is_dropped() {
        let registry = Arc::new(Mutex::new(PeerConnectionState::default()));
        let peer_id = PeerId::new("cancelled-peer");
        let reservation = PendingPeerReservation::reserve(registry.clone(), peer_id.clone())
            .expect("first reservation should succeed");
        assert!(registry
            .lock()
            .expect("peer connection registry poisoned")
            .pending
            .contains(&peer_id));

        drop(reservation);

        assert!(!registry
            .lock()
            .expect("peer connection registry poisoned")
            .pending
            .contains(&peer_id));
    }

    #[tokio::test]
    async fn service_introduces_peer_through_network_actor() -> Result<()> {
        let state = AppState::new(Config::default(), None);
        state.lobby.write().await.our_name = "Bob".into();
        *state.ticket.write().await = Some("bob-ticket".into());
        let (event_tx, mut event_rx) = mpsc::channel::<NetworkEvent>(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let service = PeerConnectionService::new(state, network.clone());
        let (actor_stream, remote_stream) = duplex(4096);
        let (actor_reader, actor_writer) = split(actor_stream);
        let (remote_reader, _remote_writer) = split(remote_stream);
        let mut remote_reader = BufReader::new(remote_reader);
        let peer_id = PeerId::new(iroh::SecretKey::from_bytes(&[9; 32]).public().to_string());
        let connection_id = network
            .register_incoming_iroh_peer(peer_id, actor_reader, actor_writer)
            .await?;
        let opened = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should publish the incoming peer connection");
        assert!(matches!(
            opened,
            NetworkEvent::PeerConnected {
                connection_id: opened_id,
                direction: PeerConnectionDirection::Incoming,
                ..
            } if opened_id == connection_id
        ));

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
            .connection_opened(
                connection_id,
                peer_id.clone(),
                PeerConnectionDirection::Incoming,
            )
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

    #[tokio::test]
    async fn simultaneous_cross_connect_keeps_the_same_physical_connection() -> Result<()> {
        let first_endpoint = iroh::SecretKey::from_bytes(&[11; 32]).public();
        let second_endpoint = iroh::SecretKey::from_bytes(&[12; 32]).public();
        let (lower_endpoint, higher_endpoint) = if first_endpoint < second_endpoint {
            (first_endpoint, second_endpoint)
        } else {
            (second_endpoint, first_endpoint)
        };
        let lower_peer = PeerId::new(lower_endpoint.to_string());
        let higher_peer = PeerId::new(higher_endpoint.to_string());
        let (event_tx, _event_rx) = mpsc::channel::<NetworkEvent>(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());

        let lower_state = AppState::new(Config::default(), None);
        *lower_state.ticket.write().await =
            Some(EndpointTicket::new(iroh::EndpointAddr::new(lower_endpoint)).serialize());
        let lower_service = PeerConnectionService::new(lower_state, network.clone());
        let lower_incoming = ConnectionId::new(51);
        let lower_outgoing = ConnectionId::new(52);
        assert_eq!(
            lower_service
                .connection_opened(
                    lower_incoming,
                    higher_peer.clone(),
                    PeerConnectionDirection::Incoming,
                )
                .await,
            lower_incoming
        );
        assert_eq!(
            lower_service
                .connection_opened(
                    lower_outgoing,
                    higher_peer.clone(),
                    PeerConnectionDirection::Outgoing,
                )
                .await,
            lower_outgoing
        );

        let higher_state = AppState::new(Config::default(), None);
        *higher_state.ticket.write().await =
            Some(EndpointTicket::new(iroh::EndpointAddr::new(higher_endpoint)).serialize());
        let higher_service = PeerConnectionService::new(higher_state, network);
        let higher_incoming = ConnectionId::new(61);
        let higher_outgoing = ConnectionId::new(62);
        assert_eq!(
            higher_service
                .connection_opened(
                    higher_incoming,
                    lower_peer.clone(),
                    PeerConnectionDirection::Incoming,
                )
                .await,
            higher_incoming
        );
        assert_eq!(
            higher_service
                .connection_opened(
                    higher_outgoing,
                    lower_peer,
                    PeerConnectionDirection::Outgoing,
                )
                .await,
            higher_incoming
        );

        assert_eq!(
            lower_service
                .registry
                .lock()
                .expect("peer connection registry poisoned")
                .active
                .len(),
            1
        );
        assert_eq!(
            higher_service
                .registry
                .lock()
                .expect("peer connection registry poisoned")
                .active
                .len(),
            1
        );

        supervisor_task.abort();
        let _ = supervisor_task.await;
        Ok(())
    }
}
