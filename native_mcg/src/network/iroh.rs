use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use iroh::endpoint::Endpoint;
use iroh_tickets::{endpoint::EndpointTicket, Ticket};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, Peer2PeerMsg};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::task::{JoinHandle, JoinSet};

use crate::config::Config;
use crate::public::{path_for_config, PublicInfo};
use crate::transport::{send_peer_msg_to_writer, send_server_msg_to_writer};

use super::types::ActorEvent;
use super::{
    ConnectionCloseReason, ConnectionId, FrontendConnectionCommand, NetworkHandle,
    PeerConnectionCommand, PeerId, ProtocolRole,
};

/// Application protocol for Iroh connections between an MCG frontend and backend.
pub const IROH_FRONTEND_ALPN: &[u8] = b"mcg/iroh/frontend";
/// Application protocol for Iroh connections between MCG backend peers.
pub const IROH_PEER_ALPN: &[u8] = b"mcg/iroh/peer";

const IROH_CONNECTION_SETUP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub(super) type IrohReader = Box<dyn AsyncRead + Unpin + Send>;
pub(super) type IrohWriter = Box<dyn AsyncWrite + Unpin + Send>;

#[derive(Debug)]
pub(super) enum IrohConnectError {
    InvalidTicket(String),
    Connect(String),
    OpenStream(String),
}

#[async_trait]
pub(super) trait IrohConnector: Send + Sync {
    async fn connect(
        &self,
        ticket: String,
    ) -> Result<(PeerId, IrohReader, IrohWriter), IrohConnectError>;
}

pub(super) struct IrohEndpointConnector {
    endpoint: Endpoint,
}

impl IrohEndpointConnector {
    pub(super) fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
}

#[async_trait]
impl IrohConnector for IrohEndpointConnector {
    async fn connect(
        &self,
        ticket: String,
    ) -> Result<(PeerId, IrohReader, IrohWriter), IrohConnectError> {
        let ticket = EndpointTicket::decode_string(&ticket)
            .map_err(|error| IrohConnectError::InvalidTicket(error.to_string()))?;
        let connection = self
            .endpoint
            .connect(ticket.endpoint_addr().clone(), IROH_PEER_ALPN)
            .await
            .map_err(|error| IrohConnectError::Connect(error.to_string()))?;
        let peer_id = PeerId::new(connection.remote_id().to_string());
        let (writer, reader) = connection
            .open_bi()
            .await
            .map_err(|error| IrohConnectError::OpenStream(error.to_string()))?;

        Ok((peer_id, Box::new(reader), Box::new(writer)))
    }
}

/// Owned background task for the Iroh endpoint and its incoming connections.
pub struct IrohListenerTask {
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl IrohListenerTask {
    pub async fn shutdown(mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        if let Some(task) = self.task.take() {
            if let Err(error) = task.await {
                tracing::error!(%error, "Iroh listener task failed during shutdown");
            }
        }
    }
}

impl Drop for IrohListenerTask {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}

/// Starts an owned Iroh listener task without delaying the HTTP server startup.
pub fn spawn_iroh_listener(
    config: Config,
    config_path: Option<PathBuf>,
    local_ticket: Arc<RwLock<Option<String>>>,
    network: NetworkHandle,
) -> IrohListenerTask {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        if let Err(error) =
            run_iroh_listener(config, config_path, local_ticket, network, shutdown_rx).await
        {
            tracing::error!(%error, "Iroh listener failed");
        }
    });
    IrohListenerTask {
        shutdown_tx: Some(shutdown_tx),
        task: Some(task),
    }
}

