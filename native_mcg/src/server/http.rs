// HTTP handlers for the MCG server API.
//
// Provides a single transport-agnostic endpoint that mirrors websocket actions.
// Handlers reuse the centralized backend handler `dispatch_client_message` to ensure
// consistent behavior across transports (iroh, websocket, HTTP).

use axum::{extract::State, Json};

use crate::network::NetworkHandle;
use crate::server::{network_adapter::connect_and_introduce, AppState};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};

/// Unified handler for all Frontend2BackendMsg variants. Returns the serialized Backend2FrontendMsg response.
pub async fn message_handler(
    State(state): State<AppState>,
    State(network): State<NetworkHandle>,
    Json(cm): Json<Frontend2BackendMsg>,
) -> Json<Backend2FrontendMsg> {
    let response = match cm {
        Frontend2BackendMsg::QrValue(ticket) => {
            match connect_and_introduce(&state, &network, ticket).await {
                Ok(_) => Backend2FrontendMsg::Pong,
                Err(error) => {
                    Backend2FrontendMsg::Error(format!("Failed to connect to peer: {error}"))
                }
            }
        }
        message => crate::server::dispatch_client_message(&state, message).await,
    };
    Json(response)
}
