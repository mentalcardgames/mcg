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

use crate::controller::{
    spawn_controller_command_forwarder, spawn_network_event_forwarder, start_controller,
    ChannelControllerSink, Controller, ControllerHandle,
};
use crate::network::{NetworkEvent, NetworkHandle, NetworkSupervisor};
use crate::server::{
    bot_driver::spawn_bot_driver, peer_connections::PeerConnectionService, AppState,
};
use anyhow::{Context, Result};
use tokio::sync::mpsc;

const NETWORK_EVENT_CHANNEL_CAPACITY: usize = 256;

#[derive(Clone)]
struct RouterState {
    app: AppState,
    controller: ControllerHandle,
    network: NetworkHandle,
    peer_connections: PeerConnectionService,
    _network_tasks: Arc<NetworkTasks>,
}

impl FromRef<RouterState> for AppState {
    fn from_ref(state: &RouterState) -> Self {
        state.app.clone()
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

struct NetworkTasks {
    controller_handle: ControllerHandle,
    supervisor: Mutex<Option<tokio::task::JoinHandle<()>>>,
    event_forwarder: Mutex<Option<tokio::task::JoinHandle<()>>>,
    command_forwarder: Mutex<Option<tokio::task::JoinHandle<()>>>,
    bot_driver: Mutex<Option<tokio::task::JoinHandle<()>>>,
    controller_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl NetworkTasks {
    async fn shutdown(&self) {
        let _ = self.controller_handle.shutdown().await;

        let bot_driver = self
            .bot_driver
            .lock()
            .expect("bot driver task lock poisoned")
            .take();
        if let Some(bot_driver) = bot_driver {
            bot_driver.abort();
        }

        let supervisor = self
            .supervisor
            .lock()
            .expect("network supervisor task lock poisoned")
            .take();
        if let Some(supervisor) = supervisor {
            if let Err(error) = supervisor.await {
                tracing::error!(%error, "network supervisor task failed during shutdown");
            }
        }

        let event_forwarder = self
            .event_forwarder
            .lock()
            .expect("event forwarder task lock poisoned")
            .take();
        if let Some(event_forwarder) = event_forwarder {
            if let Err(error) = event_forwarder.await {
                tracing::error!(%error, "network event forwarder failed during shutdown");
            }
        }

        let command_forwarder = self
            .command_forwarder
            .lock()
            .expect("command forwarder task lock poisoned")
            .take();
        if let Some(command_forwarder) = command_forwarder {
            if let Err(error) = command_forwarder.await {
                tracing::error!(%error, "controller command forwarder failed during shutdown");
            }
        }

        let controller_thread = self
            .controller_thread
            .lock()
            .expect("controller thread lock poisoned")
            .take();
        if let Some(controller_thread) = controller_thread {
            if let Err(error) = controller_thread.join() {
                tracing::error!(?error, "controller thread panicked during shutdown");
            }
        }
    }
}

impl Drop for NetworkTasks {
    fn drop(&mut self) {
        if let Some(bot_driver) = self
            .bot_driver
            .get_mut()
            .expect("bot driver task lock poisoned")
            .take()
        {
            bot_driver.abort();
        }
        if let Some(supervisor) = self
            .supervisor
            .get_mut()
            .expect("network supervisor task lock poisoned")
            .take()
        {
            supervisor.abort();
        }
        if let Some(event_forwarder) = self
            .event_forwarder
            .get_mut()
            .expect("event forwarder task lock poisoned")
            .take()
        {
            event_forwarder.abort();
        }
        if let Some(command_forwarder) = self
            .command_forwarder
            .get_mut()
            .expect("command forwarder task lock poisoned")
            .take()
        {
            command_forwarder.abort();
        }
    }
}

impl RouterState {
    fn new(
        app: AppState,
        controller: ControllerHandle,
        network: NetworkHandle,
        peer_connections: PeerConnectionService,
        network_tasks: Arc<NetworkTasks>,
    ) -> Self {
        Self {
            app,
            controller,
            network,
            peer_connections,
            _network_tasks: network_tasks,
        }
    }
}

fn start_network(
    app: AppState,
) -> (
    ControllerHandle,
    NetworkHandle,
    PeerConnectionService,
    Arc<NetworkTasks>,
) {
    let (event_tx, event_rx) = mpsc::channel::<NetworkEvent>(NETWORK_EVENT_CHANNEL_CAPACITY);
    let (supervisor, network) = NetworkSupervisor::new(event_tx);
    let peer_connections = PeerConnectionService::new(app.clone(), network.clone());

    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let sink = ChannelControllerSink::new(command_tx);
    let config = app.config.try_read().map(|c| c.clone()).unwrap_or_default();
    let config_path = app.config_path.clone();
    let controller = Controller::new(config.clone(), config_path);
    let (controller_thread, controller_handle) = start_controller(controller, 256, sink);

    let (state_watch_tx, state_watch_rx) = tokio::sync::watch::channel(None);
    let event_forwarder = spawn_network_event_forwarder(
        event_rx,
        controller_handle.clone(),
        Some(peer_connections.clone()),
    );
    let command_forwarder = spawn_controller_command_forwarder(
        command_rx,
        network.clone(),
        Some(peer_connections.clone()),
        Some(state_watch_tx),
    );
    let supervisor = tokio::spawn(supervisor.run());
    let bot_delay_range = config.bot_delay_range();
    let bot_driver = spawn_bot_driver(
        controller_handle.clone(),
        state_watch_rx,
        crate::bot::BotManager::new(),
        bot_delay_range,
    );

    (
        controller_handle.clone(),
        network,
        peer_connections,
        Arc::new(NetworkTasks {
            controller_handle,
            supervisor: Mutex::new(Some(supervisor)),
            event_forwarder: Mutex::new(Some(event_forwarder)),
            command_forwarder: Mutex::new(Some(command_forwarder)),
            bot_driver: Mutex::new(Some(bot_driver)),
            controller_thread: Mutex::new(Some(controller_thread)),
        }),
    )
}

fn build_router_with_network(
    state: AppState,
    controller: ControllerHandle,
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
            controller,
            network,
            peer_connections,
            network_tasks,
        ))
}

pub fn build_router(state: AppState) -> Router {
    let (controller, network, peer_connections, network_tasks) = start_network(state.clone());
    build_router_with_network(state, controller, network, peer_connections, network_tasks)
}

pub async fn run_server(addr: SocketAddr, state: AppState) -> Result<()> {
    let (controller, network, peer_connections, network_tasks) = start_network(state.clone());
    let app = build_router_with_network(
        state.clone(),
        controller,
        network.clone(),
        peer_connections,
        network_tasks.clone(),
    );

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