/// Creates the Iroh endpoint and accepts peer connections until shutdown.
async fn run_iroh_listener(
    config: Config,
    config_path: Option<PathBuf>,
    local_ticket: Arc<RwLock<Option<String>>>,
    network: NetworkHandle,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    use iroh::SecretKey;
    use iroh_tickets::{endpoint::EndpointTicket, Ticket};

    let secret_key: SecretKey = load_or_generate_iroh_secret(&config, config_path.as_deref()).await;
    let endpoint = build_iroh_endpoint(secret_key).await?;
    network
        .configure_iroh_endpoint(endpoint.clone())
        .await
        .context("configuring Iroh endpoint in network supervisor")?;

    tokio::select! {
        _ = &mut shutdown_rx => {
            endpoint.close().await;
            return Ok(());
        }
        online = tokio::time::timeout(IROH_CONNECTION_SETUP_TIMEOUT, endpoint.online()) => {
            match online {
                Ok(()) => tracing::info!("iroh endpoint is online (relay connected)"),
                Err(_) => {
                    tracing::warn!("timeout waiting for iroh endpoint to come online; proceeding anyway")
                }
            }
        }
    }

    let endpoint_id = endpoint.id();
    println!("\n\x1b[1;32m=== Iroh Endpoint Ready ===\x1b[0m");
    println!("\x1b[1mNode ID:\x1b[0m {endpoint_id}");
    println!("\x1b[1;32m===========================\x1b[0m\n");

    let addr = endpoint.addr();
    let relay_urls: Vec<_> = addr.relay_urls().collect();
    tracing::info!(iroh_node_id = %endpoint_id, iroh_addr = ?addr, relay_urls = ?relay_urls);

    let ticket = EndpointTicket::new(addr);
    println!("{ticket}");
    tracing::info!(ticket = %ticket);
    let ticket = ticket.encode_string();
    *local_ticket.write().await = Some(ticket);

    let public_path = path_for_config(config_path.as_deref());
    match PublicInfo::write_iroh_node_id(&public_path, endpoint_id.to_string()) {
        Ok(_) => tracing::info!(path = %public_path.display(), "stored iroh node id"),
        Err(error) => {
            tracing::warn!(%error, path = %public_path.display(), "failed to persist iroh node id")
        }
    }

    tracing::info!(
        peer_alpn = %std::str::from_utf8(IROH_PEER_ALPN).unwrap_or("mcg/iroh/peer"),
        frontend_alpn = %std::str::from_utf8(IROH_FRONTEND_ALPN).unwrap_or("mcg/iroh/frontend"),
        "iroh listener started"
    );
    run_iroh_accept_loop(endpoint, network, shutdown_rx).await;
    Ok(())
}

async fn load_or_generate_iroh_secret(
    config: &Config,
    config_path: Option<&Path>,
) -> iroh::SecretKey {
    use getrandom::getrandom;
    use iroh::SecretKey;

    let generate_new_key = || -> SecretKey {
        let mut bytes = [0u8; 32];
        if let Err(error) = getrandom(&mut bytes) {
            tracing::error!(%error, "failed to get randomness for iroh key");
        }
        SecretKey::from_bytes(&bytes)
    };

    if let Some(config_path) = config_path {
        if let Some(bytes) = config.iroh_key_bytes() {
            if bytes.len() >= 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[..32]);
                return SecretKey::from_bytes(&key);
            }
        }

        let secret_key = generate_new_key();
        let mut updated_config = config.clone();
        if let Err(error) =
            updated_config.set_iroh_key_bytes_and_save(config_path, &secret_key.to_bytes())
        {
            tracing::error!(%error, "failed to save generated Iroh key to config '{}'", config_path.display());
        } else {
            tracing::info!(config_path = %config_path.display(), "saved generated Iroh key into config");
        }
        secret_key
    } else {
        if let Some(bytes) = config.iroh_key_bytes() {
            if bytes.len() >= 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[..32]);
                return SecretKey::from_bytes(&key);
            }
        }
        tracing::warn!(
            "no server config path provided; generating ephemeral iroh key (not persisted)"
        );
        generate_new_key()
    }
}

async fn build_iroh_endpoint(secret_key: iroh::SecretKey) -> Result<iroh::endpoint::Endpoint> {
    use iroh::endpoint::Endpoint;

    Endpoint::builder(iroh::endpoint::presets::N0)
        .alpns(vec![IROH_PEER_ALPN.to_vec(), IROH_FRONTEND_ALPN.to_vec()])
        .secret_key(secret_key)
        .bind()
        .await
        .context("binding iroh endpoint")
}

