//! Main entry point for the MCG poker server.

mod eval;
mod game;
mod server;
mod transport;

use server::AppState;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use crate::transport::iroh_transport::IrohTransport;
use crate::transport::Transport;
use tokio::sync::{mpsc, Mutex};

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server --bots <N> [--transport websocket|iroh] [--iroh-node-id <ID>]
#[tokio::main]
async fn main() {
    // Parse CLI args using clap
    #[derive(clap::Parser, Debug)]
    struct Cli {
        /// Number of bots
        #[arg(long, default_value_t = 1)]
        bots: usize,

        /// Enable debug logging
        #[arg(long)]
        debug: bool,
    }

    use clap::Parser;
    let cli = Cli::parse();

    // Initialize shared state for the server
    let mut state = AppState {
        bot_count: cli.bots,
        ..Default::default()
    };

    // Accept an optional --debug flag for verbose iroh event logging
    let debug = cli.debug;

    // Construct iroh transport and obtain the receiver for parsed inbound messages
    let (it, mut iroh_rx) = IrohTransport::new(debug).await.expect("Failed to start iroh transport");

    let id = it.node_id();
    println!("IROH node id: {}", id);
    state.transport_node_id = Some(id.clone());

    // Print a more prominent full NodeTicket string so CLI callers can dial using a concrete ticket.
    let full_na = it.node_addr_string().await;
    println!("IROH full node_addr: {}", full_na);

    let boxed: Box<dyn Transport> = Box::new(it);
    let transport: Arc<Mutex<Box<dyn Transport>>> = Arc::new(Mutex::new(boxed));

    // Register transport in server state transports so broadcasts reach it
    {
        let mut transports_lock = state.transports.lock().await;
        transports_lock.push(transport.clone());
    }

    // Spawn a background task to forward parsed iroh protocol messages into the central processing loop
    {
        let state_clone = state.clone();
        let tr2 = transport.clone();
        tokio::spawn(async move {
            while let Some((peer, cm)) = iroh_rx.recv().await {
                crate::server::process_client_msg_from_transport(
                    peer,
                    &state_clone,
                    cm,
                    Some(tr2.clone()),
                )
                .await;
            }
        });
    }

    // Find first available port starting from 3000
    let port = find_available_port(3000).expect("Could not find an available port");
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("🚀 Starting server on port {}", port);
    if port != 3000 {
        println!(
            "⚠️  Port 3000 was not available, using port {} instead",
            port
        );
    }

    // Run the server
    server::run_server(addr, state).await;
}

/// Find the first available port starting from the given port number
fn find_available_port(start_port: u16) -> Result<u16, std::io::Error> {
    for port in start_port..start_port + 100 {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => return Ok(port),
            Err(_) => continue,
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "No available ports found in range",
    ))
}
