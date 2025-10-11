// HTTP handlers for the MCG server API.
//
// Provides a single transport-agnostic endpoint that mirrors websocket actions.
// Handlers reuse the centralized backend handler `handle_client_msg` to ensure
// consistent behavior across transports (iroh, websocket, HTTP).

use axum::{extract::State, Json};

use crate::server::AppState;
use mcg_shared::{ClientMsg, ServerMsg};

/// Unified handler for all ClientMsg variants. Returns the serialized ServerMsg response.
pub async fn message_handler(State(state): State<AppState>, Json(cm): Json<ClientMsg>) -> Json<ServerMsg> {
    Json(crate::server::handle_client_msg(&state, cm).await)
}
