// HTTP handlers for the MCG server API.
//
// Provides a single transport-agnostic endpoint that mirrors websocket actions.
// Handlers reuse the centralized backend handler `dispatch_client_message` to ensure
// consistent behavior across transports (iroh, websocket, HTTP).

use axum::{extract::State, Json};

use crate::server::AppState;
use mcg_shared::{Frontend2BackendMsg, Backend2FrontendMsg};

/// Unified handler for all Frontend2BackendMsg variants. Returns the serialized Backend2FrontendMsg response.
pub async fn message_handler(
    State(state): State<AppState>,
    Json(cm): Json<Frontend2BackendMsg>,
) -> Json<Backend2FrontendMsg> {
    Json(crate::server::dispatch_client_message(&state, cm).await)
}
