// WebSocket handlers and websocket-specific helpers.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::StreamExt;
use tokio::sync::broadcast;

use crate::server::state::{subscribe_connection, AppState};
use owo_colors::OwoColorize;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let hello = format!("{} {}", "[CONNECT]".bold().green(), "Client".bold());
    tracing::info!("{}", hello);

    let mut subscription: Option<broadcast::Receiver<mcg_shared::ServerMsg>> = None;

    loop {
        if let Some(rx) = subscription.as_mut() {
            tokio::select! {
                biased;
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
                msg = socket.next() => {
                    if !handle_socket_message(&state, &mut socket, &mut subscription, msg).await {
                        break;
                    }
                }
            }
        } else {
            let msg = socket.next().await;
            if !handle_socket_message(&state, &mut socket, &mut subscription, msg).await {
                break;
            }
        }
    }
    tracing::info!("client disconnecting: websocket client");
}

async fn send_ws(socket: &mut WebSocket, msg: &mcg_shared::ServerMsg) {
    match serde_json::to_string(msg) {
        Ok(txt) => {
            let _ = socket.send(Message::Text(txt)).await;
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to serialize ServerMsg for websocket send");
        }
    }
}

async fn handle_socket_message(
    state: &AppState,
    socket: &mut WebSocket,
    subscription: &mut Option<broadcast::Receiver<mcg_shared::ServerMsg>>,
    msg: Option<Result<Message, axum::Error>>,
) -> bool {
    match msg {
        Some(Ok(Message::Text(txt))) => {
            handle_client_text(state, socket, subscription, txt).await;
            true
        }
        Some(Ok(Message::Binary(_))) => true,
        Some(Ok(Message::Close(_))) | Some(Err(_)) | None => false,
        _ => true,
    }
}

async fn handle_client_text(
    state: &AppState,
    socket: &mut WebSocket,
    subscription: &mut Option<broadcast::Receiver<mcg_shared::ServerMsg>>,
    txt: String,
) {
    match serde_json::from_str::<mcg_shared::ClientMsg>(&txt) {
        Ok(mcg_shared::ClientMsg::Subscribe) => {
            if subscription.is_some() {
                send_ws(
                    socket,
                    &mcg_shared::ServerMsg::Error("already subscribed".into()),
                )
                .await;
                return;
            }
            let sub = subscribe_connection(state).await;
            if let Some(gs) = sub.initial_state {
                send_ws(socket, &mcg_shared::ServerMsg::State(gs)).await;
            }
            *subscription = Some(sub.receiver);
        }
        Ok(other) => {
            let resp = crate::server::handle_client_msg(state, other).await;
            send_ws(socket, &resp).await;
        }
        Err(err) => {
            tracing::warn!(error = %err, "failed to parse incoming ClientMsg JSON");
            send_ws(
                socket,
                &mcg_shared::ServerMsg::Error("Malformed ClientMsg JSON".into()),
            )
            .await;
        }
    }
}