async fn run_iroh_accept_loop(
    endpoint: iroh::endpoint::Endpoint,
    network: NetworkHandle,
    mut shutdown_rx: oneshot::Receiver<()>,
) {
    let mut connection_tasks = JoinSet::new();
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => break,
            incoming = endpoint.accept() => {
                let Some(incoming) = incoming else {
                    break;
                };
                let network = network.clone();
                connection_tasks.spawn(async move {
                    let setup = async {
                        let mut accepting = incoming
                            .accept()
                            .context("starting incoming Iroh handshake")?;
                        let alpn = accepting
                            .alpn()
                            .await
                            .context("reading incoming Iroh ALPN")?;
                        let connection = accepting.await.context("accepting incoming Iroh connection")?;
                        let remote_id = connection.remote_id();
                        tracing::info!(peer = %remote_id, alpn = %String::from_utf8_lossy(&alpn), "accepted new Iroh connection");
                        register_incoming_iroh_connection(network, connection, &alpn)
                            .await
                            .with_context(|| format!("registering incoming Iroh connection from {remote_id}"))
                    };
                    match tokio::time::timeout(IROH_CONNECTION_SETUP_TIMEOUT, setup).await {
                        Ok(Ok(())) => {}
                        Ok(Err(error)) => tracing::error!(%error, "failed to set up incoming Iroh connection"),
                        Err(_) => tracing::warn!("timed out while setting up incoming Iroh connection"),
                    }
                });
            }
            task = connection_tasks.join_next(), if !connection_tasks.is_empty() => {
                if let Some(Err(error)) = task {
                    tracing::error!(%error, "incoming Iroh connection task failed");
                }
            }
        }
    }

    endpoint.close().await;
    connection_tasks.shutdown().await;
    tracing::info!("Iroh listener stopped");
}

async fn register_incoming_iroh_connection(
    network: NetworkHandle,
    connection: iroh::endpoint::Connection,
    alpn: &[u8],
) -> Result<()> {
    let remote_id = connection.remote_id();
    let (writer, reader) = connection
        .accept_bi()
        .await
        .context("accepting incoming Iroh bidirectional stream")?;
    match protocol_role_from_alpn(alpn) {
        Some(ProtocolRole::Peer) => {
            let peer_id = PeerId::new(remote_id.to_string());
            let connection_id = network
                .register_incoming_iroh_peer(peer_id.clone(), reader, writer)
                .await
                .context("registering incoming Iroh peer with network supervisor")?;
            tracing::info!(%connection_id, %peer_id, "incoming Iroh peer registered");
            Ok(())
        }
        Some(ProtocolRole::Frontend) => {
            let connection_id = network
                .register_incoming_iroh_frontend(reader, writer)
                .await
                .context("registering incoming Iroh frontend with network supervisor")?;
            tracing::info!(%connection_id, endpoint_id = %remote_id, "incoming Iroh frontend registered");
            Ok(())
        }
        None => anyhow::bail!("unsupported Iroh ALPN {}", String::from_utf8_lossy(alpn)),
    }
}

fn protocol_role_from_alpn(alpn: &[u8]) -> Option<ProtocolRole> {
    if alpn == IROH_PEER_ALPN {
        Some(ProtocolRole::Peer)
    } else if alpn == IROH_FRONTEND_ALPN {
        Some(ProtocolRole::Frontend)
    } else {
        None
    }
}

