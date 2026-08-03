use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

use anyhow::{Context, Result};
use iroh::endpoint::{Endpoint, RelayMode};
use iroh_tickets::{endpoint::EndpointTicket, Ticket};
use mcg_shared::Peer2PeerMsg;
use native_mcg::network::{
    ConnectionCloseReason, ConnectionId, NetworkCommand, NetworkError, NetworkEvent, NetworkHandle,
    NetworkSupervisor, PeerConnectionDirection, PeerId, TransportKind, IROH_ALPN,
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const TEST_TIMEOUT: Duration = Duration::from_secs(5);

async fn local_endpoint() -> Result<Endpoint> {
    Endpoint::empty_builder(RelayMode::Disabled)
        .clear_discovery()
        .bind_addr_v4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
        .alpns(vec![IROH_ALPN.to_vec()])
        .bind()
        .await
        .context("binding local Iroh test endpoint")
}

fn endpoint_ticket(endpoint: &Endpoint) -> String {
    EndpointTicket::new(endpoint.addr()).serialize()
}

fn start_supervisor() -> (NetworkHandle, mpsc::Receiver<NetworkEvent>, JoinHandle<()>) {
    let (event_tx, event_rx) = mpsc::channel(32);
    let (supervisor, network) = NetworkSupervisor::new(event_tx);
    let task = tokio::spawn(supervisor.run());
    (network, event_rx, task)
}

async fn accept_one(endpoint: Endpoint, network: NetworkHandle) -> Result<ConnectionId> {
    let incoming = tokio::time::timeout(TEST_TIMEOUT, endpoint.accept())
        .await
        .context("waiting for incoming Iroh connection timed out")?
        .context("Iroh endpoint closed before accepting a connection")?;
    let connection = tokio::time::timeout(TEST_TIMEOUT, incoming)
        .await
        .context("accepting Iroh connection timed out")?
        .context("accepting Iroh connection")?;
    let peer_id = PeerId::new(connection.remote_id().to_string());
    let (writer, reader) = tokio::time::timeout(TEST_TIMEOUT, connection.accept_bi())
        .await
        .context("accepting Iroh bidirectional stream timed out")?
        .context("accepting Iroh bidirectional stream")?;

    network
        .register_incoming_iroh_peer(peer_id, reader, writer)
        .await
        .context("registering incoming Iroh stream")
}

async fn next_event(events: &mut mpsc::Receiver<NetworkEvent>) -> Result<NetworkEvent> {
    tokio::time::timeout(TEST_TIMEOUT, events.recv())
        .await
        .context("waiting for network event timed out")?
        .context("network event channel closed")
}

async fn close_endpoint(endpoint: &Endpoint) -> Result<()> {
    tokio::time::timeout(TEST_TIMEOUT, endpoint.close())
        .await
        .context("closing Iroh endpoint timed out")?;
    Ok(())
}

async fn shutdown_supervisor(network: NetworkHandle, task: JoinHandle<()>) -> Result<()> {
    drop(network);
    tokio::time::timeout(TEST_TIMEOUT, task)
        .await
        .context("network supervisor shutdown timed out")??;
    Ok(())
}

#[tokio::test]
async fn real_iroh_endpoints_exchange_typed_messages_and_close_cleanly() -> Result<()> {
    let first_endpoint = local_endpoint().await?;
    let second_endpoint = local_endpoint().await?;
    let first_peer_id = PeerId::new(first_endpoint.id().to_string());
    let second_peer_id = PeerId::new(second_endpoint.id().to_string());
    let first_ticket = endpoint_ticket(&first_endpoint);
    let second_ticket = endpoint_ticket(&second_endpoint);
    let (first_network, mut first_events, first_supervisor) = start_supervisor();
    let (second_network, mut second_events, second_supervisor) = start_supervisor();
    first_network
        .configure_iroh_endpoint(first_endpoint.clone())
        .await?;
    second_network
        .configure_iroh_endpoint(second_endpoint.clone())
        .await?;

    assert!(matches!(
        first_network
            .connect_iroh_peer("not-an-endpoint-ticket")
            .await,
        Err(NetworkError::InvalidPeerTicket(_))
    ));

    let incoming = tokio::spawn(accept_one(second_endpoint.clone(), second_network.clone()));
    let outgoing_id = first_network.connect_iroh_peer(second_ticket).await?;
    assert!(matches!(
        next_event(&mut first_events).await?,
        NetworkEvent::PeerConnected {
            connection_id,
            peer_id,
            transport: TransportKind::Iroh,
            direction: PeerConnectionDirection::Outgoing,
        } if connection_id == outgoing_id && peer_id == second_peer_id
    ));

    first_network
        .send_command(NetworkCommand::SendPeer {
            connection_id: outgoing_id,
            message: Peer2PeerMsg::Connect("Alice".into(), Some(first_ticket.clone())),
        })
        .await?;
    let incoming_id = incoming.await??;
    assert!(matches!(
        next_event(&mut second_events).await?,
        NetworkEvent::PeerConnected {
            connection_id,
            peer_id,
            transport: TransportKind::Iroh,
            direction: PeerConnectionDirection::Incoming,
        } if connection_id == incoming_id && peer_id == first_peer_id
    ));
    assert!(matches!(
        next_event(&mut second_events).await?,
        NetworkEvent::PeerMessage {
            connection_id,
            message: Peer2PeerMsg::Connect(name, Some(ticket)),
        } if connection_id == incoming_id && name == "Alice" && ticket == first_ticket
    ));

    second_network
        .send_command(NetworkCommand::SendPeer {
            connection_id: incoming_id,
            message: Peer2PeerMsg::LobbyAccept(2, "Poker".into()),
        })
        .await?;
    assert!(matches!(
        next_event(&mut first_events).await?,
        NetworkEvent::PeerMessage {
            connection_id,
            message: Peer2PeerMsg::LobbyAccept(2, game_type),
        } if connection_id == outgoing_id && game_type == "Poker"
    ));

    first_network
        .send_command(NetworkCommand::CloseConnection {
            connection_id: outgoing_id,
            reason: "integration test complete".into(),
        })
        .await?;
    assert!(matches!(
        next_event(&mut first_events).await?,
        NetworkEvent::ConnectionClosed {
            connection_id,
            reason: ConnectionCloseReason::LocalRequest(reason),
        } if connection_id == outgoing_id && reason == "integration test complete"
    ));
    let remote_close = next_event(&mut second_events).await?;
    assert!(
        matches!(
            remote_close,
        NetworkEvent::ConnectionClosed {
            connection_id,
            reason: ConnectionCloseReason::RemoteClosed
                | ConnectionCloseReason::TransportError(_),
        } if connection_id == incoming_id
        ),
        "unexpected remote close event: {remote_close:?}"
    );

    close_endpoint(&first_endpoint).await?;
    close_endpoint(&second_endpoint).await?;
    shutdown_supervisor(first_network, first_supervisor).await?;
    shutdown_supervisor(second_network, second_supervisor).await?;
    Ok(())
}

#[tokio::test]
async fn real_iroh_transport_drop_closes_both_connection_actors() -> Result<()> {
    let first_endpoint = local_endpoint().await?;
    let second_endpoint = local_endpoint().await?;
    let second_ticket = endpoint_ticket(&second_endpoint);
    let (first_network, mut first_events, first_supervisor) = start_supervisor();
    let (second_network, mut second_events, second_supervisor) = start_supervisor();
    first_network
        .configure_iroh_endpoint(first_endpoint.clone())
        .await?;
    second_network
        .configure_iroh_endpoint(second_endpoint.clone())
        .await?;

    let incoming = tokio::spawn(accept_one(second_endpoint.clone(), second_network.clone()));
    let outgoing_id = first_network.connect_iroh_peer(second_ticket).await?;
    assert!(matches!(
        next_event(&mut first_events).await?,
        NetworkEvent::PeerConnected {
            connection_id,
            direction: PeerConnectionDirection::Outgoing,
            ..
        } if connection_id == outgoing_id
    ));
    first_network
        .send_command(NetworkCommand::SendPeer {
            connection_id: outgoing_id,
            message: Peer2PeerMsg::Ping,
        })
        .await?;
    let incoming_id = incoming.await??;
    assert!(matches!(
        next_event(&mut second_events).await?,
        NetworkEvent::PeerConnected {
            connection_id,
            direction: PeerConnectionDirection::Incoming,
            ..
        } if connection_id == incoming_id
    ));
    assert!(matches!(
        next_event(&mut second_events).await?,
        NetworkEvent::PeerMessage {
            connection_id,
            message: Peer2PeerMsg::Ping,
        } if connection_id == incoming_id
    ));

    close_endpoint(&first_endpoint).await?;

    assert!(matches!(
        next_event(&mut first_events).await?,
        NetworkEvent::ConnectionClosed {
            connection_id,
            reason: ConnectionCloseReason::TransportError(_),
        } if connection_id == outgoing_id
    ));
    assert!(matches!(
        next_event(&mut second_events).await?,
        NetworkEvent::ConnectionClosed {
            connection_id,
            reason: ConnectionCloseReason::TransportError(_),
        } if connection_id == incoming_id
    ));

    close_endpoint(&second_endpoint).await?;
    shutdown_supervisor(first_network, first_supervisor).await?;
    shutdown_supervisor(second_network, second_supervisor).await?;
    Ok(())
}
