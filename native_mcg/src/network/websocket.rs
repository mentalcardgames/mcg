use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use tokio::sync::mpsc;

use super::{ConnectionId, NetworkEvent};

/// Runs one frontend WebSocket connection.
///
/// The actor translates WebSocket frames into typed [`NetworkEvent`] values
/// and writes application responses received through its private outbound
/// channel. It deliberately has no access to application state.
pub async fn run_websocket_actor(
    connection_id: ConnectionId,
    socket: WebSocket,
    event_tx: mpsc::Sender<NetworkEvent>,
    mut outbound_rx: mpsc::Receiver<Backend2FrontendMsg>,
) {
    let (mut writer, mut reader) = socket.split();
    tracing::info!(%connection_id, "WebSocket connection actor started");

    loop {
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
                                    break;
                                }
                            }
                            Err(error) => {
                                tracing::warn!(%connection_id, %error, "failed to parse frontend WebSocket message");
                                let response = Backend2FrontendMsg::Error(
                                    "Malformed Frontend2BackendMsg JSON".into(),
                                );
                                if !send_backend_message(&mut writer, connection_id, &response).await {
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_)))
                    | Some(Ok(Message::Ping(_)))
                    | Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                }
            }
            outgoing = outbound_rx.recv() => {
                match outgoing {
                    Some(message) => {
                        if !send_backend_message(&mut writer, connection_id, &message).await {
                            break;
                        }
                    }
                    None => {
                        tracing::debug!(%connection_id, "WebSocket outbound channel closed");
                        break;
                    }
                }
            }
        }
    }

    let _ = event_tx
        .send(NetworkEvent::ConnectionClosed { connection_id })
        .await;
    tracing::info!(%connection_id, "WebSocket connection actor stopped");
}

async fn send_backend_message(
    writer: &mut futures::stream::SplitSink<WebSocket, Message>,
    connection_id: ConnectionId,
    message: &Backend2FrontendMsg,
) -> bool {
    let text = match serde_json::to_string(message) {
        Ok(text) => text,
        Err(error) => {
            tracing::error!(%connection_id, %error, "failed to serialize backend WebSocket message");
            return true;
        }
    };

    if let Err(error) = writer.send(Message::Text(text)).await {
        tracing::debug!(%connection_id, %error, "failed to write WebSocket message");
        return false;
    }

    true
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
        outbound_rx: Arc<Mutex<Option<mpsc::Receiver<Backend2FrontendMsg>>>>,
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

        outbound_tx.send(Backend2FrontendMsg::Pong).await?;
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

        client.close(None).await?;
        let event = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("actor should report the closed connection");
        assert!(matches!(
            event,
            NetworkEvent::ConnectionClosed { connection_id }
                if connection_id == ConnectionId::new(17)
        ));

        server.abort();
        Ok(())
    }
}
