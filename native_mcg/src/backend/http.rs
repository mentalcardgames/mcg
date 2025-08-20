// HTTP handlers for the MCG server API.
//
// Provides simple POST/GET endpoints that mirror existing websocket actions so
// the server logic can remain transport-agnostic. Handlers reuse functions from
// crate::backend::state to operate on the shared AppState and return the same
// ServerMsg/ClientMsg shapes (mcg_shared) as other transports.

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::backend::AppState;
use mcg_shared::{ClientMsg, ServerMsg};

/// Accept a NewGame ClientMsg and create a new game.
///
/// Example payload:
///   { "type": "NewGame", "data": { "players": [...], "primary_player_id": 0 } }
pub async fn newgame_handler(
    State(state): State<AppState>,
    Json(cm): Json<ClientMsg>,
) -> impl IntoResponse {
    match cm {
        ClientMsg::NewGame { players } => {
            if let Err(e) = crate::backend::create_new_game(&state, players).await {
                let err = ServerMsg::Error(format!("Failed to create game: {}", e));
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
            }
            // Mirror websocket behavior: welcome the client with default player ID
            let welcome = ServerMsg::Welcome { you: 0 };
            // Also send initial state to broadcaster (backend printing/bookkeeping is done elsewhere)
            crate::backend::broadcast_state(&state, 0).await;
            (StatusCode::OK, Json(welcome)).into_response()
        }
        _ => {
            let err = ServerMsg::Error("Expected NewGame message".into());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// Apply a player action.
///
/// Body: { "type": "Action", "data": ... } or simply the action shape if you prefer.
/// Returns a ServerMsg::State on success or ServerMsg::Error on failure.
pub async fn action_handler(
    State(state): State<AppState>,
    Json(cm): Json<ClientMsg>,
) -> impl IntoResponse {
    match cm {
        ClientMsg::Action { player_id, action } => {
            // Use the provided player_id as the actor (validate centrally)
            match crate::backend::validate_and_apply_action(&state, player_id, action.clone()).await
            {
                Ok(()) => {
                    // Send state to requester immediately (we return it) and broadcast to subscribers.
                    if let Some(gs) = crate::backend::current_state_public(&state, player_id).await
                    {
                        // Broadcast current state immediately so other transports see the update.
                        crate::backend::broadcast_state(&state, player_id).await;
                        // Drive bots in background (don't block the HTTP response)
                        let state_clone = state.clone();
                        tokio::spawn(async move {
                            crate::backend::broadcast_and_drive(&state_clone, player_id, 500, 1500)
                                .await;
                        });
                        (StatusCode::OK, Json(ServerMsg::State(gs))).into_response()
                    } else {
                        let err = ServerMsg::Error("No game running".into());
                        (StatusCode::NOT_FOUND, Json(err)).into_response()
                    }
                }
                Err(e) => {
                    let err = ServerMsg::Error(e);
                    (StatusCode::BAD_REQUEST, Json(err)).into_response()
                }
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
        (
            StatusCode::NOT_FOUND,
            Json(ServerMsg::Error("No game running".into())),
        )
            .into_response()
    }
}

/// Advance to the next hand (server-side). Returns state after change.
pub async fn next_hand_handler(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = crate::backend::start_new_hand_and_print(&state, 0).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ServerMsg::Error(format!("Failed to start new hand: {}", e))),
        )
            .into_response();
    }
    if let Some(gs) = crate::backend::current_state_public(&state, 0).await {
        crate::backend::broadcast_state(&state, 0).await;
        // drive bots asynchronously
        let state_clone = state.clone();
        tokio::spawn(async move {
            crate::backend::broadcast_and_drive(&state_clone, 0, 500, 1500).await;
        });
        (StatusCode::OK, Json(ServerMsg::State(gs))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ServerMsg::Error("No game running".into())),
        )
            .into_response()
    }
}
