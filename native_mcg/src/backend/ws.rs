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

use super::state::AppState;
use crate::pretty;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let name = match socket.next().await {
        Some(Ok(Message::Text(t))) => match serde_json::from_str::<mcg_shared::ClientMsg>(&t) {
            Ok(mcg_shared::ClientMsg::Join { name }) => name,
            _ => {
                send_ws(
                    &mut socket,
                    &mcg_shared::ServerMsg::Error("Expected Join".into()),
                )
                .await;
                return;
            }
        },
        _ => return,
    };
    let hello = format!("{} {}", "[CONNECT]".bold().green(), name.bold());
    println!("{}", hello);

    if let Err(e) = super::state::ensure_game_started(&state, &name).await {
        let _ = send_ws(
            &mut socket,
            &mcg_shared::ServerMsg::Error(format!("Failed to start game: {}", e)),
        )
        .await;
        return;
    }

    let you_id = 0usize;
    send_ws(&mut socket, &mcg_shared::ServerMsg::Welcome { you: you_id }).await;
    // Send initial state directly to this socket (does local printing & bookkeeping).
    send_state_to(&mut socket, &state, you_id).await;

    // Subscribe to broadcasts so this socket receives state updates produced by other connections.
    let mut rx = state.broadcaster.subscribe();

    loop {
        tokio::select! {
            biased;

            // Broadcast messages from server-wide channel
            biased_recv = rx.recv() => {
                match biased_recv {
                    Ok(sm) => {
                        // Forward to connected socket. Ignore send failures.
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
                            process_client_msg(&name, &state, &mut socket, cm, you_id).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }
    println!("{} {}", "[DISCONNECT]".bold().red(), name.bold());
}

async fn send_ws(socket: &mut WebSocket, msg: &mcg_shared::ServerMsg) {
    match serde_json::to_string(msg) {
        Ok(txt) => {
            let _ = socket.send(Message::Text(txt)).await;
        }
        Err(e) => {
            eprintln!("Failed to serialize ServerMsg for websocket send: {}", e);
        }
    }
}

async fn send_state_to(socket: &mut WebSocket, state: &AppState, you_id: usize) {
    if let Some(gs) = super::state::current_state_public(state, you_id).await {
        // Only print newly added events since the last print, to avoid repeating "Preflop"
        let mut lobby = state.lobby.write().await;
        let already = lobby.last_printed_log_len;
        let total = gs.action_log.len();
        if total > already {
            for e in gs.action_log.iter().skip(already) {
                let line = pretty::format_event_human(
                    e,
                    &gs.players,
                    gs.you_id,
                    std::io::stdout().is_terminal(),
                );
                println!("{}", line);
            }
            lobby.last_printed_log_len = total;
        }
        drop(lobby);
        send_ws(socket, &mcg_shared::ServerMsg::State(gs)).await;
    }
}

async fn process_client_msg(
    name: &str,
    state: &AppState,
    socket: &mut WebSocket,
    cm: mcg_shared::ClientMsg,
    you_id: usize,
) {
    match cm {
        // ClientMsg::Action now has form: Action { player_id, action }
        mcg_shared::ClientMsg::Action { player_id, action } => {
            println!("[WS] Action from {}: player_id={} action={:?}", name, player_id, action);

            match super::state::validate_and_apply_action(state, player_id, action.clone()).await {
                Ok(()) => {
                    // Send updated state immediately to the originating socket so the client sees its own action
                    send_state_to(socket, state, player_id).await;
                    // Broadcast and drive via centralized helper
                    super::state::broadcast_and_drive(state, you_id, 500, 1500).await;
                }
                Err(e) => {
                    let _ = send_ws(socket, &mcg_shared::ServerMsg::Error(e)).await;
                }
            }
        }

        // RequestState { player_id }
        mcg_shared::ClientMsg::RequestState { player_id } => {
            println!("[WS] State requested by {} for player {}", name, player_id);
            send_state_to(socket, state, player_id).await;
            super::state::broadcast_and_drive(state, you_id, 500, 1500).await;
        }

        // NextHand { player_id }
        mcg_shared::ClientMsg::NextHand { player_id } => {
            println!("[WS] NextHand requested by {} for player {}", name, player_id);
            if let Err(e) = super::state::start_new_hand_and_print(state, player_id).await {
                let _ = send_ws(
                    socket,
                    &mcg_shared::ServerMsg::Error(format!("Failed to start new hand: {}", e)),
                )
                .await;
            }
            // Send updated state to the requesting socket, then broadcast and drive bots.
            send_state_to(socket, state, player_id).await;
            super::state::broadcast_and_drive(state, you_id, 500, 1500).await;
        }

        // ResetGame { bots, bots_auto }
        mcg_shared::ClientMsg::ResetGame { bots, bots_auto } => {
            println!("[WS] ResetGame requested by {}: bots={} bots_auto={}", name, bots, bots_auto);

            // Persist the bots_auto preference in lobby so drive_bots can observe it.
            {
                let mut lobby_w = state.lobby.write().await;
                lobby_w.bots_auto = bots_auto;
            }

            if let Err(e) = super::state::reset_game_with_bots(state, name, bots, you_id).await {
                let _ = send_ws(
                    socket,
                    &mcg_shared::ServerMsg::Error(format!("Failed to reset game: {}", e)),
                )
                .await;
            } else {
                // Send updated state to the requesting socket, then broadcast to others.
                send_state_to(socket, state, you_id).await;
                super::state::broadcast_and_drive(state, you_id, 500, 1500).await;
            }
        }

        mcg_shared::ClientMsg::Join { .. } => {}
    }
}
