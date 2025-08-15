//! Server networking and WebSocket handling.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::StreamExt;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::game::Game;
use mcg_server::pretty::format_table_header;
use mcg_shared::{ClientMsg, GameStatePublic, ServerMsg};
use owo_colors::OwoColorize;
use std::io::IsTerminal;

#[derive(Clone, Default)]
pub struct AppState {
    pub lobby: Arc<RwLock<Lobby>>,
    pub bot_count: usize,
}

#[derive(Clone, Default)]
pub(crate) struct Lobby {
    game: Option<Game>,
    last_printed_log_len: usize,
}

pub fn build_router(state: AppState) -> Router {
    // Serve static files from the project root. Assumes process CWD is repo root.
    let serve_dir = ServeDir::new("pkg").append_index_html_on_directories(true);
    let serve_media = ServeDir::new("media").append_index_html_on_directories(true);

    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true })) }),
        )
        .route("/ws", get(ws_handler))
        .nest_service("/pkg", serve_dir)
        .nest_service("/media", serve_media)
        // Serve index.html for the root route
        .route("/", get(serve_index))
        // Fallback handler for SPA routing - serve index.html for all other routes
        .fallback(spa_handler)
        .with_state(state)
}

pub async fn run_server(addr: SocketAddr, state: AppState) {
    let app = build_router(state.clone());

    let display_addr = if addr.ip().to_string() == "127.0.0.1" {
        format!("localhost:{}", addr.port())
    } else {
        addr.to_string()
    };

    println!("🌐 MCG Server running at http://{}", display_addr);
    println!("📱 Open your browser and navigate to the above URL");
    println!();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let name = match socket.next().await {
        Some(Ok(Message::Text(t))) => match serde_json::from_str::<ClientMsg>(&t) {
            Ok(ClientMsg::Join { name }) => name,
            _ => {
                send_ws(&mut socket, &ServerMsg::Error("Expected Join".into())).await;
                return;
            }
        },
        _ => return,
    };
    let hello = format!("{} {}", "[CONNECT]".bold().green(), name.bold());
    println!("{}", hello);

    ensure_game_started(&state, &name).await;

    let you_id = 0usize;
    send_ws(&mut socket, &ServerMsg::Welcome { you: you_id }).await;
    send_state_to(&mut socket, &state, you_id).await;

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(txt)) => {
                if let Ok(cm) = serde_json::from_str::<ClientMsg>(&txt) {
                    process_client_msg(&name, &state, &mut socket, cm, you_id).await;
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
    println!("{} {}", "[DISCONNECT]".bold().red(), name.bold());
}

async fn ensure_game_started(state: &AppState, name: &str) {
    let mut lobby = state.lobby.write().await;
    if lobby.game.is_none() {
        lobby.game = Some(Game::new(name.to_string(), state.bot_count));
        println!(
            "[GAME] Created new game for {} with {} bot(s)",
            name, state.bot_count
        );
    }
}

async fn send_ws(socket: &mut WebSocket, msg: &ServerMsg) {
    let _ = socket
        .send(Message::Text(serde_json::to_string(msg).unwrap()))
        .await;
}

async fn send_state_to(socket: &mut WebSocket, state: &AppState, you_id: usize) {
    if let Some(gs) = current_state_public(state, you_id).await {
        // Only print newly added events since the last print, to avoid repeating "Preflop"
        let mut lobby = state.lobby.write().await;
        let already = lobby.last_printed_log_len;
        let total = gs.action_log.len();
        if total > already {
            for e in gs.action_log.iter().skip(already) {
                let line = mcg_server::pretty::format_event_human(
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
        send_ws(socket, &ServerMsg::State(gs)).await;
    }
}

async fn drive_bots_with_delays(
    socket: &mut WebSocket,
    state: &AppState,
    you_id: usize,
    min_ms: u64,
    max_ms: u64,
) {
    loop {
        // Perform a single bot action if it's their turn
        let did_act = {
            let mut lobby = state.lobby.write().await;
            if let Some(game) = &mut lobby.game {
                if game.stage != mcg_shared::Stage::Showdown && game.to_act != you_id {
                    let actor = game.to_act;
                    // Choose a simple bot action using the same logic as random_bot_action
                    let need = game.current_bet.saturating_sub(game.round_bets[actor]);
                    let action = if need == 0 {
                        mcg_shared::PlayerAction::Bet(game.bb)
                    } else {
                        mcg_shared::PlayerAction::CheckCall
                    };
                    let _ = game.apply_player_action(actor, action);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        // Send updated state to client
        send_state_to(socket, state, you_id).await;

        if !did_act {
            break;
        }
        // Sleep a pseudo-random-ish delay between actions without holding non-Send state
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u64;
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
}

async fn process_client_msg(
    name: &str,
    state: &AppState,
    socket: &mut WebSocket,
    cm: ClientMsg,
    you_id: usize,
) {
    match cm {
        ClientMsg::Action(a) => {
            println!("[WS] Action from {}: {:?}", name, a);
            let mut err: Option<String> = None;
            {
                let mut lobby = state.lobby.write().await;
                if let Some(game) = &mut lobby.game {
                    if let Err(e) = game.apply_player_action(0, a.clone()) {
                        err = Some(e.to_string());
                    }
                }
            }
            if let Some(e) = err {
                send_ws(socket, &ServerMsg::Error(e)).await;
            }
            // Always send latest state then drive bots stepwise with delays
            send_state_to(socket, state, you_id).await;
            drive_bots_with_delays(socket, state, you_id, 500, 1500).await;
        }
        ClientMsg::RequestState => {
            println!("[WS] State requested by {}", name);
            // Send latest then drive bots if it's their turn
            send_state_to(socket, state, you_id).await;
            drive_bots_with_delays(socket, state, you_id, 500, 1500).await;
        }
        ClientMsg::NextHand => {
            println!("[WS] NextHand requested by {}", name);
            {
                let mut lobby = state.lobby.write().await;
                if let Some(game) = &mut lobby.game {
                    let n = game.players.len();
                    if n > 0 {
                        game.dealer_idx = (game.dealer_idx + 1) % n;
                    }
                    game.start_new_hand();
                    // After starting a new hand, print a table header banner once
                    let sb = game.sb;
                    let bb = game.bb;
                    let gs = game.public_for(you_id);
                    lobby.last_printed_log_len = gs.action_log.len();
                    let header = format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
                    println!("{}", header);
                }
            }
            send_state_to(socket, state, you_id).await;
            drive_bots_with_delays(socket, state, you_id, 500, 1500).await;
        }
        ClientMsg::ResetGame { bots } => {
            println!("[WS] ResetGame requested by {}: bots={} ", name, bots);
            {
                let mut lobby = state.lobby.write().await;
                lobby.game = Some(Game::new(name.to_string(), bots));
                if let Some(game) = &mut lobby.game {
                    let sb = game.sb;
                    let bb = game.bb;
                    let gs = game.public_for(you_id);
                    lobby.last_printed_log_len = gs.action_log.len();
                    let header = format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
                    println!("{}", header);
                }
            }
            send_state_to(socket, state, you_id).await;
            drive_bots_with_delays(socket, state, you_id, 500, 1500).await;
        }
        ClientMsg::Join { .. } => {}
    }
}

async fn current_state_public(state: &AppState, you_id: usize) -> Option<GameStatePublic> {
    let lobby = state.lobby.read().await;
    lobby.game.as_ref().map(|g| g.public_for(you_id))
}

/// Serve index.html file
async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("index.html").await {
        Ok(content) => (StatusCode::OK, [("content-type", "text/html")], content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

/// SPA fallback handler - serves index.html for client-side routing
async fn spa_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path();

    // Don't serve index.html for API routes or asset requests
    if path.starts_with("/api")
        || path.starts_with("/pkg")
        || path.starts_with("/media")
        || path.starts_with("/ws")
        || path.starts_with("/health")
    {
        return StatusCode::NOT_FOUND.into_response();
    }

    // For all other routes, serve index.html to enable client-side routing
    match tokio::fs::read_to_string("index.html").await {
        Ok(content) => (StatusCode::OK, [("content-type", "text/html")], content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}
