// WebSocket handlers and websocket-specific helpers.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};

use crate::server::state::{subscribe_connection, AppState};
use owo_colors::OwoColorize;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| manage_websocket(socket, state))
}

async fn manage_websocket(mut socket: WebSocket, state: AppState) {
    let hello = format!("{} {}", "[CONNECT]".bold().green(), "Client".bold());
    tracing::info!("{}", hello);

    // Local channel for targeted messages to this websocket connection.
    // The sender is stored in AppState so other components can send only to the local UI.
    let (local_tx, mut local_rx) = mpsc::channel::<mcg_shared::Backend2FrontendMsg>(16);
    {
        let mut guard = state.local_frontend_tx.write().await;
        *guard = Some(local_tx);
    }

    let mut subscription: Option<broadcast::Receiver<mcg_shared::Backend2FrontendMsg>> = None;

    loop {
        if let Some(rx) = subscription.as_mut() {
            tokio::select! {
                biased;
                // Prefer broadcasted state messages (existing behaviour)
                recv = rx.recv() => {
                    match recv {
                        Ok(sm) => {
                            send_ws(&mut socket, &sm).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                // Local targeted messages for this websocket only
                local = local_rx.recv() => {
                    if let Some(msg) = local {
                        send_ws(&mut socket, &msg).await;
                    } else {
                        // local sender was dropped; nothing to do, continue
                    }
                }
                // Incoming websocket frames
                msg = socket.next() => {
                    if !process_websocket_frame(&state, &mut socket, &mut subscription, msg).await {
                        break;
                    }
                }
            }
        } else {
            // No subscription yet: still accept local targeted messages and socket frames.
            tokio::select! {
                local = local_rx.recv() => {
                    if let Some(msg) = local {
                        send_ws(&mut socket, &msg).await;
                    } else {
                        // local sender was dropped; continue
                    }
                }
                msg = socket.next() => {
                    if !process_websocket_frame(&state, &mut socket, &mut subscription, msg).await {
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("client disconnecting: websocket client");

    // Cleanup: remove registered local sender so other tasks stop sending to this connection.
    {
        let mut guard = state.local_frontend_tx.write().await;
        *guard = None;
    }
}

async fn send_ws(socket: &mut WebSocket, msg: &mcg_shared::Backend2FrontendMsg) {
    match serde_json::to_string(msg) {
        Ok(txt) => {
            let _ = socket.send(Message::Text(txt)).await;
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to serialize Backend2FrontendMsg for websocket send");
        }
    }
}

async fn process_websocket_frame(
    state: &AppState,
    socket: &mut WebSocket,
    subscription: &mut Option<broadcast::Receiver<mcg_shared::Backend2FrontendMsg>>,
    msg: Option<Result<Message, axum::Error>>,
) -> bool {
    match msg {
        Some(Ok(Message::Text(txt))) => {
            process_websocket_text(state, socket, subscription, txt).await;
            true
        }
        Some(Ok(Message::Binary(_))) => true,
        Some(Ok(Message::Close(_))) | Some(Err(_)) | None => false,
        _ => true,
    }
}

async fn process_websocket_text(
    state: &AppState,
    socket: &mut WebSocket,
    subscription: &mut Option<broadcast::Receiver<mcg_shared::Backend2FrontendMsg>>,
    txt: String,
) {
    match serde_json::from_str::<mcg_shared::Frontend2BackendMsg>(&txt) {
        Ok(mcg_shared::Frontend2BackendMsg::Subscribe) => {
            if subscription.is_some() {
                send_ws(
                    socket,
                    &mcg_shared::Backend2FrontendMsg::Error("already subscribed".into()),
                )
                .await;
                return;
            }
            let sub = subscribe_connection(state).await;
            if let Some(gs) = sub.initial_state {
                send_ws(socket, &mcg_shared::Backend2FrontendMsg::State(gs)).await;
            }
            *subscription = Some(sub.receiver);
        }
        Ok(other) => {
            let resp = crate::server::dispatch_client_message(state, other).await;
            send_ws(socket, &resp).await;
        }
        Err(err) => {
            tracing::warn!(error = %err, "failed to parse incoming Frontend2BackendMsg JSON");
            send_ws(
                socket,
                &mcg_shared::Backend2FrontendMsg::Error("Malformed Frontend2BackendMsg JSON".into()),
            )
            .await;
        }
    }
}
