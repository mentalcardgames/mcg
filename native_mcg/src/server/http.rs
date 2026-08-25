// HTTP handlers for the MCG server API.
//
// Provides a single transport-agnostic endpoint that mirrors websocket actions.
// Handlers route client messages through the Controller via oneshot request-response.

use axum::{Json, extract::State};

use crate::controller::ControllerHandle;
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};

/// Unified handler for all Frontend2BackendMsg variants. Returns the serialized Backend2FrontendMsg response.
pub(super) async fn message_handler(
    State(controller): State<ControllerHandle>,
    Json(cm): Json<Frontend2BackendMsg>,
) -> Json<Backend2FrontendMsg> {
    let response = controller
        .send_http_request(cm)
        .await
        .unwrap_or_else(|error| Backend2FrontendMsg::Error(format!("Controller error: {error}")));
    Json(response)
}
