use async_trait::async_trait;
use iroh::endpoint::Endpoint;
use iroh_tickets::{endpoint::EndpointTicket, Ticket};
use mcg_shared::Peer2PeerMsg;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use crate::transport::send_peer_msg_to_writer;

use super::{
    ConnectionCloseReason, ConnectionId, ConnectionInfo, ConnectionRole, NetworkEvent,
    PeerConnectionCommand, TransportKind,
};

pub(super) const IROH_ALPN: &[u8] = b"mcg/iroh/1";

pub(super) type PeerReader = Box<dyn AsyncRead + Unpin + Send>;
pub(super) type PeerWriter = Box<dyn AsyncWrite + Unpin + Send>;

#[derive(Debug)]
pub(super) enum IrohConnectError {
    InvalidTicket(String),
    Connect(String),
    OpenStream(String),
}

#[async_trait]
pub(super) trait IrohConnector: Send + Sync {
    async fn connect(&self, ticket: String) -> Result<(PeerReader, PeerWriter), IrohConnectError>;
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
    async fn connect(&self, ticket: String) -> Result<(PeerReader, PeerWriter), IrohConnectError> {
        let ticket = EndpointTicket::deserialize(&ticket)
            .map_err(|error| IrohConnectError::InvalidTicket(error.to_string()))?;
        let connection = self
            .endpoint
            .connect(ticket.endpoint_addr().clone(), IROH_ALPN)
            .await
            .map_err(|error| IrohConnectError::Connect(error.to_string()))?;
        let (writer, reader) = connection
            .open_bi()
            .await
            .map_err(|error| IrohConnectError::OpenStream(error.to_string()))?;

        Ok((Box::new(reader), Box::new(writer)))
    }
}

/// Runs one established Iroh peer stream.
///
/// Stream establishment remains the responsibility of the Iroh endpoint
/// owner. The actor only translates newline-delimited peer messages and has no
/// access to application state or connection policy.
pub async fn run_iroh_peer_actor<R, W>(
    connection_id: ConnectionId,
    reader: R,
    mut writer: W,
    event_tx: mpsc::Sender<NetworkEvent>,
    mut command_rx: mpsc::Receiver<PeerConnectionCommand>,
) where
    R: AsyncRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send,
{
    let connection = ConnectionInfo {
        id: connection_id,
        role: ConnectionRole::Peer,
        transport: TransportKind::Iroh,
    };

    // Respond success with starting the actor loop
    if event_tx
        .send(NetworkEvent::ConnectionOpened { connection })
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
                                let event = NetworkEvent::PeerMessage {
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
        .send(NetworkEvent::ConnectionClosed {
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
            NetworkEvent::ConnectionOpened {
                connection: ConnectionInfo {
                    id,
                    role: ConnectionRole::Peer,
                    transport: TransportKind::Iroh,
                }
            } if id == connection_id
        ));

        remote_writer
            .write_all(format!("{}\n", serde_json::to_string(&Peer2PeerMsg::Ping)?).as_bytes())
            .await?;
        let incoming = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should emit a peer message");
        assert!(matches!(
            incoming,
            NetworkEvent::PeerMessage {
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
            NetworkEvent::ConnectionClosed {
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
        assert!(matches!(opened, NetworkEvent::ConnectionOpened { .. }));

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
            NetworkEvent::ConnectionClosed {
                connection_id: closed_id,
                reason: ConnectionCloseReason::ProtocolError(_),
            } if closed_id == connection_id
        ));

        actor_task.await?;
        Ok(())
    }
}
