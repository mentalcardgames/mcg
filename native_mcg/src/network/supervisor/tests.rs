use std::sync::Arc;
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
use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

use super::*;
use crate::network::iroh::{IrohConnectError, IrohConnector, IrohReader, IrohWriter};
use crate::network::{ConnectionCloseReason, NetworkCommand, PeerId, ProtocolRole, TransportKind};

struct OneShotIrohConnector {
    peer_id: PeerId,
    stream: TokioMutex<Option<(IrohReader, IrohWriter)>>,
}

struct PendingIrohConnector;

struct SignalingPendingIrohConnector {
    started_tx: TokioMutex<Option<oneshot::Sender<()>>>,
}

#[async_trait]
impl IrohConnector for OneShotIrohConnector {
    async fn connect(
        &self,
        _ticket: String,
    ) -> Result<(PeerId, IrohReader, IrohWriter), IrohConnectError> {
        let (reader, writer) = self
            .stream
            .lock()
            .await
            .take()
            .ok_or_else(|| IrohConnectError::Connect("test stream already consumed".into()))?;
        Ok((self.peer_id.clone(), reader, writer))
    }
}

#[async_trait]
impl IrohConnector for PendingIrohConnector {
    async fn connect(
        &self,
        _ticket: String,
    ) -> Result<(PeerId, IrohReader, IrohWriter), IrohConnectError> {
        std::future::pending().await
    }
}

#[async_trait]
impl IrohConnector for SignalingPendingIrohConnector {
    async fn connect(
        &self,
        _ticket: String,
    ) -> Result<(PeerId, IrohReader, IrohWriter), IrohConnectError> {
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
            .register_frontend_websocket(socket)
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

    let (mut client, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws")).await?;
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
            stream: TokioMutex::new(Some((Box::new(actor_reader), Box::new(actor_writer)))),
        }))
        .await?;
    let duplicate_configuration = network
        .configure_iroh_connector(Arc::new(OneShotIrohConnector {
            peer_id: PeerId::new("unused-test-peer"),
            stream: TokioMutex::new(None),
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
            started_tx: TokioMutex::new(Some(started_tx)),
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

#[tokio::test]
async fn supervisor_broadcasts_to_all_peers_and_frontends() -> Result<()> {
    let (event_tx, mut event_rx) = mpsc::channel(16);
    let (supervisor, network) = NetworkSupervisor::new(event_tx);
    let supervisor_task = tokio::spawn(supervisor.run());

    // Setup 2 peer connections using duplex streams
    let (peer1_stream, peer1_remote) = duplex(4096);
    let (peer1_r, peer1_w) = split(peer1_stream);
    let (peer1_rem_r, _peer1_rem_w) = split(peer1_remote);
    let mut peer1_reader = BufReader::new(peer1_rem_r);

    let (peer2_stream, peer2_remote) = duplex(4096);
    let (peer2_r, peer2_w) = split(peer2_stream);
    let (peer2_rem_r, _peer2_rem_w) = split(peer2_remote);
    let mut peer2_reader = BufReader::new(peer2_rem_r);

    let _conn1 = network
        .register_incoming_iroh_peer(PeerId::new("peer-1"), peer1_r, peer1_w)
        .await?;
    let _conn2 = network
        .register_incoming_iroh_peer(PeerId::new("peer-2"), peer2_r, peer2_w)
        .await?;

    // Drain peer connected events
    for _ in 0..2 {
        let event = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("event");
        assert!(matches!(event, NetworkEvent::PeerConnected { .. }));
    }

    // Broadcast a peer message
    network.broadcast_peer(Peer2PeerMsg::Ping).await?;

    // Read the broadcast from both peer remotes
    let mut line1 = String::new();
    peer1_reader.read_line(&mut line1).await?;
    let msg1: Peer2PeerMsg = serde_json::from_str(line1.trim())?;
    assert!(matches!(msg1, Peer2PeerMsg::Ping));

    let mut line2 = String::new();
    peer2_reader.read_line(&mut line2).await?;
    let msg2: Peer2PeerMsg = serde_json::from_str(line2.trim())?;
    assert!(matches!(msg2, Peer2PeerMsg::Ping));

    network.shutdown().await?;
    tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
    Ok(())
}
