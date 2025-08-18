//! Main entry point for the MCG poker server.

mod pretty;
mod eval;
mod game;
mod backend;
mod transport;
mod iroh_transport;
mod config;
mod cli;

use backend::AppState;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use crate::config::Config;
use clap::Parser;
use anyhow::Context;

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server [--config PATH] [--bots N]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use clap-based CLI for parsing
    let cli = crate::cli::ServerCli::parse();

    let config_path: PathBuf = cli.config.clone();

    // Load or create config file (creates file if missing).
    let mut cfg = Config::load_or_create(&config_path)
        .with_context(|| format!("loading or creating config '{}'", config_path.display()))?;

    // Apply CLI overrides in-memory (non-persistent by default)
    if let Some(b) = cli.bots {
        cfg.bots = b;
    }
    if let Some(k) = cli.iroh_key {
        cfg.iroh_key = Some(k);
    }

    // Persist overrides only if requested
    if cli.persist {
        cfg.save(&config_path)
            .with_context(|| format!("saving updated config '{}'", config_path.display()))?;
    }

    let bots = cfg.bots;

    println!("Using config: {}", config_path.display());
    println!("Starting with {} bot(s)", bots);

    // Initialize shared state for the server and record config path for transports.
    let mut state = AppState {
        bot_count: bots,
        ..Default::default()
    };
    // Store config path and the loaded config into shared AppState so components
    // can access the single in-memory config instance.
    state.config_path = Some(config_path.clone());
    // Overwrite the default in-memory config with the loaded one
    {
        let mut cg = state.config.write().await;
        *cg = cfg.clone();
    }
 
    // Find first available port starting from 3000
    let port = find_available_port(3000)
        .map_err(|e| anyhow::anyhow!("Could not find an available port: {}", e))?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
 
    println!("🚀 Starting server on port {}", port);
    if port != 3000 {
        println!(
            "⚠️  Port 3000 was not available, using port {} instead",
            port
        );
    }
 
    // Run the server
    backend::run_server(addr, state).await?;
    Ok(())
}

/// Find the first available port starting from the given port number
fn find_available_port(start_port: u16) -> anyhow::Result<u16> {
    for port in start_port..start_port + 100 {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => return Ok(port),
            Err(_) => continue,
        }
    }
    Err(anyhow::anyhow!(
        "No available ports found in range {}..{}",
        start_port,
        start_port + 100
    ))
}
