// Run and routing helpers (build_router, run_server, SPA handlers).

use std::net::SocketAddr;

use axum::{
    http::Uri,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::services::ServeDir;

use crate::server::AppState;
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
        .route("/ws", get(crate::server::ws::ws_handler))
        // HTTP API endpoint using unified ClientMsg/ServerMsg payloads
        .route("/api/message", post(crate::server::http::message_handler))
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
            if let Err(e) = crate::backend::iroh::spawn_iroh_listener(state_clone).await {
                eprintln!("Iroh listener failed: {}", e);
            }
        });
    }

    // Continuously drive bots in the background.
    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            crate::server::bot_driver::run_bot_driver(state_clone).await;
        });
    }

    let display_addr = if addr.ip().to_string() == "127.0.0.1" {
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
