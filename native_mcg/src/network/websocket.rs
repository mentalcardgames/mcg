use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures::{SinkExt, StreamExt};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg, Peer2PeerMsg, WebSocketPeerHandshake};
use tokio::sync::mpsc;

use super::types::ActorEvent;
use super::{
    ConnectionCloseReason, ConnectionId, FrontendConnectionCommand, PeerConnectionCommand, PeerId,
};

pub const WEBSOCKET_FRONTEND_PROTOCOL: &str = "mcg.frontend";
pub const WEBSOCKET_PEER_PROTOCOL: &str = "mcg.peer";

const PEER_HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Runs one frontend WebSocket connection.
///
/// The actor translates WebSocket frames into typed [`ActorEvent`] values
/// and writes application responses received through its private outbound
/// channel. It deliberately has no access to application state.
pub(crate) async fn run_websocket_frontend_actor(
    connection_id: ConnectionId,
    socket: WebSocket,
    event_tx: mpsc::Sender<ActorEvent>,
    mut outbound_rx: mpsc::Receiver<FrontendConnectionCommand>,
) {
    let (mut writer, mut reader) = socket.split();
    // Respond success with starting the actor loop
    if event_tx
        .send(ActorEvent::Ready { connection_id })
        .await
        .is_err()
    {
        tracing::debug!(%connection_id, "network event receiver dropped before WebSocket actor started");
        return;
    }
    tracing::info!(%connection_id, "WebSocket connection actor started");

    let close_reason = loop {
        tokio::select! {
            incoming = reader.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<Frontend2BackendMsg>(&text) {
                            Ok(message) => {
                                let event = ActorEvent::FrontendMessage {
                                    connection_id,
                                    message,
                                };
                                if event_tx.send(event).await.is_err() {
                                    tracing::debug!(%connection_id, "network event receiver dropped");
                                    break ConnectionCloseReason::EventReceiverClosed;
                                }
                            }
                            Err(error) => {
                                tracing::warn!(%connection_id, %error, "failed to parse frontend WebSocket message");
                                let response = Backend2FrontendMsg::Error(
                                    "Malformed Frontend2BackendMsg JSON".into(),
                                );
                                if let Err(error) = send_backend_message(&mut writer, connection_id, &response).await {
                                    break ConnectionCloseReason::TransportError(error.to_string());
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_)))
                    | Some(Ok(Message::Ping(_)))
                    | Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => {
                        break ConnectionCloseReason::RemoteClosed;
                    }
                    Some(Err(error)) => {
                        break ConnectionCloseReason::TransportError(error.to_string());
                    }
                }
            }
            outgoing = outbound_rx.recv() => {
                match outgoing {
                    Some(FrontendConnectionCommand::Send(message)) => {
                        if let Err(error) = send_backend_message(&mut writer, connection_id, &message).await {
                            break ConnectionCloseReason::TransportError(error.to_string());
                        }
                    }
                    Some(FrontendConnectionCommand::Close { reason }) => {
                        let frame = CloseFrame {
                            code: 1000,
                            reason: reason.clone().into(),
                        };
                        if let Err(error) = writer.send(Message::Close(Some(frame))).await {
                            break ConnectionCloseReason::TransportError(error.to_string());
                        }
                        break ConnectionCloseReason::LocalRequest(reason);
                    }
                    None => {
                        tracing::debug!(%connection_id, "WebSocket outbound channel closed");
                        break ConnectionCloseReason::OutboundChannelClosed;
                    }
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
    tracing::info!(%connection_id, "WebSocket connection actor stopped");
}

/// Runs one incoming peer WebSocket connection.
///
/// Unlike Iroh, a WebSocket does not expose an authenticated endpoint ID. The
/// connection therefore stays pending until its first text frame claims a
/// syntactically valid Iroh endpoint ID. The claim is intentionally not
/// authenticated at this layer.
pub(crate) async fn run_websocket_pending_peer_actor(
    connection_id: ConnectionId,
    socket: WebSocket,
    event_tx: mpsc::Sender<ActorEvent>,
    mut outbound_rx: mpsc::Receiver<PeerConnectionCommand>,
) {
    let (mut writer, mut reader) = socket.split();
    tracing::info!(%connection_id, "pending peer WebSocket actor started");

    let handshake_timeout = tokio::time::sleep(PEER_HANDSHAKE_TIMEOUT);
    tokio::pin!(handshake_timeout);

    let close_reason = loop {
        tokio::select! {
            _ = &mut handshake_timeout => {
                break ConnectionCloseReason::ProtocolError(
                    "peer WebSocket handshake timed out".into(),
                );
            }
            incoming = reader.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        let handshake = match serde_json::from_str::<WebSocketPeerHandshake>(&text) {
                            Ok(handshake) => handshake,
                            Err(error) => {
                                tracing::warn!(%connection_id, %error, "invalid peer WebSocket handshake");
                                break ConnectionCloseReason::ProtocolError(
                                    "expected a valid WebSocket peer handshake".into(),
                                );
                            }
                        };
                        let endpoint_id = match handshake.peer_id.parse::<iroh::EndpointId>() {
                            Ok(endpoint_id) => endpoint_id,
                            Err(error) => {
                                tracing::warn!(%connection_id, %error, "invalid peer ID in WebSocket handshake");
                                break ConnectionCloseReason::ProtocolError(
                                    "WebSocket peer handshake contains an invalid peer ID".into(),
                                );
                            }
                        };
                        let peer_id = PeerId::new(endpoint_id.to_string());
                        if event_tx
                            .send(ActorEvent::PeerIdentified {
                                connection_id,
                                peer_id,
                            })
                            .await
                            .is_err()
                        {
                            break ConnectionCloseReason::EventReceiverClosed;
                        }
                        break run_established_peer_loop(
                            connection_id,
                            &mut writer,
                            &mut reader,
                            &event_tx,
                            &mut outbound_rx,
                        )
                        .await;
                    }
                    Some(Ok(Message::Binary(_)))
                    | Some(Ok(Message::Ping(_)))
                    | Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => {
                        break ConnectionCloseReason::RemoteClosed;
                    }
                    Some(Err(error)) => {
                        break ConnectionCloseReason::TransportError(error.to_string());
                    }
                }
            }
            outgoing = outbound_rx.recv() => {
                match outgoing {
                    Some(PeerConnectionCommand::Close { reason }) => {
                        if let Err(error) = send_close_frame(&mut writer, &reason).await {
                            break ConnectionCloseReason::TransportError(error.to_string());
                        }
                        break ConnectionCloseReason::LocalRequest(reason);
                    }
                    Some(PeerConnectionCommand::Send(_)) => {
                        break ConnectionCloseReason::ProtocolError(
                            "peer message queued before WebSocket handshake completed".into(),
                        );
                    }
                    None => break ConnectionCloseReason::OutboundChannelClosed,
                }
            }
        }
    };

    if matches!(close_reason, ConnectionCloseReason::ProtocolError(_)) {
        let _ = send_protocol_close(&mut writer, &close_reason).await;
    }
    let _ = event_tx
        .send(ActorEvent::Closed {
            connection_id,
            reason: close_reason,
        })
        .await;
    tracing::info!(%connection_id, "peer WebSocket actor stopped");
}

