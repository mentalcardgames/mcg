//! Main entry point for the MCG poker server.

use native_mcg::{config, server};

use clap::Parser;
use config::ServerCli;
use std::net::{SocketAddr, TcpListener};

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server [--config PATH]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use clap-based CLI for parsing
    let cli = ServerCli::parse();

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

    // Load configuration and apply CLI overrides
    let (cfg, config_path) = config::init_server_config(&cli)?;

    let bots = cfg.bots;

    tracing::info!(config = %config_path.display(), bots);

    // Find first available port starting from 3000
    let port = find_available_port(3000)
        .map_err(|e| anyhow::anyhow!("Could not find an available port: {}", e))?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!(port, "starting server");
    if port != 3000 {
        tracing::warn!(port, "port 3000 was not available, using alternative port");
    }

    // Run the server
    server::run_server(addr, cfg, Some(config_path)).await?;
    Ok(())
}

/// Find the first available port starting from the given port number
fn find_available_port(start_port: u16) -> anyhow::Result<u16> {
    for port in start_port..start_port + 100 {
        match TcpListener::bind(("0.0.0.0", port)) {
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