/// Runs one established Iroh frontend stream.
///
/// Iroh frontend connections use the same typed protocol as WebSockets, with
/// newline-delimited JSON as their transport framing.
pub(super) async fn run_iroh_frontend_actor<R, W>(
    connection_id: ConnectionId,
    reader: R,
    mut writer: W,
    event_tx: mpsc::Sender<ActorEvent>,
    mut command_rx: mpsc::Receiver<FrontendConnectionCommand>,
) where
    R: AsyncRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send,
{
    if event_tx
        .send(ActorEvent::Ready { connection_id })
        .await
        .is_err()
    {
        tracing::debug!(%connection_id, "network event receiver dropped before Iroh frontend actor started");
        return;
    }
    tracing::info!(%connection_id, "Iroh frontend connection actor started");

    let mut lines = BufReader::new(reader).lines();
    let close_reason = loop {
        tokio::select! {
            incoming = lines.next_line() => {
                match incoming {
                    Ok(Some(line)) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<Frontend2BackendMsg>(line) {
                            Ok(message) => {
                                let event = ActorEvent::FrontendMessage {
                                    connection_id,
                                    message,
                                };
                                if event_tx.send(event).await.is_err() {
                                    break ConnectionCloseReason::EventReceiverClosed;
                                }
                            }
                            Err(error) => {
                                tracing::warn!(%connection_id, %error, "failed to parse Iroh frontend message");
                                let response = Backend2FrontendMsg::Error(
                                    "Malformed Frontend2BackendMsg JSON".into(),
                                );
                                if let Err(error) = send_server_msg_to_writer(&mut writer, &response).await {
                                    break ConnectionCloseReason::TransportError(error.to_string());
                                }
                            }
                        }
                    }
                    Ok(None) => break ConnectionCloseReason::RemoteClosed,
                    Err(error) => break ConnectionCloseReason::TransportError(error.to_string()),
                }
            }
            command = command_rx.recv() => {
                match command {
                    Some(FrontendConnectionCommand::Send(message)) => {
                        if let Err(error) = send_server_msg_to_writer(&mut writer, &message).await {
                            break ConnectionCloseReason::TransportError(error.to_string());
                        }
                    }
                    Some(FrontendConnectionCommand::Close { reason }) => {
                        if let Err(error) = writer.shutdown().await {
                            break ConnectionCloseReason::TransportError(error.to_string());
                        }
                        break ConnectionCloseReason::LocalRequest(reason);
                    }
                    None => break ConnectionCloseReason::OutboundChannelClosed,
                }
            }
        }
    };

    let _ = event_tx
        .send(ActorEvent::Closed {
            connection_id,
            reason: close_reason,
        })
        .await;
    tracing::info!(%connection_id, "Iroh frontend connection actor stopped");
}

