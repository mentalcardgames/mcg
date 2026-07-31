use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures::{SinkExt, StreamExt};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use tokio::sync::mpsc;

use super::{
    ConnectionCloseReason, ConnectionId, ConnectionInfo, ConnectionRole, FrontendConnectionCommand,
    NetworkEvent, TransportKind,
};

/// Runs one frontend WebSocket connection.
///
/// The actor translates WebSocket frames into typed [`NetworkEvent`] values
/// and writes application responses received through its private outbound
/// channel. It deliberately has no access to application state.
pub async fn run_websocket_actor(
    connection_id: ConnectionId,
    socket: WebSocket,
    event_tx: mpsc::Sender<NetworkEvent>,
    mut outbound_rx: mpsc::Receiver<FrontendConnectionCommand>,
) {
    let (mut writer, mut reader) = socket.split();
    let connection = ConnectionInfo {
        id: connection_id,
        role: ConnectionRole::Frontend,
        transport: TransportKind::WebSocket,
    };

    // Respond success with starting the actor loop
    if event_tx
        .send(NetworkEvent::ConnectionOpened { connection })
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
                                let event = NetworkEvent::FrontendMessage {
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
        .send(NetworkEvent::ConnectionClosed {
            connection_id,
            reason: close_reason,
        })
        .await;
    tracing::info!(%connection_id, "WebSocket connection actor stopped");
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
        event_tx: mpsc::Sender<NetworkEvent>,
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
            run_websocket_actor(ConnectionId::new(17), socket, state.event_tx, outbound_rx)
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
            NetworkEvent::ConnectionOpened {
                connection: ConnectionInfo {
                    id,
                    role: ConnectionRole::Frontend,
                    transport: TransportKind::WebSocket,
                }
            } if id == ConnectionId::new(17)
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
            NetworkEvent::FrontendMessage {
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
            NetworkEvent::ConnectionClosed {
                connection_id,
                reason: ConnectionCloseReason::LocalRequest(reason),
            } if connection_id == ConnectionId::new(17)
                && reason == "test shutdown"
        ));

        server.abort();
        Ok(())
    }
}
