//! Main entry point for the MCG poker server.

use native_mcg::{cli, config, server};

use anyhow::Context;
use clap::Parser;
use config::Config;
use server::AppState;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server [--config PATH]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use clap-based CLI for parsing
    // TODO extract config init into utility function
    let cli = cli::ServerCli::parse();

    // Initialize tracing subscriber for logging
    // If debug is on: show everything at DEBUG level
    // If debug is off: show native_mcg at INFO, everything else at WARN/ERROR to reduce noise
    let log_filter = if cli.debug {
        "debug".to_string()
    } else {
        // Default to info for our crate, warn/error for others to keep noise down
        // Suppress noisy netlink warnings on newer kernels
        "native_mcg=info,mcg_shared=info,warn,iroh=error,iroh::magicsock=error,netlink_packet_route=error".to_string()
    };

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_filter));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        // Use compact format in non-debug mode for cleaner output
        .with_target(cli.debug)
        .with_thread_ids(cli.debug)
        .with_file(cli.debug)
        .with_line_number(cli.debug)
        .init();

    let config_path: PathBuf = cli.config.clone();

    // Load or create config file (creates file if missing).
    let mut cfg = Config::load_or_create(&config_path)
        .with_context(|| format!("loading or creating config '{}'", config_path.display()))?;

    // Apply CLI overrides in-memory (non-persistent by default)
    if let Some(k) = cli.iroh_key {
        cfg.iroh_key = Some(k);
    }

    // Persist overrides only if requested
    if cli.persist {
        cfg.save(&config_path)
            .with_context(|| format!("saving updated config '{}'", config_path.display()))?;
    }

    let bots = cfg.bots;

    tracing::info!(config = %config_path.display(), bots);

    // Initialize shared state for the server and record config path for transports.
    let state = AppState::new(cfg.clone(), Some(config_path.clone()));

    // Find first available port starting from 3000
    let port = find_available_port(3000)
        .map_err(|e| anyhow::anyhow!("Could not find an available port: {}", e))?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!(port, "starting server");
    if port != 3000 {
        tracing::warn!(port, "port 3000 was not available, using alternative port");
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
