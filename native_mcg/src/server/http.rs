// HTTP handlers for the MCG server API.
//
// Provides simple POST/GET endpoints that mirror existing websocket actions so
// the server logic can remain transport-agnostic. Handlers reuse the centralized
// backend handler `handle_client_msg` to ensure consistent behavior across
// transports (iroh, websocket, HTTP).

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::server::AppState;
use mcg_shared::{ClientMsg, ServerMsg};

/// Accept a NewGame ClientMsg and create a new game.
///
/// Example payload:
///   { "type": "NewGame", "data": { "players": [...]} }
pub async fn newgame_handler(
    State(state): State<AppState>,
    Json(cm): Json<ClientMsg>,
) -> impl IntoResponse {
    // Delegate to unified handler
    let resp = crate::server::handle_client_msg(&state, cm).await;
    match resp {
        ServerMsg::State(gs) => (StatusCode::OK, Json(ServerMsg::State(gs))).into_response(),
        ServerMsg::Error(e) => {
            // Creating a new game failing is server-side error historically.
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ServerMsg::Error(e))).into_response()
        }
        other => (StatusCode::OK, Json(other)).into_response(),
    }
}

/// Apply a player action.
///
/// Body: { "type": "Action", "data": ... }
/// Returns a ServerMsg::State on success or ServerMsg::Error on failure.
pub async fn action_handler(
    State(state): State<AppState>,
    Json(cm): Json<ClientMsg>,
) -> impl IntoResponse {
    let resp = crate::server::handle_client_msg(&state, cm).await;
    match resp {
        ServerMsg::State(gs) => {
            // Broadcast/drive side-effects are handled by the unified handler.
            (StatusCode::OK, Json(ServerMsg::State(gs))).into_response()
        }
        ServerMsg::Error(e) => (StatusCode::BAD_REQUEST, Json(ServerMsg::Error(e))).into_response(),
        other => (StatusCode::OK, Json(other)).into_response(),
    }
}

pub async fn state_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Reuse unified handler by requesting state
    let resp = crate::server::handle_client_msg(&state, ClientMsg::RequestState).await;
    match resp {
        ServerMsg::State(gs) => (StatusCode::OK, Json(ServerMsg::State(gs))).into_response(),
        ServerMsg::Error(e) => {
            // No active game historically returned NOT_FOUND; map error to NOT_FOUND when appropriate.
            (StatusCode::NOT_FOUND, Json(ServerMsg::Error(e))).into_response()
        }
        other => (StatusCode::OK, Json(other)).into_response(),
    }
}

/// Advance to the next hand (server-side). Returns state after change.
pub async fn next_hand_handler(State(state): State<AppState>) -> impl IntoResponse {
    let resp = crate::backend::handle_client_msg(&state, ClientMsg::NextHand).await;
    match resp {
        ServerMsg::State(gs) => (StatusCode::OK, Json(ServerMsg::State(gs))).into_response(),
        ServerMsg::Error(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ServerMsg::Error(e))).into_response()
        }
        other => (StatusCode::OK, Json(other)).into_response(),
    }
}
