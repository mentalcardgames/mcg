// HTTP handlers for the MCG server API.
//
// Provides simple POST/GET endpoints that mirror existing websocket actions so
// the server logic can remain transport-agnostic. Handlers reuse functions from
// crate::backend::state to operate on the shared AppState and return the same
// ServerMsg/ClientMsg shapes (mcg_shared) as other transports.

use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use anyhow::Result;

use crate::backend::AppState;
use mcg_shared::{ClientMsg, PlayerAction, ServerMsg};

/// Accept a Join-like ClientMsg (or JSON equivalent) and ensure a game exists.
///
/// Example payload:
///   { "type": "Join", "data": { "name": "CLI" } }
pub async fn join_handler(State(state): State<AppState>, Json(cm): Json<ClientMsg>) -> impl IntoResponse {
    match cm {
        ClientMsg::Join { name } => {
            if let Err(e) = crate::backend::ensure_game_started(&state, &name).await {
                let err = ServerMsg::Error(format!("Failed to start game: {}", e));
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
            }
            // Mirror websocket behavior: welcome the client (you_id = 0)
            let welcome = ServerMsg::Welcome { you: 0 };
            // Also send initial state to broadcaster (backend printing/bookkeeping is done elsewhere)
            crate::backend::broadcast_state(&state, 0).await;
            (StatusCode::OK, Json(welcome)).into_response()
        }
        _ => {
            let err = ServerMsg::Error("Expected Join message".into());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// Apply a player action.
///
/// Body: { "type": "Action", "data": ... } or simply the action shape if you prefer.
/// Returns a ServerMsg::State on success or ServerMsg::Error on failure.
pub async fn action_handler(State(state): State<AppState>, Json(cm): Json<ClientMsg>) -> impl IntoResponse {
    match cm {
        ClientMsg::Action(action) => {
            // actor id is 0 for single-player CLI usage
            if let Some(e) = crate::backend::apply_action_to_game(&state, 0, action.clone()).await {
                let err = ServerMsg::Error(e);
                return (StatusCode::BAD_REQUEST, Json(err)).into_response();
            }
            // Send state to requester immediately (we return it) and broadcast to subscribers.
            if let Some(gs) = crate::backend::current_state_public(&state, 0).await {
                // Broadcast to other transports
                crate::backend::broadcast_state(&state, 0).await;
                // Drive bots in background (don't block the HTTP response)
                let state_clone = state.clone();
                tokio::spawn(async move {
                    crate::backend::drive_bots_with_delays(&state_clone, 0, 500, 1500).await;
                });
                return (StatusCode::OK, Json(ServerMsg::State(gs))).into_response();
            } else {
                let err = ServerMsg::Error("No game running".into());
                return (StatusCode::NOT_FOUND, Json(err)).into_response();
            }
        }
        _ => {
            let err = ServerMsg::Error("Expected Action message".into());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// Return the current public state for the (single) CLI player (you_id = 0).
pub async fn state_handler(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(gs) = crate::backend::current_state_public(&state, 0).await {
        (StatusCode::OK, Json(ServerMsg::State(gs))).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(ServerMsg::Error("No game running".into()))).into_response()
    }
}

/// Advance to the next hand (server-side). Returns state after change.
pub async fn next_hand_handler(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = crate::backend::start_new_hand_and_print(&state, 0).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ServerMsg::Error(format!("Failed to start new hand: {}", e)))).into_response();
    }
    if let Some(gs) = crate::backend::current_state_public(&state, 0).await {
        crate::backend::broadcast_state(&state, 0).await;
        // drive bots asynchronously
        let state_clone = state.clone();
        tokio::spawn(async move {
            crate::backend::drive_bots_with_delays(&state_clone, 0, 500, 1500).await;
        });
        (StatusCode::OK, Json(ServerMsg::State(gs))).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(ServerMsg::Error("No game running".into()))).into_response()
    }
}

/// Reset the game with N bots. Expects payload:
///   { "type": "ResetGame", "data": { "bots": 2 } }
pub async fn reset_handler(State(state): State<AppState>, Json(cm): Json<ClientMsg>) -> impl IntoResponse {
    match cm {
        ClientMsg::ResetGame { bots } => {
            // Use a default name for HTTP-initiated resets
            let name = "HTTP";
            if let Err(e) = crate::backend::reset_game_with_bots(&state, name, bots, 0).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ServerMsg::Error(format!("Failed to reset game: {}", e)))).into_response();
            }
            if let Some(gs) = crate::backend::current_state_public(&state, 0).await {
                crate::backend::broadcast_state(&state, 0).await;
                (StatusCode::OK, Json(ServerMsg::State(gs))).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json(ServerMsg::Error("No game running".into()))).into_response()
            }
        }
        _ => {
            let err = ServerMsg::Error("Expected ResetGame message".into());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}