async fn run_established_peer_loop(
    connection_id: ConnectionId,
    writer: &mut futures::stream::SplitSink<WebSocket, Message>,
    reader: &mut futures::stream::SplitStream<WebSocket>,
    event_tx: &mpsc::Sender<ActorEvent>,
    outbound_rx: &mut mpsc::Receiver<PeerConnectionCommand>,
) -> ConnectionCloseReason {
    loop {
        tokio::select! {
            incoming = reader.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<Peer2PeerMsg>(&text) {
                            Ok(message) => {
                                if event_tx
                                    .send(ActorEvent::PeerMessage { connection_id, message })
                                    .await
                                    .is_err()
                                {
                                    return ConnectionCloseReason::EventReceiverClosed;
                                }
                            }
                            Err(error) => {
                                tracing::warn!(%connection_id, %error, "failed to parse peer WebSocket message");
                                return ConnectionCloseReason::ProtocolError(
                                    "malformed Peer2PeerMsg JSON".into(),
                                );
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_)))
                    | Some(Ok(Message::Ping(_)))
                    | Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => {
                        return ConnectionCloseReason::RemoteClosed;
                    }
                    Some(Err(error)) => {
                        return ConnectionCloseReason::TransportError(error.to_string());
                    }
                }
            }
            outgoing = outbound_rx.recv() => {
                match outgoing {
                    Some(PeerConnectionCommand::Send(message)) => {
                        if let Err(error) = send_peer_message(writer, connection_id, &message).await {
                            return ConnectionCloseReason::TransportError(error.to_string());
                        }
                    }
                    Some(PeerConnectionCommand::Close { reason }) => {
                        if let Err(error) = send_close_frame(writer, &reason).await {
                            return ConnectionCloseReason::TransportError(error.to_string());
                        }
                        return ConnectionCloseReason::LocalRequest(reason);
                    }
                    None => return ConnectionCloseReason::OutboundChannelClosed,
                }
            }
        }
    }
}

async fn send_peer_message(
    writer: &mut futures::stream::SplitSink<WebSocket, Message>,
    connection_id: ConnectionId,
    message: &Peer2PeerMsg,
) -> Result<(), axum::Error> {
    let text = match serde_json::to_string(message) {
        Ok(text) => text,
        Err(error) => {
            tracing::error!(%connection_id, %error, "failed to serialize peer WebSocket message");
            return Ok(());
        }
    };
    writer.send(Message::Text(text)).await
}

