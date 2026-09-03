use std::sync::Arc;

use axum::{
    extract::{ws::WebSocketUpgrade, FromRef, State},
    http::{header::SEC_WEBSOCKET_PROTOCOL, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use tower_http::services::ServeDir;

use super::{NetworkHandle, PeerConnectionService, WEBSOCKET_FRONTEND_PROTOCOL};
use crate::controller::ControllerHandle;

/// Shared state container for the Axum router and handlers.
#[derive(Clone)]
pub struct RouterState {
    pub controller: ControllerHandle,
    pub network: NetworkHandle,
    pub peer_connections: PeerConnectionService,
    pub _task_guard: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl RouterState {
    pub fn new(
        controller: ControllerHandle,
        network: NetworkHandle,
        peer_connections: PeerConnectionService,
    ) -> Self {
        Self {
            controller,
            network,
            peer_connections,
            _task_guard: None,
        }
    }

    pub fn with_task_guard(mut self, guard: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        self._task_guard = Some(guard);
        self
    }
}

impl FromRef<RouterState> for ControllerHandle {
    fn from_ref(state: &RouterState) -> Self {
        state.controller.clone()
    }
}

impl FromRef<RouterState> for NetworkHandle {
    fn from_ref(state: &RouterState) -> Self {
        state.network.clone()
    }
}

impl FromRef<RouterState> for PeerConnectionService {
    fn from_ref(state: &RouterState) -> Self {
        state.peer_connections.clone()
    }
}

#[derive(Clone, Copy)]
enum WebSocketRole {
    LegacyFrontend,
    Frontend,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(network): State<NetworkHandle>,
) -> Response {
    let role = match websocket_role(&headers) {
        Ok(role) => role,
        Err(message) => return (StatusCode::BAD_REQUEST, message).into_response(),
    };

    match role {
        WebSocketRole::LegacyFrontend => ws
            .on_upgrade(move |socket| register_frontend(network, socket))
            .into_response(),
        WebSocketRole::Frontend => ws
            .protocols([WEBSOCKET_FRONTEND_PROTOCOL])
            .on_upgrade(move |socket| register_frontend(network, socket))
            .into_response(),
    }
}

async fn register_frontend(network: NetworkHandle, socket: axum::extract::ws::WebSocket) {
    if let Err(error) = network.register_frontend_websocket(socket).await {
        tracing::error!(%error, "failed to register frontend WebSocket connection");
    }
}

fn websocket_role(headers: &HeaderMap) -> Result<WebSocketRole, &'static str> {
    let Some(protocols) = headers.get(SEC_WEBSOCKET_PROTOCOL) else {
        return Ok(WebSocketRole::LegacyFrontend);
    };
    let protocols = protocols
        .to_str()
        .map_err(|_| "invalid WebSocket subprotocol header")?;

    for protocol in protocols.split(',').map(str::trim) {
        if protocol == WEBSOCKET_FRONTEND_PROTOCOL {
            return Ok(WebSocketRole::Frontend);
        }
    }

    Err("unsupported WebSocket subprotocol")
}

/// Unified handler for all Frontend2BackendMsg variants. Returns the serialized Backend2FrontendMsg response.
pub async fn http_handler(
    State(controller): State<ControllerHandle>,
    Json(cm): Json<Frontend2BackendMsg>,
) -> Json<Backend2FrontendMsg> {
    let response = controller
        .send_http_request(cm)
        .await
        .unwrap_or_else(|error| Backend2FrontendMsg::Error(format!("Controller error: {error}")));
    Json(response)
}

/// Health check endpoint responding with `{ "ok": true }`.
pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

/// Serve index.html file
pub async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("index.html").await {
        Ok(content) => (StatusCode::OK, [("content-type", "text/html")], content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

/// Single Page Application (SPA) fallback handler - serves index.html for client-side routing
pub async fn spa_handler(uri: Uri) -> impl IntoResponse {
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
    serve_index().await.into_response()
}

/// Constructs the Axum application router with all HTTP, WebSocket, static assets, and SPA fallback routes.
pub fn build_router(state: RouterState) -> Router {
    // Serve static files from the project root. Assumes process CWD is repo root.
    let serve_dir = ServeDir::new("pkg").append_index_html_on_directories(true);
    let serve_media = ServeDir::new("media").append_index_html_on_directories(true);

    Router::new()
        .route("/health", get(health_handler))
        // WebSocket endpoint (WASM GUI remains websocket-only)
        .route("/ws", get(ws_handler))
        // HTTP API endpoint using unified Frontend2BackendMsg/Backend2FrontendMsg payloads
        .route("/api/message", post(http_handler))
        .nest_service("/pkg", serve_dir)
        .nest_service("/media", serve_media)
        // Serve index.html for the root route
        .route("/", get(serve_index))
        // Fallback handler for SPA routing - serve index.html for all other routes
        .fallback(spa_handler)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use tokio_tungstenite::tungstenite::http::HeaderValue;

    use super::*;

    #[test]
    fn subprotocol_selection_keeps_legacy_frontend_compatibility() {
        assert!(matches!(
            websocket_role(&HeaderMap::new()),
            Ok(WebSocketRole::LegacyFrontend)
        ));

        let mut frontend = HeaderMap::new();
        frontend.insert(
            SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static(WEBSOCKET_FRONTEND_PROTOCOL),
        );
        assert!(matches!(
            websocket_role(&frontend),
            Ok(WebSocketRole::Frontend)
        ));

        let mut peer = HeaderMap::new();
        peer.insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static("mcg.peer"));
        assert_eq!(
            websocket_role(&peer).err(),
            Some("unsupported WebSocket subprotocol")
        );

        let mut unsupported = HeaderMap::new();
        unsupported.insert(
            SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static("mcg.unknown"),
        );
        assert_eq!(
            websocket_role(&unsupported).err(),
            Some("unsupported WebSocket subprotocol")
        );
    }

    #[tokio::test]
    async fn health_handler_returns_ok() {
        let Json(body) = health_handler().await;
        assert_eq!(body.get("ok").and_then(|v| v.as_bool()), Some(true));
    }

    #[tokio::test]
    async fn spa_handler_rejects_api_and_asset_prefixes() {
        assert_eq!(
            spa_handler(Uri::from_static("/api/unknown"))
                .await
                .into_response()
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            spa_handler(Uri::from_static("/pkg/missing.js"))
                .await
                .into_response()
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            spa_handler(Uri::from_static("/media/card.png"))
                .await
                .into_response()
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            spa_handler(Uri::from_static("/ws"))
                .await
                .into_response()
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            spa_handler(Uri::from_static("/health"))
                .await
                .into_response()
                .status(),
            StatusCode::NOT_FOUND
        );
    }
}
