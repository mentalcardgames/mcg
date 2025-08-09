//! Main entry point for the MCG poker server.

mod eval;
mod game;
mod server;

use server::AppState;
use std::net::SocketAddr;

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server --bots <N>
#[tokio::main]
async fn main() {
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
    let mut state = AppState::default();
    state.bot_count = bots;

    // Bind address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Run the server
    server::run_server(addr, state).await;
}
