// Run and routing helpers (build_router, run_server, SPA handlers).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{
    extract::FromRef,
    http::Uri,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::services::ServeDir;

use crate::network::{NetworkEvent, NetworkHandle, NetworkSupervisor};
use crate::server::{
    network_adapter::LegacyBackendAdapter, peer_connections::PeerConnectionService, AppState,
};
use anyhow::{Context, Result};
use tokio::sync::mpsc;

const NETWORK_EVENT_CHANNEL_CAPACITY: usize = 256;

#[derive(Clone)]
struct RouterState {
    app: AppState,
    network: NetworkHandle,
    peer_connections: PeerConnectionService,
    _network_tasks: Arc<NetworkTasks>,
}

impl FromRef<RouterState> for AppState {
    fn from_ref(state: &RouterState) -> Self {
        state.app.clone()
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

struct NetworkTasks {
    supervisor: Mutex<Option<tokio::task::JoinHandle<()>>>,
    adapter: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl NetworkTasks {
    async fn shutdown(&self) {
        let supervisor = self
            .supervisor
            .lock()
            .expect("network supervisor task lock poisoned")
            .take();
        let adapter = self
            .adapter
            .lock()
            .expect("network adapter task lock poisoned")
            .take();

        if let Some(supervisor) = supervisor {
            if let Err(error) = supervisor.await {
                tracing::error!(%error, "network supervisor task failed during shutdown");
            }
        }
        if let Some(adapter) = adapter {
            if let Err(error) = adapter.await {
                tracing::error!(%error, "network adapter task failed during shutdown");
            }
        }
    }
}

impl Drop for NetworkTasks {
    fn drop(&mut self) {
        if let Some(supervisor) = self
            .supervisor
            .get_mut()
            .expect("network supervisor task lock poisoned")
            .take()
        {
            supervisor.abort();
        }
        if let Some(adapter) = self
            .adapter
            .get_mut()
            .expect("network adapter task lock poisoned")
            .take()
        {
            adapter.abort();
        }
    }
}

impl RouterState {
    fn new(
        app: AppState,
        network: NetworkHandle,
        peer_connections: PeerConnectionService,
        network_tasks: Arc<NetworkTasks>,
    ) -> Self {
        Self {
            app,
            network,
            peer_connections,
            _network_tasks: network_tasks,
        }
    }
}

fn start_network(app: AppState) -> (NetworkHandle, PeerConnectionService, Arc<NetworkTasks>) {
    let (event_tx, event_rx) = mpsc::channel::<NetworkEvent>(NETWORK_EVENT_CHANNEL_CAPACITY);
    let (supervisor, network) = NetworkSupervisor::new(event_tx);
    let peer_connections = PeerConnectionService::new(app.clone(), network.clone());
    let adapter =
        LegacyBackendAdapter::new(app, network.clone(), peer_connections.clone(), event_rx);
    let supervisor = tokio::spawn(supervisor.run());
    let adapter = tokio::spawn(adapter.run());

    (
        network,
        peer_connections,
        Arc::new(NetworkTasks {
            supervisor: Mutex::new(Some(supervisor)),
            adapter: Mutex::new(Some(adapter)),
        }),
    )
}

fn build_router_with_network(
    state: AppState,
    network: NetworkHandle,
    peer_connections: PeerConnectionService,
    network_tasks: Arc<NetworkTasks>,
) -> Router {
    // Serve static files from the project root. Assumes process CWD is repo root.
    let serve_dir = ServeDir::new("pkg").append_index_html_on_directories(true);
    let serve_media = ServeDir::new("media").append_index_html_on_directories(true);

    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true })) }),
        )
        // WebSocket endpoint (WASM GUI remains websocket-only)
        .route("/ws", get(crate::server::ws::ws_handler))
        // HTTP API endpoint using unified Frontend2BackendMsg/Backend2FrontendMsg payloads
        .route("/api/message", post(crate::server::http::message_handler))
        .nest_service("/pkg", serve_dir)
        .nest_service("/media", serve_media)
        // Serve index.html for the root route
        .route("/", get(serve_index))
        // Fallback handler for SPA routing - serve index.html for all other routes
        .fallback(spa_handler)
        .with_state(RouterState::new(
            state,
            network,
            peer_connections,
            network_tasks,
        ))
}

pub fn build_router(state: AppState) -> Router {
    let (network, peer_connections, network_tasks) = start_network(state.clone());
    build_router_with_network(state, network, peer_connections, network_tasks)
}

pub async fn run_server(addr: SocketAddr, state: AppState) -> Result<()> {
    let (network, peer_connections, network_tasks) = start_network(state.clone());
    let app = build_router_with_network(
        state.clone(),
        network.clone(),
        peer_connections,
        network_tasks.clone(),
    );

    // Continuously drive bots in the background.
    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            crate::server::bot_driver::run_bot_driver(state_clone).await;
        });
    }

    let display_addr = if addr.ip().is_loopback() {
        format!("localhost:{}", addr.port())
    } else {
        addr.to_string()
    };

    tracing::info!(display_addr = %display_addr, "MCG Server running");

    // Nice clickable banner for the Web UI
    println!("\n\x1b[1;36m=== Web UI Available ===\x1b[0m");
    println!(
        "\x1b[1mURL:\x1b[0m       \x1b[4;34mhttp://{}\x1b[0m",
        display_addr
    );
    println!("\x1b[1;36m========================\x1b[0m\n");

    tracing::info!("open your browser and navigate to the above URL");
    tracing::debug!("blank line");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind to {}", display_addr))?;
    // The owned task starts Iroh concurrently and is shut down after Axum stops.
    let iroh_listener = crate::server::iroh::spawn_iroh_listener(state, network.clone());
    let server_result = axum::serve(listener, app).await;
    iroh_listener.shutdown().await;
    if let Err(error) = network.shutdown().await {
        tracing::warn!(%error, "network supervisor stopped before server shutdown");
    }
    network_tasks.shutdown().await;
    server_result.context("running HTTP/WebSocket server")?;
    Ok(())
}

/// Serve index.html file
async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("index.html").await {
        Ok(content) => (
            axum::http::StatusCode::OK,
            [("content-type", "text/html")],
            content,
        )
            .into_response(),
        Err(_) => (axum::http::StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

/// Single Page Application (SPA) fallback handler - serves index.html for client-side routing
async fn spa_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path();

    // Don't serve index.html for API routes or asset requests
    if path.starts_with("/api")
        || path.starts_with("/pkg")
        || path.starts_with("/media")
        || path.starts_with("/ws")
        || path.starts_with("/health")
    {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }

    // For all other routes, serve index.html to enable client-side routing
    serve_index().await.into_response()
}
