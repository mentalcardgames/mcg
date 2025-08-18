//! Main entry point for the MCG poker server.

mod pretty;
mod eval;
mod game;
mod server;
mod transport;
mod iroh_transport;
mod config;

use server::AppState;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use crate::config::Config;
use anyhow::Context;

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server [--config PATH] [--bots N]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CLI-collected overrides
    let mut cli_bots: Option<usize> = None;
    let mut config_path = PathBuf::from("mcg-server.toml");

    // Parse simple CLI args: --bots <N> and --config <PATH>
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bots" => {
                if let Some(n) = args.next() {
                    if let Ok(v) = n.parse::<usize>() {
                        cli_bots = Some(v);
                    }
                }
            }
            "--config" => {
                if let Some(p) = args.next() {
                    config_path = PathBuf::from(p);
                }
            }
            _ => {
                // Unknown flags are ignored for now
            }
        }
    }

    // Load or create config file (creates file if missing). CLI --bots overrides config.bots
    let cfg = Config::load_or_create_with_override(&config_path, cli_bots)
        .with_context(|| format!("loading or creating config '{}'", config_path.display()))?;

    let bots = cfg.bots;

    println!("Using config: {}", config_path.display());
    println!("Starting with {} bot(s)", bots);

    // Initialize shared state for the server
    let state = AppState {
        bot_count: bots,
        ..Default::default()
    };
 
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
    server::run_server(addr, state).await?;
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
