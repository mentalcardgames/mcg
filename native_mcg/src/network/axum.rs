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

use super::{
    NetworkHandle, PeerConnectionService, WEBSOCKET_FRONTEND_PROTOCOL, WEBSOCKET_PEER_PROTOCOL,
};
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
    Peer,
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
        WebSocketRole::Peer => ws
            .protocols([WEBSOCKET_PEER_PROTOCOL])
            .on_upgrade(move |socket| async move {
                if let Err(error) = network.register_pending_peer_websocket(socket).await {
                    tracing::error!(%error, "failed to register pending peer WebSocket connection");
                }
            })
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
        match protocol {
            WEBSOCKET_FRONTEND_PROTOCOL => return Ok(WebSocketRole::Frontend),
            WEBSOCKET_PEER_PROTOCOL => return Ok(WebSocketRole::Peer),
            _ => {}
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
    use std::time::Duration;

    use anyhow::Result;
    use axum::{routing::get, Router};
    use futures::{SinkExt, StreamExt};
    use mcg_shared::{Peer2PeerMsg, WebSocketPeerHandshake};
    use tokio::sync::mpsc;
    use tokio_tungstenite::tungstenite::{
        client::IntoClientRequest,
        http::{HeaderValue, Request},
        Message,
    };

    use super::*;
    use crate::network::{
        ConnectionCloseReason, NetworkCommand, NetworkEvent, NetworkSupervisor,
        PeerConnectionDirection, PeerId, TransportKind,
    };

    fn websocket_request(url: String, protocol: &'static str) -> Result<Request<()>> {
        let mut request = url.into_client_request()?;
        request
            .headers_mut()
            .insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static(protocol));
        Ok(request)
    }

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
        peer.insert(
            SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_static(WEBSOCKET_PEER_PROTOCOL),
        );
        assert!(matches!(websocket_role(&peer), Ok(WebSocketRole::Peer)));

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
    async fn peer_subprotocol_identifies_then_exchanges_peer_messages() -> Result<()> {
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(network.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server_task = tokio::spawn(async move { axum::serve(listener, app).await });

        let request = websocket_request(format!("ws://{address}/ws"), WEBSOCKET_PEER_PROTOCOL)?;
        let (mut client, response) = tokio_tungstenite::connect_async(request).await?;
        assert_eq!(
            response.headers().get(SEC_WEBSOCKET_PROTOCOL),
            Some(&HeaderValue::from_static(WEBSOCKET_PEER_PROTOCOL))
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(50), event_rx.recv())
                .await
                .is_err()
        );

        let endpoint_id = iroh::SecretKey::from_bytes(&[13; 32]).public();
        client
            .send(Message::Text(serde_json::to_string(
                &WebSocketPeerHandshake {
                    peer_id: endpoint_id.to_string(),
                },
            )?))
            .await?;

        let opened = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("identified WebSocket peer should be promoted");
        let connection_id = match opened {
            NetworkEvent::PeerConnected {
                connection_id,
                peer_id,
                transport: TransportKind::WebSocket,
                direction: PeerConnectionDirection::Incoming,
            } if peer_id == PeerId::new(endpoint_id.to_string()) => connection_id,
            other => panic!("unexpected event: {other:?}"),
        };

        client
            .send(Message::Text(serde_json::to_string(&Peer2PeerMsg::Ping)?))
            .await?;
        let incoming = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward a peer message");
        assert!(matches!(
            incoming,
            NetworkEvent::PeerMessage {
                connection_id: source,
                message: Peer2PeerMsg::Ping,
            } if source == connection_id
        ));

        network
            .send_command(NetworkCommand::SendPeer {
                connection_id,
                message: Peer2PeerMsg::Pong,
            })
            .await?;
        let response = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("peer WebSocket should remain open")?;
        let Message::Text(response) = response else {
            panic!("expected a peer text response");
        };
        assert!(matches!(
            serde_json::from_str::<Peer2PeerMsg>(&response)?,
            Peer2PeerMsg::Pong
        ));

        network
            .send_command(NetworkCommand::CloseConnection {
                connection_id,
                reason: "peer WebSocket test shutdown".into(),
            })
            .await?;
        let close = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("peer WebSocket should send a close frame")?;
        assert!(matches!(close, Message::Close(_)));
        let closed = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("supervisor should forward the close event");
        assert!(matches!(
            closed,
            NetworkEvent::ConnectionClosed {
                connection_id: closed_id,
                reason: ConnectionCloseReason::LocalRequest(reason),
            } if closed_id == connection_id && reason == "peer WebSocket test shutdown"
        ));

        server_task.abort();
        let _ = server_task.await;
        network.shutdown().await?;
        tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
        Ok(())
    }

    #[tokio::test]
    async fn invalid_peer_identity_never_opens_a_peer_connection() -> Result<()> {
        let (event_tx, mut event_rx) = mpsc::channel(8);
        let (supervisor, network) = NetworkSupervisor::new(event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(network.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let server_task = tokio::spawn(async move { axum::serve(listener, app).await });

        let request = websocket_request(format!("ws://{address}/ws"), WEBSOCKET_PEER_PROTOCOL)?;
        let (mut client, _) = tokio_tungstenite::connect_async(request).await?;
        client
            .send(Message::Text(serde_json::to_string(
                &WebSocketPeerHandshake {
                    peer_id: "not-an-endpoint-id".into(),
                },
            )?))
            .await?;

        let close = tokio::time::timeout(Duration::from_secs(1), client.next())
            .await?
            .expect("invalid handshake should close the WebSocket")?;
        assert!(matches!(close, Message::Close(_)));
        let event = tokio::time::timeout(Duration::from_secs(1), event_rx.recv())
            .await?
            .expect("invalid handshake should report the closed connection");
        assert!(matches!(
            event,
            NetworkEvent::ConnectionClosed {
                reason: ConnectionCloseReason::ProtocolError(_),
                ..
            }
        ));

        server_task.abort();
        let _ = server_task.await;
        network.shutdown().await?;
        tokio::time::timeout(Duration::from_secs(1), supervisor_task).await??;
        Ok(())
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
