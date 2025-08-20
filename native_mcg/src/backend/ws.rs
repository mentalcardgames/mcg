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
    let primary_player_id = match socket.next().await {
        Some(Ok(Message::Text(t))) => match serde_json::from_str::<mcg_shared::ClientMsg>(&t) {
            Ok(mcg_shared::ClientMsg::NewGame { players }) => {
                // Create new game with the specified players
                if let Err(e) = super::state::create_new_game(&state, players).await {
                    let _ = send_ws(
                        &mut socket,
                        &mcg_shared::ServerMsg::Error(format!("Failed to create new game: {}", e)),
                    )
                    .await;
                    return;
                }
                mcg_shared::PlayerId(0) // Default player ID for initial state
            }
            _ => {
                send_ws(
                    &mut socket,
                    &mcg_shared::ServerMsg::Error("Expected NewGame message".into()),
                )
                .await;
                return;
            }
        },
        _ => return,
    };

    let hello = format!(
        "{} {} (primary player: {})",
        "[CONNECT]".bold().green(),
        "New Game".bold(),
        primary_player_id
    );
    println!("{}", hello);

    send_ws(
        &mut socket,
        &mcg_shared::ServerMsg::Welcome {
            you: primary_player_id,
        },
    )
    .await;
    // Send initial state directly to this socket (does local printing & bookkeeping).
    send_state_to(&mut socket, &state).await;
    // After creating a new game via the initial NewGame handshake, trigger broadcast
    // and bot driving so server-side bots begin acting without requiring an extra
    // client message (fixes stuck-game where bots don't advance play).
    super::state::broadcast_and_drive(&state, 500, 1500).await;

    // Subscribe to broadcasts so this socket receives state updates produced by other connections.
    let mut rx = state.broadcaster.subscribe();

    loop {
        tokio::select! {
            biased;

            // Broadcast messages from server-wide channel
            biased_recv = rx.recv() => {
                match biased_recv {
                    Ok(sm) => {
                        // If this is a State message, re-compute a personalized view
                        // for this socket's primary_player_id before sending. This ensures
                        // we don't broadcast a state that was generated for another viewer
                        // (which would incorrectly change `you`/card visibility on recipients).
                        match sm {
                            mcg_shared::ServerMsg::State(_) => {
                                // Re-send a viewer-specific state to this socket.
                                // send_state_to will call current_state_public(viewer_id)
                                // so the you_id and card visibility are correct per-client.
                                send_state_to(&mut socket, &state ).await;
                            }
                            _ => {
                                // Forward other server messages unchanged.
                                send_ws(&mut socket, &sm).await;
                            }
                        }
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
                            process_client_msg(&state, &mut socket, cm, primary_player_id).await;
                        } else {
                            eprintln!("[WS] Failed to parse incoming ClientMsg JSON");
                            // Debug: log raw incoming text so we can see what the client actually sends.
                            // This helps diagnose why client actions may not be parsed/handled.
                            println!("[WS RAW IN] {}", txt);
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
    println!(
        "{} {}",
        "[DISCONNECT]".bold().red(),
        "New Game Client".bold()
    );
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

async fn send_state_to(socket: &mut WebSocket, state: &AppState) {
    if let Some(gs) = super::state::current_state_public(state).await {
        // Only print newly added events since the last print, to avoid repeating "Preflop"
        let mut lobby = state.lobby.write().await;
        let already = lobby.last_printed_log_len;
        let total = gs.action_log.len();
        if total > already {
            for e in gs.action_log.iter().skip(already) {
                let line =
                    pretty::format_event_human(e, &gs.players, std::io::stdout().is_terminal());
                println!("{}", line);
            }
            lobby.last_printed_log_len = total;
        }
        drop(lobby);
        send_ws(socket, &mcg_shared::ServerMsg::State(gs)).await;
    }
}

async fn process_client_msg(
    state: &AppState,
    socket: &mut WebSocket,
    cm: mcg_shared::ClientMsg,
    primary_player_id: mcg_shared::PlayerId,
) {
    match cm {
        // ClientMsg::Action now has form: Action { player_id, action }
        mcg_shared::ClientMsg::Action { player_id, action } => {
            println!(
                "[WS] Action from primary_player_id={}: player_id={} action={:?}",
                primary_player_id, player_id, action
            );

            // Check if there's an active game before processing actions
            {
                let lobby = state.lobby.read().await;
                if lobby.game.is_none() {
                    let _ = send_ws(
                        socket,
                        &mcg_shared::ServerMsg::Error(
                            "No active game. Please start a new game first.".into(),
                        ),
                    )
                    .await;
                    return;
                }
            }

            match super::state::validate_and_apply_action(state, player_id.into(), action.clone())
                .await
            {
                Ok(()) => {
                    // Send updated state immediately to the originating socket so the client sees its own action
                    send_state_to(socket, state).await;
                    // Broadcast and drive via centralized helper
                    super::state::broadcast_and_drive(state, 500, 1500).await;
                }
                Err(e) => {
                    let _ = send_ws(socket, &mcg_shared::ServerMsg::Error(e)).await;
                }
            }
        }

        // RequestState { player_id }
        mcg_shared::ClientMsg::RequestState => {
            println!(
                "[WS] State requested by primary_player_id={}",
                primary_player_id
            );
            send_state_to(socket, state).await;
            super::state::broadcast_and_drive(state, 500, 1500).await;
        }

        // NextHand { player_id }
        mcg_shared::ClientMsg::NextHand => {
            println!(
                "[WS] NextHand requested by primary_player_id={} ",
                primary_player_id
            );

            // Check if there's an active game before processing NextHand
            {
                let lobby = state.lobby.read().await;
                if lobby.game.is_none() {
                    let _ = send_ws(
                        socket,
                        &mcg_shared::ServerMsg::Error(
                            "No active game. Please start a new game first.".into(),
                        ),
                    )
                    .await;
                    return;
                }
            }

            if let Err(e) = super::state::start_new_hand_and_print(state).await {
                let _ = send_ws(
                    socket,
                    &mcg_shared::ServerMsg::Error(format!("Failed to start new hand: {}", e)),
                )
                .await;
            } else {
                // Send updated state to the requesting socket, then broadcast and drive bots.
                send_state_to(socket, state).await;
                super::state::broadcast_and_drive(state, 500, 1500).await;
            }
        }

        // NewGame message - can override current game at any time
        mcg_shared::ClientMsg::NewGame { players } => {
            println!(
                "[WS] NewGame requested by {} with {} players",
                primary_player_id,
                players.len()
            );
            if let Err(e) = super::state::create_new_game(state, players).await {
                let _ = send_ws(
                    socket,
                    &mcg_shared::ServerMsg::Error(format!("Failed to create new game: {}", e)),
                )
                .await;
            } else {
                send_state_to(socket, state).await;
                super::state::broadcast_and_drive(state, 500, 1500).await;
            }
        } // Note: Join and ResetGame have been removed from ClientMsg enum
    }
}
