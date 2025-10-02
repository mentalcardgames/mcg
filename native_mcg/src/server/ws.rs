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

use owo_colors::OwoColorize;
use std::io::IsTerminal;

use crate::server::state::AppState;
use crate::pretty;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Immediately accept the connection and perform a lightweight handshake.
    // Do NOT require the first client message to be `NewGame` — transports
    // should be able to send any supported `ClientMsg` after receiving a
    // `Welcome`/initial `State` from the server. We keep the default viewer
    // id as 0 for now; transports or future session managers can assign
    // concrete player ids later without changing transport handlers.
    let hello = format!("{} {}", "[CONNECT]".bold().green(), "Client".bold(),);
    tracing::info!(%hello);

    // Send welcome + initial state immediately (mirrors iroh behaviour).
    let _ = send_ws(&mut socket, &mcg_shared::ServerMsg::Welcome).await;
    send_state_to(&mut socket, &state).await;

    // Subscribe to broadcasts so this socket receives state updates produced by other connections.
    let mut rx = state.broadcaster.subscribe();

    loop {
        tokio::select! {
            biased;

            // Broadcast messages from server-wide channel
            biased_recv = rx.recv() => {
                match biased_recv {
                    Ok(sm) => {
                        // Forward all ServerMsg values unchanged. The backend no longer
                        // maintains a notion of a per-connection primary player or a
                        // personalized view — transports should forward server messages
                        // as-is to their clients.
                        send_ws(&mut socket, &sm).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // We missed messages; continue and try to catch up on next send.
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // Broadcaster closed - treat as shutdown
                        break;
                    }
                }
            }

            // Incoming websocket messages from this client
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(cm) = serde_json::from_str::<mcg_shared::ClientMsg>(&txt) {
                            process_client_msg(&state, &mut socket, cm).await;
                        } else {
                            tracing::warn!("failed to parse incoming ClientMsg JSON");
                            // Debug: log raw incoming text so we can see what the client actually sends.
                            // This helps diagnose why client actions may not be parsed/handled.
                            tracing::debug!(raw_in = %txt);
                            // Optionally send an error back to client for visibility
                            let _ = send_ws(&mut socket, &mcg_shared::ServerMsg::Error("Malformed ClientMsg JSON".into())).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }
    tracing::info!("client disconnecting: New Game Client");
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

async fn send_state_to(socket: &mut WebSocket, state: &AppState) {
    if let Some(gs) = crate::server::current_state_public(state).await {
        // Only print newly added events since the last print, to avoid repeating "Preflop"
        let mut lobby = state.lobby.write().await;
        let already = lobby.last_printed_log_len;
        let total = gs.action_log.len();
        if total > already {
            for e in gs.action_log.iter().skip(already) {
                let line =
                    pretty::format_event_human(e, &gs.players, std::io::stdout().is_terminal());
                tracing::info!("{}", line);
            }
            lobby.last_printed_log_len = total;
        }
        drop(lobby);
        send_ws(socket, &mcg_shared::ServerMsg::State(gs)).await;
    }
}

async fn process_client_msg(state: &AppState, socket: &mut WebSocket, cm: mcg_shared::ClientMsg) {
    // Delegate handling to the centralized backend handler to ensure consistent behavior.
    tracing::debug!(ws_received_client_msg = ?cm);
    let resp = crate::server::handle_client_msg(state, cm).await;

    // Forward the backend response to this socket unchanged.
    let _ = send_ws(socket, &resp).await;
}
