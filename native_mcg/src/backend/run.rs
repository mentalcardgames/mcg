// Run and routing helpers (build_router, run_server, SPA handlers).

use std::net::SocketAddr;

use axum::{
    http::Uri,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::services::ServeDir;

use crate::backend::AppState;
use anyhow::{Context, Result};

pub fn build_router(state: AppState) -> Router {
    // Serve static files from the project root. Assumes process CWD is repo root.
    let serve_dir = ServeDir::new("pkg").append_index_html_on_directories(true);
    let serve_media = ServeDir::new("media").append_index_html_on_directories(true);

    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true })) }),
        )
        // WebSocket endpoint (WASM GUI remains websocket-only)
        .route("/ws", get(crate::backend::ws::ws_handler))
        // HTTP API endpoints (transport-agnostic server logic is reused)
        .route("/api/join", post(crate::backend::http::join_handler))
        .route("/api/action", post(crate::backend::http::action_handler))
        .route("/api/state", get(crate::backend::http::state_handler))
        .route(
            "/api/next_hand",
            post(crate::backend::http::next_hand_handler),
        )
        .route("/api/reset", post(crate::backend::http::reset_handler))
        .nest_service("/pkg", serve_dir)
        .nest_service("/media", serve_media)
        // Serve index.html for the root route
        .route("/", get(serve_index))
        // Fallback handler for SPA routing - serve index.html for all other routes
        .fallback(spa_handler)
        .with_state(state)
}

pub async fn run_server(addr: SocketAddr, state: AppState) -> Result<()> {
    let app = build_router(state.clone());

    // Spawn the iroh listener so it runs concurrently with the Axum HTTP/WebSocket server.
    // This is always enabled.
    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::iroh_transport::spawn_iroh_listener(state_clone).await {
                eprintln!("Iroh listener failed: {}", e);
            }
        });
    }

    let display_addr = if addr.ip().to_string() == "127.0.0.1" {
        format!("localhost:{}", addr.port())
    } else {
        addr.to_string()
    };

    println!("🌐 MCG Server running at http://{}", display_addr);
    println!("📱 Open your browser and navigate to the above URL");
    println!();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind to {}", display_addr))?;
    // axum::serve returns a future that runs the server; propagate any error if it returns one.
    let _ = axum::serve(listener, app).await;
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

/// SPA fallback handler - serves index.html for client-side routing
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
