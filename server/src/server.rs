//! Server networking and WebSocket handling.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures::StreamExt;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::game::Game;
use mcg_shared::{ClientMsg, GameStatePublic, ServerMsg};

#[derive(Clone, Default)]
pub struct AppState {
    pub lobby: Arc<RwLock<Lobby>>,
    pub bot_count: usize,
}

#[derive(Clone, Default)]
pub(crate) struct Lobby {
    game: Option<Game>,
}

pub fn build_router(state: AppState) -> Router {
    // Serve static files from the project root. Assumes process CWD is repo root.
    let serve_dir = ServeDir::new("pkg").append_index_html_on_directories(true);
    let serve_media = ServeDir::new("media").append_index_html_on_directories(true);
    let serve_root = ServeDir::new(".").append_index_html_on_directories(true);

    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true })) }),
        )
        .route("/ws", get(ws_handler))
        .nest_service("/pkg", serve_dir)
        .nest_service("/media", serve_media)
        // Serve index.html at root
        .nest_service("/", serve_root)
        .with_state(state)
}

pub async fn run_server(addr: SocketAddr, state: AppState) {
    let app = build_router(state.clone());
    println!("[START] Server running at http://{}", addr);
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
                let _ = socket
                    .send(Message::Text(
                        serde_json::to_string(&ServerMsg::Error("Expected Join".into())).unwrap(),
                    ))
                    .await;
                return;
            }
        },
        _ => return,
    };
    println!("[CONNECT] New player: {}", name);

    {
        let mut lobby = state.lobby.write().await;
        if lobby.game.is_none() {
            lobby.game = Some(Game::new(name.clone(), state.bot_count));
            println!(
                "[GAME] Created new game for {} with {} bot(s)",
                name, state.bot_count
            );
            // Let bots act if it's their turn
            if let Some(game) = &mut lobby.game {
                if game.players.len() > 1 && game.to_act != 0 {
                    game.play_out_bots();
                }
            }
        }
    }

    let you_id = 0usize;
    let _ = socket
        .send(Message::Text(
            serde_json::to_string(&ServerMsg::Welcome { you: you_id }).unwrap(),
        ))
        .await;

    if let Some(gs) = current_state_public(&state, you_id).await {
        let _ = socket
            .send(Message::Text(
                serde_json::to_string(&ServerMsg::State(gs)).unwrap(),
            ))
            .await;
    }

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(txt)) => {
                if let Ok(cm) = serde_json::from_str::<ClientMsg>(&txt) {
                    match cm {
                        ClientMsg::Action(a) => {
                            println!("[WS] Action from {}: {:?}", name, a);
                            {
                                let mut lobby = state.lobby.write().await;
                                if let Some(game) = &mut lobby.game {
                                    game.apply_player_action(0, a.clone());
                                    game.play_out_bots();
                                }
                            }
                            if let Some(gs) = current_state_public(&state, you_id).await {
                                let _ = socket
                                    .send(Message::Text(
                                        serde_json::to_string(&ServerMsg::State(gs)).unwrap(),
                                    ))
                                    .await;
                            }
                        }
                        ClientMsg::RequestState => {
                            println!("[WS] State requested by {}", name);
                            {
                                let mut lobby = state.lobby.write().await;
                                if let Some(game) = &mut lobby.game {
                                    game.play_out_bots();
                                }
                            }
                            if let Some(gs) = current_state_public(&state, you_id).await {
                                let _ = socket
                                    .send(Message::Text(
                                        serde_json::to_string(&ServerMsg::State(gs)).unwrap(),
                                    ))
                                    .await;
                            }
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
                                    if game.to_act != 0 {
                                        game.play_out_bots();
                                    }
                                }
                            }
                            if let Some(gs) = current_state_public(&state, you_id).await {
                                let _ = socket
                                    .send(Message::Text(
                                        serde_json::to_string(&ServerMsg::State(gs)).unwrap(),
                                    ))
                                    .await;
                            }
                        }
                        ClientMsg::Join { .. } => {}
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
    println!("[DISCONNECT] {} disconnected", name);
}

async fn current_state_public(state: &AppState, you_id: usize) -> Option<GameStatePublic> {
    let lobby = state.lobby.read().await;
    lobby.game.as_ref().map(|g| g.public_for(you_id))
}
