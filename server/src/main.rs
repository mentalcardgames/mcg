//! Main entry point for the MCG poker server.

mod eval;
mod game;
mod server;

use server::AppState;
use std::net::{SocketAddr, TcpListener};

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server --bots <N>
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Default settings
    let mut bots: usize = 1;
 
    // Parse simple CLI args
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bots" => {
                if let Some(n) = args.next() {
                    if let Ok(v) = n.parse::<usize>() {
                        bots = v;
                    }
                }
            }
            _ => {
                // Unknown flags are ignored for now
            }
        }
    }
 
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