async fn send_close_frame(
    writer: &mut futures::stream::SplitSink<WebSocket, Message>,
    reason: &str,
) -> Result<(), axum::Error> {
    writer
        .send(Message::Close(Some(CloseFrame {
            code: 1000,
            reason: reason.to_owned().into(),
        })))
        .await
}

async fn send_protocol_close(
    writer: &mut futures::stream::SplitSink<WebSocket, Message>,
    reason: &ConnectionCloseReason,
) -> Result<(), axum::Error> {
    let ConnectionCloseReason::ProtocolError(reason) = reason else {
        return Ok(());
    };
    writer
        .send(Message::Close(Some(CloseFrame {
            code: 1002,
            reason: reason.clone().into(),
        })))
        .await
}

async fn send_backend_message(
    writer: &mut futures::stream::SplitSink<WebSocket, Message>,
    connection_id: ConnectionId,
    message: &Backend2FrontendMsg,
) -> Result<(), axum::Error> {
    let text = match serde_json::to_string(message) {
        Ok(text) => text,
        Err(error) => {
            tracing::error!(%connection_id, %error, "failed to serialize backend WebSocket message");
            return Ok(());
        }
    };

    writer.send(Message::Text(text)).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::Result;
    use axum::{
        extract::{ws::WebSocketUpgrade, State},
        response::IntoResponse,
        routing::get,
        Router,
    };
    use futures::{SinkExt, StreamExt};
    use tokio::sync::Mutex;
    use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

    use super::*;

    #[derive(Clone)]
    struct TestState {
        event_tx: mpsc::Sender<ActorEvent>,
        outbound_rx: Arc<Mutex<Option<mpsc::Receiver<FrontendConnectionCommand>>>>,
    }

    async fn test_ws_handler(
        ws: WebSocketUpgrade,
        State(state): State<TestState>,
    ) -> impl IntoResponse {
        let outbound_rx = state
            .outbound_rx
            .lock()
            .await
            .take()
            .expect("test accepts exactly one WebSocket connection");

        ws.on_upgrade(move |socket| {
            run_websocket_frontend_actor(ConnectionId::new(17), socket, state.event_tx, outbound_rx)
        })
    }

    #[tokio::test]
    async fn actor_translates_messages_in_both_directions_and_reports_close() -> Result<()> {
        let (event_tx, mut event_rx) = mpsc::channel(8);
        let (outbound_tx, outbound_rx) = mpsc::channel(8);
        let state = TestState {
            event_tx,
            outbound_rx: Arc::new(Mutex::new(Some(outbound_rx))),
        };
        let app = Router::new()
            .route("/ws", get(test_ws_handler))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server = tokio::spawn(async move { axum::serve(listener, app).await });

        let url = format!("ws://{address}/ws");
        let (mut client, _) = tokio_tungstenite::connect_async(url).await?;

        let event = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should report the open connection");
        assert!(matches!(
            event,
            ActorEvent::Ready { connection_id } if connection_id == ConnectionId::new(17)
        ));

        client
            .send(TungsteniteMessage::Text("not valid JSON".into()))
            .await?;
        let malformed_response = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("WebSocket should remain open")?;
        let TungsteniteMessage::Text(malformed_response) = malformed_response else {
            panic!("expected a text response for malformed JSON");
        };
        assert!(matches!(
            serde_json::from_str::<Backend2FrontendMsg>(&malformed_response)?,
            Backend2FrontendMsg::Error(message)
                if message == "Malformed Frontend2BackendMsg JSON"
        ));

        client
            .send(TungsteniteMessage::Text(serde_json::to_string(
                &Frontend2BackendMsg::Ping,
            )?))
            .await?;
        let event = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should emit a frontend event");
        assert!(matches!(
            event,
            ActorEvent::FrontendMessage {
                connection_id,
                message: Frontend2BackendMsg::Ping,
            } if connection_id == ConnectionId::new(17)
        ));

        outbound_tx
            .send(FrontendConnectionCommand::Send(Backend2FrontendMsg::Pong))
            .await?;
        let outbound_message = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("WebSocket should remain open")?;
        let TungsteniteMessage::Text(outbound_message) = outbound_message else {
            panic!("expected a text response from the outbound channel");
        };
        assert!(matches!(
            serde_json::from_str::<Backend2FrontendMsg>(&outbound_message)?,
            Backend2FrontendMsg::Pong
        ));

        outbound_tx
            .send(FrontendConnectionCommand::Close {
                reason: "test shutdown".into(),
            })
            .await?;
        let close_message = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("WebSocket should send a close frame")?;
        assert!(matches!(close_message, TungsteniteMessage::Close(_)));

        let event = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should report the closed connection");
        assert!(matches!(
            event,
            ActorEvent::Closed {
                connection_id,
                reason: ConnectionCloseReason::LocalRequest(reason),
            } if connection_id == ConnectionId::new(17)
                && reason == "test shutdown"
        ));

        server.abort();
        Ok(())
    }
}
