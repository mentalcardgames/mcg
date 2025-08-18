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
use mcg_shared::{ClientMsg, GameStatePublic, ServerMsg};
use owo_colors::OwoColorize;
use std::io::IsTerminal;

use anyhow::{Result, Context};

use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub lobby: Arc<RwLock<Lobby>>,
    pub bot_count: usize,
    pub broadcaster: broadcast::Sender<ServerMsg>,
}

#[derive(Clone, Default)]
pub(crate) struct Lobby {
    pub(crate) game: Option<Game>,
    pub(crate) last_printed_log_len: usize,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(16);
        AppState {
            lobby: Arc::new(RwLock::new(Lobby::default())),
            bot_count: 0,
            broadcaster: tx,
        }
    }
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

pub async fn run_server(addr: SocketAddr, state: AppState) -> Result<()> {
    let app = build_router(state.clone());

    // Spawn the iroh listener so it runs concurrently with the Axum HTTP/WebSocket server.
    // This is always enabled.
    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::iroh_transport::spawn_iroh_listener(state_clone).await {
                eprintln!("Iroh listener failed: {}", e);
            }
        });
    }

    let display_addr = if addr.ip().to_string() == "127.0.0.1" {
        format!("localhost:{}", addr.port())
    } else {
        addr.to_string()
    };

    println!("🌐 MCG Server running at http://{}", display_addr);
    println!("📱 Open your browser and navigate to the above URL");
    println!();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind to {}", display_addr))?;
    // axum::serve returns a future that runs the server; propagate any error if it returns one.
    let _ = axum::serve(listener, app).await;
    Ok(())
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

    if let Err(e) = ensure_game_started(&state, &name).await {
        let _ = send_ws(&mut socket, &ServerMsg::Error(format!("Failed to start game: {}", e))).await;
        return;
    }

    let you_id = 0usize;
    send_ws(&mut socket, &ServerMsg::Welcome { you: you_id }).await;
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
                        if let Ok(cm) = serde_json::from_str::<ClientMsg>(&txt) {
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

pub async fn ensure_game_started(state: &AppState, name: &str) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if lobby.game.is_none() {
        let g = Game::new(name.to_string(), state.bot_count)
            .with_context(|| format!("creating new game for {}", name))?;
        lobby.game = Some(g);
        println!(
            "[GAME] Created new game for {} with {} bot(s)",
            name, state.bot_count
        );
    }
    Ok(())
}

async fn send_ws(socket: &mut WebSocket, msg: &ServerMsg) {
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

/// Broadcast the current state (and print new events to server console) to all subscribers.
///
/// This centralizes server-side broadcasting so websocket and iroh transports both
/// share the same behavior.
pub async fn broadcast_state(state: &AppState, you_id: usize) {
    if let Some(gs) = current_state_public(state, you_id).await {
        // Print any newly added events to server console and update bookkeeping.
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

        // Broadcast the new state to all subscribers.
        let _ = state.broadcaster.send(ServerMsg::State(gs));
    }
}

/// Drive bots similarly to the websocket handler, but mutate shared state and
/// broadcast resulting states. Exposed so iroh transport can reuse the same behaviour.
pub async fn drive_bots_with_delays(state: &AppState, you_id: usize, min_ms: u64, max_ms: u64) {
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

        // Broadcast updated state to all subscribers
        broadcast_state(state, you_id).await;

        if !did_act {
            break;
        }
        // Sleep a pseudo-random-ish delay between actions without holding non-Send state
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
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
            if let Some(e) = apply_action_to_game(state, 0, a.clone()).await {
                let _ = send_ws(socket, &ServerMsg::Error(e)).await;
            }
            // Broadcast latest state then drive bots stepwise with delays
            broadcast_state(state, you_id).await;
            drive_bots_with_delays(state, you_id, 500, 1500).await;
        }
        ClientMsg::RequestState => {
            println!("[WS] State requested by {}", name);
            // Broadcast latest then drive bots if it's their turn
            broadcast_state(state, you_id).await;
            drive_bots_with_delays(state, you_id, 500, 1500).await;
        }
        ClientMsg::NextHand => {
            println!("[WS] NextHand requested by {}", name);
            if let Err(e) = start_new_hand_and_print(state, you_id).await {
                let _ = send_ws(socket, &ServerMsg::Error(format!("Failed to start new hand: {}", e))).await;
            }
            // Broadcast the changed state and drive bots
            broadcast_state(state, you_id).await;
            drive_bots_with_delays(state, you_id, 500, 1500).await;
        }
        ClientMsg::ResetGame { bots } => {
            println!("[WS] ResetGame requested by {}: bots={} ", name, bots);
            if let Err(e) = reset_game_with_bots(state, &name, bots, you_id).await {
                let _ = send_ws(socket, &ServerMsg::Error(format!("Failed to reset game: {}", e))).await;
            } else {
                broadcast_state(state, you_id).await;
            }
            drive_bots_with_delays(state, you_id, 500, 1500).await;
        }
        ClientMsg::Join { .. } => {}
    }
}

pub async fn current_state_public(state: &AppState, you_id: usize) -> Option<GameStatePublic> {
    let lobby = state.lobby.read().await;
    lobby.game.as_ref().map(|g| g.public_for(you_id))
}

/// Apply an action to the game's state. Returns Some(error_string) if the
/// underlying Game::apply_player_action returned an error, otherwise None.
pub async fn apply_action_to_game(
    state: &AppState,
    actor: usize,
    action: mcg_shared::PlayerAction,
) -> Option<String> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        if let Err(e) = game.apply_player_action(actor, action) {
            return Some(e.to_string());
        }
    }
    None
}

/// Advance to the next hand (increment dealer, start a new hand) and print a table header.
/// Mirrors the logic previously duplicated in websocket / iroh handlers.
pub async fn start_new_hand_and_print(state: &AppState, you_id: usize) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        let n = game.players.len();
        if n > 0 {
            game.dealer_idx = (game.dealer_idx + 1) % n;
        }
        if let Err(e) = game.start_new_hand() {
            return Err(e);
        }
        let sb = game.sb;
        let bb = game.bb;
        let gs = game.public_for(you_id);
        lobby.last_printed_log_len = gs.action_log.len();
        let header = mcg_server::pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
        println!("{}", header);
    }
    Ok(())
}

/// Reset the game with a new Game created for `name` with `bots` bots, and print header.
/// Returns Err if Game::new fails.
pub async fn reset_game_with_bots(
    state: &AppState,
    name: &str,
    bots: usize,
    you_id: usize,
) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    match Game::new(name.to_string(), bots) {
        Ok(g) => {
            lobby.game = Some(g);
            if let Some(game) = &mut lobby.game {
                let sb = game.sb;
                let bb = game.bb;
                let gs = game.public_for(you_id);
                lobby.last_printed_log_len = gs.action_log.len();
                let header = mcg_server::pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
                println!("{}", header);
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
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
