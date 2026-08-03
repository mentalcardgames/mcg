// HTTP handlers for the MCG server API.
//
// Provides a single transport-agnostic endpoint that mirrors websocket actions.
// Handlers reuse the centralized backend handler `dispatch_client_message` to ensure
// consistent behavior across transports (iroh, websocket, HTTP).

use axum::{extract::State, Json};

use crate::server::{peer_connections::PeerConnectionService, AppState};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};

/// Unified handler for all Frontend2BackendMsg variants. Returns the serialized Backend2FrontendMsg response.
pub(super) async fn message_handler(
    State(state): State<AppState>,
    State(peer_connections): State<PeerConnectionService>,
    Json(cm): Json<Frontend2BackendMsg>,
) -> Json<Backend2FrontendMsg> {
    let response = match cm {
        Frontend2BackendMsg::QrValue(ticket) => match peer_connections.connect(ticket).await {
            Ok(_) => Backend2FrontendMsg::Pong,
            Err(error) => Backend2FrontendMsg::Error(format!("Failed to connect to peer: {error}")),
        },
        message => crate::server::dispatch_client_message(&state, message).await,
    };
    Json(response)
}
