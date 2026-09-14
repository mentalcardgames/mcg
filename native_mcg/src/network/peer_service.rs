use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use iroh_tickets::{endpoint::EndpointTicket, Ticket};
use mcg_shared::Peer2PeerMsg;
use tokio::sync::RwLock;

use super::{
    ConnectionId, NetworkError, NetworkHandle, PeerConnectionDirection, PeerConnectionError,
    PeerId, TransportKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstablishedPeer {
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
            state.pending.insert(peer_id);
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
/// initial peer introduction all pass through this service.
#[derive(Clone)]
pub struct PeerConnectionService {
    local_ticket: Arc<RwLock<Option<EndpointTicket>>>,
    network: NetworkHandle,
    registry: Arc<Mutex<PeerConnectionState>>,
}

impl PeerConnectionService {
    pub fn new(local_ticket: Arc<RwLock<Option<EndpointTicket>>>, network: NetworkHandle) -> Self {
        Self {
            local_ticket,
            network,
            registry: Arc::new(Mutex::new(PeerConnectionState::default())),
        }
    }

    pub async fn connect(
        &self,
        ticket: EndpointTicket,
    ) -> Result<EstablishedPeer, PeerConnectionError> {
        let peer_id = ticket.endpoint_addr().id;
        let own_ticket = self.wait_for_local_ticket().await?;
        if own_ticket.endpoint_addr().id == peer_id {
            return Err(PeerConnectionError::LocalEndpoint(peer_id));
        }

        let _pending = PendingPeerReservation::reserve(self.registry.clone(), peer_id)?;

        let result = self.network.establish_iroh_peer_connection(ticket).await;
        match result {
            Ok(opened_connection_id) => {
                let connection_id = self
                    .connection_opened(
                        opened_connection_id,
                        peer_id,
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

    pub async fn connection_opened(
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
            resolve_connection_in_registry(
                &mut registry,
                connection_id,
                &peer_id,
                direction,
                preferred_direction,
            )
        };

        if let Some(loser) = loser {
            tracing::info!(%peer_id, %winner, %loser, ?preferred_direction, "closing duplicate peer connection");
            if let Err(error) = self
                .network
                .close_connection(
                    loser,
                    format!("duplicate peer connection; keeping {winner}"),
                )
                .await
            {
                tracing::warn!(%peer_id, connection_id = %loser, %error, "failed to close duplicate peer connection");
            }
        }

        winner
    }

    pub fn blocking_connection_opened(
        &self,
        connection_id: ConnectionId,
        peer_id: PeerId,
        direction: PeerConnectionDirection,
    ) -> ConnectionId {
        let preferred_direction = self
            .blocking_local_peer_id()
            .and_then(|local_peer_id| preferred_direction(&local_peer_id, &peer_id));
        let (winner, loser) = {
            let mut registry = self
                .registry
                .lock()
                .expect("peer connection registry poisoned");
            resolve_connection_in_registry(
                &mut registry,
                connection_id,
                &peer_id,
                direction,
                preferred_direction,
            )
        };

        if let Some(loser) = loser {
            tracing::info!(%peer_id, %winner, %loser, ?preferred_direction, "closing duplicate peer connection");
            if let Err(error) = self.network.blocking_close_connection(
                loser,
                format!("duplicate peer connection; keeping {winner}"),
            ) {
                tracing::warn!(%peer_id, connection_id = %loser, %error, "failed to close duplicate peer connection");
            }
        }

        winner
    }

    pub async fn connection_closed(&self, connection_id: ConnectionId) {
        self.blocking_connection_closed(connection_id);
    }

    pub fn blocking_connection_closed(&self, connection_id: ConnectionId) {
        self.registry
            .lock()
            .expect("peer connection registry poisoned")
            .active
            .remove(&connection_id);
    }

    pub fn blocking_connect(
        &self,
        ticket: EndpointTicket,
    ) -> Result<EstablishedPeer, PeerConnectionError> {
        let peer_id = ticket.endpoint_addr().id;
        let own_ticket = self.blocking_wait_for_local_ticket()?;
        if own_ticket.endpoint_addr().id == peer_id {
            return Err(PeerConnectionError::LocalEndpoint(peer_id));
        }

        let _pending = PendingPeerReservation::reserve(self.registry.clone(), peer_id)?;

        let result = self.network.blocking_establish_iroh_peer_connection(ticket);
        match result {
            Ok(opened_connection_id) => {
                let connection_id = self.blocking_connection_opened(
                    opened_connection_id,
                    peer_id,
                    PeerConnectionDirection::Outgoing,
                );
                if connection_id == opened_connection_id {
                    if let Err(error) = self.blocking_introduce(connection_id) {
                        self.blocking_connection_closed(connection_id);
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

    async fn local_peer_id(&self) -> Option<PeerId> {
        self.local_ticket
            .read()
            .await
            .as_ref()
            .map(|ticket| ticket.endpoint_addr().id)
    }

    fn blocking_local_peer_id(&self) -> Option<PeerId> {
        self.local_ticket
            .blocking_read()
            .as_ref()
            .map(|ticket| ticket.endpoint_addr().id)
    }

    async fn wait_for_local_ticket(&self) -> Result<EndpointTicket, NetworkError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(10);
        loop {
            if let Some(ticket) = self.local_ticket.read().await.clone() {
                return Ok(ticket);
            }
            if start.elapsed() > timeout {
                return Err(NetworkError::TransportUnavailable(TransportKind::Iroh));
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    fn blocking_wait_for_local_ticket(&self) -> Result<EndpointTicket, NetworkError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(10);
        loop {
            if let Some(ticket) = self.local_ticket.blocking_read().clone() {
                return Ok(ticket);
            }
            if start.elapsed() > timeout {
                return Err(NetworkError::TransportUnavailable(TransportKind::Iroh));
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    async fn introduce(&self, connection_id: ConnectionId) -> Result<(), NetworkError> {
        let own_ticket = self.wait_for_local_ticket().await?;

        if let Err(error) = self
            .network
            .unicast_peer(
                connection_id,
                Peer2PeerMsg::Connect(String::new(), own_ticket.encode_string()),
            )
            .await
        {
            let _ = self
                .network
                .close_connection(connection_id, "failed to send peer introduction")
                .await;
            return Err(error);
        }

        Ok(())
    }

    fn blocking_introduce(&self, connection_id: ConnectionId) -> Result<(), NetworkError> {
        let own_ticket = self.blocking_wait_for_local_ticket()?;

        if let Err(error) = self.network.blocking_unicast_peer(
            connection_id,
            Peer2PeerMsg::Connect(String::new(), own_ticket.encode_string()),
        ) {
            let _ = self
                .network
                .blocking_close_connection(connection_id, "failed to send peer introduction");
            return Err(error);
        }

        Ok(())
    }
}

fn resolve_connection_in_registry(
    registry: &mut PeerConnectionState,
    connection_id: ConnectionId,
    peer_id: &PeerId,
    direction: PeerConnectionDirection,
    preferred_direction: Option<PeerConnectionDirection>,
) -> (ConnectionId, Option<ConnectionId>) {
    registry.pending.remove(peer_id);

    if registry.active.contains_key(&connection_id) {
        return (connection_id, None);
    }

    let existing = registry
        .active
        .iter()
        .find(|(_, connection)| connection.peer_id == *peer_id)
        .map(|(connection_id, connection)| (*connection_id, connection.clone()));

    match existing {
        None => {
            registry.active.insert(
                connection_id,
                ActivePeerConnection {
                    peer_id: *peer_id,
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
                        peer_id: *peer_id,
                        direction,
                    },
                );
                (connection_id, Some(existing_id))
            } else {
                (existing_id, Some(connection_id))
            }
        }
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Result;
    use tokio::io::{duplex, split, AsyncBufReadExt, BufReader};
    use tokio::sync::mpsc;

    use super::*;
    use crate::controller::ControllerEvent;
    use crate::network::{NetworkEvent, NetworkSupervisor, TransportKind};

    #[test]
    fn pending_peer_reservation_is_removed_when_connect_future_is_dropped() {
        let registry = Arc::new(Mutex::new(PeerConnectionState::default()));
        let peer_id = iroh::SecretKey::from_bytes(&[1; 32]).public();
        let reservation = PendingPeerReservation::reserve(registry.clone(), peer_id)
            .expect("first reservation succeeds");
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
        let secret_key = iroh::SecretKey::from_bytes(&[99; 32]);
        let bob_ticket = EndpointTicket::new(iroh::EndpointAddr::new(secret_key.public()));
        let ticket = Arc::new(RwLock::new(Some(bob_ticket.clone())));
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let supervisor = NetworkSupervisor::new(event_tx);
        let (network, supervisor_task) = supervisor.start();
        let service = PeerConnectionService::new(ticket, network.clone());
        let (actor_stream, remote_stream) = duplex(4096);
        let (actor_reader, actor_writer) = split(actor_stream);
        let (remote_reader, _remote_writer) = split(remote_stream);
        let mut remote_reader = BufReader::new(remote_reader);
        let peer_id = iroh::SecretKey::from_bytes(&[9; 32]).public();
        let connection_id = network
            .register_iroh_peer(peer_id, actor_reader, actor_writer)
            .await?;
        let opened = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should publish the incoming peer connection");
        assert!(matches!(
            opened,
            ControllerEvent::Network(NetworkEvent::PeerConnected {
                connection_id: opened_id,
                direction: PeerConnectionDirection::Incoming,
                ..
            }) if opened_id == connection_id
        ));

        service.introduce(connection_id).await?;

        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::Connect(name, ticket)
                if name.is_empty() && ticket == bob_ticket.encode_string()
        ));

        supervisor_task.abort();
        let _ = supervisor_task.await;
        Ok(())
    }

    #[tokio::test]
    async fn service_deduplicates_connections_for_all_callers() -> Result<()> {
        let local_endpoint_id = iroh::SecretKey::from_bytes(&[20; 32]).public();
        let ticket = Arc::new(RwLock::new(Some(EndpointTicket::new(
            iroh::EndpointAddr::new(local_endpoint_id),
        ))));
        let (event_tx, _event_rx) = mpsc::channel(16);
        let supervisor = NetworkSupervisor::new(event_tx);
        let (network, supervisor_task) = supervisor.start();
        let service = PeerConnectionService::new(ticket, network);
        let endpoint_id = iroh::SecretKey::from_bytes(&[10; 32]).public();
        let peer_id = endpoint_id;
        let ticket = EndpointTicket::new(iroh::EndpointAddr::new(endpoint_id));
        let connection_id = ConnectionId::new(41);
        service
            .connection_opened(connection_id, peer_id, PeerConnectionDirection::Incoming)
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

    #[tokio::test(flavor = "multi_thread")]
    async fn blocking_service_deduplicates_connections_from_sync_thread() -> Result<()> {
        let local_endpoint_id = iroh::SecretKey::from_bytes(&[21; 32]).public();
        let ticket = Arc::new(RwLock::new(Some(EndpointTicket::new(
            iroh::EndpointAddr::new(local_endpoint_id),
        ))));
        let (event_tx, _event_rx) = mpsc::channel(16);
        let supervisor = NetworkSupervisor::new(event_tx);
        let (network, supervisor_task) = supervisor.start();
        let service = PeerConnectionService::new(ticket, network);
        let endpoint_id = iroh::SecretKey::from_bytes(&[10; 32]).public();
        let peer_id = endpoint_id;
        let ticket = EndpointTicket::new(iroh::EndpointAddr::new(endpoint_id));
        let connection_id = ConnectionId::new(42);

        tokio::task::spawn_blocking(move || {
            let opened_winner = service.blocking_connection_opened(
                connection_id,
                peer_id,
                PeerConnectionDirection::Incoming,
            );
            assert_eq!(opened_winner, connection_id);

            assert_eq!(
                service.blocking_connect(ticket.clone()),
                Err(PeerConnectionError::DuplicatePeer(peer_id))
            );

            service.blocking_connection_closed(connection_id);
            assert_eq!(
                service.blocking_connect(ticket),
                Err(PeerConnectionError::Network(
                    NetworkError::TransportUnavailable(TransportKind::Iroh)
                ))
            );
        })
        .await
        .expect("blocking task ok");

        supervisor_task.abort();
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
        let lower_peer = lower_endpoint;
        let higher_peer = higher_endpoint;
        let (event_tx, _event_rx) = mpsc::channel(16);
        let supervisor = NetworkSupervisor::new(event_tx);
        let (network, supervisor_task) = supervisor.start();

        let lower_ticket = Arc::new(RwLock::new(Some(EndpointTicket::new(
            iroh::EndpointAddr::new(lower_endpoint),
        ))));
        let lower_service = PeerConnectionService::new(lower_ticket, network.clone());
        let lower_incoming = ConnectionId::new(51);
        let lower_outgoing = ConnectionId::new(52);
        assert_eq!(
            lower_service
                .connection_opened(
                    lower_incoming,
                    higher_peer,
                    PeerConnectionDirection::Incoming,
                )
                .await,
            lower_incoming
        );
        assert_eq!(
            lower_service
                .connection_opened(
                    lower_outgoing,
                    higher_peer,
                    PeerConnectionDirection::Outgoing,
                )
                .await,
            lower_outgoing
        );

        let higher_ticket = Arc::new(RwLock::new(Some(EndpointTicket::new(
            iroh::EndpointAddr::new(higher_endpoint),
        ))));
        let higher_service = PeerConnectionService::new(higher_ticket, network);
        let higher_incoming = ConnectionId::new(61);
        let higher_outgoing = ConnectionId::new(62);
        assert_eq!(
            higher_service
                .connection_opened(
                    higher_incoming,
                    lower_peer,
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