/// Runs one established Iroh peer stream.
///
/// Stream establishment remains the responsibility of the Iroh endpoint
/// owner. The actor only translates newline-delimited peer messages and has no
/// access to application state or connection policy.
pub(super) async fn run_iroh_peer_actor<R, W>(
    connection_id: ConnectionId,
    reader: R,
    mut writer: W,
    event_tx: mpsc::Sender<ActorEvent>,
    mut command_rx: mpsc::Receiver<PeerConnectionCommand>,
) where
    R: AsyncRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send,
{
    // Respond success with starting the actor loop
    if event_tx
        .send(ActorEvent::Ready { connection_id })
        .await
        .is_err()
    {
        tracing::debug!(%connection_id, "network event receiver dropped before Iroh peer actor started");
        return;
    }
    tracing::info!(%connection_id, "Iroh peer connection actor started");

    let mut lines = BufReader::new(reader).lines();
    let close_reason = loop {
        tokio::select! {
            incoming = lines.next_line() => {
                match incoming {
                    Ok(Some(line)) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<Peer2PeerMsg>(line) {
                            Ok(message) => {
                                let event = ActorEvent::PeerMessage {
                                    connection_id,
                                    message,
                                };
                                if event_tx.send(event).await.is_err() {
                                    break ConnectionCloseReason::EventReceiverClosed;
                                }
                            }
                            Err(error) => {
                                tracing::warn!(%connection_id, %error, "failed to parse Iroh peer message");
                                break ConnectionCloseReason::ProtocolError(error.to_string());
                            }
                        }
                    }
                    Ok(None) => break ConnectionCloseReason::RemoteClosed,
                    Err(error) => {
                        break ConnectionCloseReason::TransportError(error.to_string());
                    }
                }
            }
            command = command_rx.recv() => {
                match command {
                    Some(PeerConnectionCommand::Send(message)) => {
                        if let Err(error) = send_peer_msg_to_writer(&mut writer, &message).await {
                            break ConnectionCloseReason::TransportError(error.to_string());
                        }
                    }
                    Some(PeerConnectionCommand::Close { reason }) => {
                        if let Err(error) = writer.shutdown().await {
                            break ConnectionCloseReason::TransportError(error.to_string());
                        }
                        break ConnectionCloseReason::LocalRequest(reason);
                    }
                    None => break ConnectionCloseReason::OutboundChannelClosed,
                }
            }
        }
    };

    let _ = event_tx
        .send(ActorEvent::Closed {
            connection_id,
            reason: close_reason,
        })
        .await;
    tracing::info!(%connection_id, "Iroh peer connection actor stopped");
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Result;
    use mcg_shared::Frontend2BackendMsg;
    use tokio::io::{duplex, split, AsyncBufReadExt, AsyncWriteExt, BufReader};

    use super::*;

    #[test]
    fn iroh_alpns_select_exactly_one_protocol_role() {
        assert_eq!(
            protocol_role_from_alpn(IROH_PEER_ALPN),
            Some(ProtocolRole::Peer)
        );
        assert_eq!(
            protocol_role_from_alpn(IROH_FRONTEND_ALPN),
            Some(ProtocolRole::Frontend)
        );
        assert_eq!(protocol_role_from_alpn(b"mcg/iroh/unknown"), None);
    }

    #[tokio::test]
    async fn actor_translates_peer_messages_in_both_directions_and_closes() -> Result<()> {
        let (actor_stream, remote_stream) = duplex(4096);
        let (actor_reader, actor_writer) = split(actor_stream);
        let (remote_reader, mut remote_writer) = split(remote_stream);
        let mut remote_reader = BufReader::new(remote_reader);
        let (event_tx, mut event_rx) = mpsc::channel(8);
        let (command_tx, command_rx) = mpsc::channel(8);
        let connection_id = ConnectionId::new(23);
        let actor_task = tokio::spawn(run_iroh_peer_actor(
            connection_id,
            actor_reader,
            actor_writer,
            event_tx,
            command_rx,
        ));

        let opened = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should report its open connection");
        assert!(matches!(
            opened,
            ActorEvent::Ready { connection_id: opened_id } if opened_id == connection_id
        ));

        remote_writer
            .write_all(format!("{}\n", serde_json::to_string(&Peer2PeerMsg::Ping)?).as_bytes())
            .await?;
        let incoming = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should emit a peer message");
        assert!(matches!(
            incoming,
            ActorEvent::PeerMessage {
                connection_id: source,
                message: Peer2PeerMsg::Ping,
            } if source == connection_id
        ));

        command_tx
            .send(PeerConnectionCommand::Send(Peer2PeerMsg::Pong))
            .await?;
        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(1), remote_reader.read_line(&mut line)).await??;
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(line.trim())?,
            Peer2PeerMsg::Pong
        ));

        command_tx
            .send(PeerConnectionCommand::Close {
                reason: "peer actor test shutdown".into(),
            })
            .await?;
        let closed = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should report its closed connection");
        assert!(matches!(
            closed,
            ActorEvent::Closed {
                connection_id: closed_id,
                reason: ConnectionCloseReason::LocalRequest(reason),
            } if closed_id == connection_id && reason == "peer actor test shutdown"
        ));

        actor_task.await?;
        Ok(())
    }

    #[tokio::test]
    async fn actor_does_not_fall_back_to_frontend_protocol() -> Result<()> {
        let (actor_stream, mut remote_stream) = duplex(4096);
        let (actor_reader, actor_writer) = split(actor_stream);
        let (event_tx, mut event_rx) = mpsc::channel(8);
        let (_command_tx, command_rx) = mpsc::channel(8);
        let connection_id = ConnectionId::new(31);
        let actor_task = tokio::spawn(run_iroh_peer_actor(
            connection_id,
            actor_reader,
            actor_writer,
            event_tx,
            command_rx,
        ));

        let opened = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should report its open connection");
        assert!(matches!(opened, ActorEvent::Ready { .. }));

        remote_stream
            .write_all(
                format!(
                    "{}\n",
                    serde_json::to_string(&Frontend2BackendMsg::RequestState)?
                )
                .as_bytes(),
            )
            .await?;
        let closed = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should reject a frontend protocol message");
        assert!(matches!(
            closed,
            ActorEvent::Closed {
                connection_id: closed_id,
                reason: ConnectionCloseReason::ProtocolError(_),
            } if closed_id == connection_id
        ));

        actor_task.await?;
        Ok(())
    }
}
