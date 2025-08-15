//! Main entry point for the MCG poker server.

mod eval;
mod game;
mod server;
mod transport;

use server::AppState;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use crate::transport::iroh_transport::IrohTransport;
use crate::transport::{Transport, WebSocketTransport};
use mcg_shared::ClientMsg;
use tokio::sync::{mpsc, Mutex};

/// Minimal server entrypoint: parse CLI args and run the server.
///
/// Usage:
///   mcg-server --bots <N> [--transport websocket|iroh] [--iroh-node-id <ID>]
#[tokio::main]
async fn main() {
    // Default settings
    let mut bots: usize = 1;
    let mut transport_choice = "websocket".to_string();
    let mut iroh_node_id_arg: Option<String> = None;

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
            "--transport" => {
                if let Some(t) = args.next() {
                    transport_choice = t;
                }
            }
            "--iroh-node-id" => {
                if let Some(id) = args.next() {
                    iroh_node_id_arg = Some(id);
                }
            }
            _ => {
                // Unknown flags are ignored for now
            }
        }
    }

    // Initialize shared state for the server
    let mut state = AppState {
        bot_count: bots,
        ..Default::default()
    };

    // Always start the Iroh transport so the server is reachable over iroh and websocket simultaneously.
    // The Iroh endpoint runs alongside the HTTP/WebSocket server; we keep a boxed transport alive.
    let mut it = IrohTransport::new();
    it.start().await.expect("Failed to start iroh transport");
    if let Some(id) = it.node_id() {
        println!("IROH node id: {}", id);
        state.transport_node_id = Some(id);
    } else {
        println!("IROH transport started but node id not yet available");
    }
    let boxed: Box<dyn Transport> = Box::new(it);
    let transport: Option<Arc<Mutex<Box<dyn Transport>>>> = Some(Arc::new(Mutex::new(boxed)));

    // If transport produces incoming messages, hook a channel to handle them centrally
    if let Some(tr) = transport.clone() {
        let (tx, mut rx) = mpsc::unbounded_channel::<(String, ClientMsg)>();
        // Provide a callback for transport incoming messages to forward into tx
        tr.lock()
            .await
            .set_on_client_message(Box::new(move |peer, cm| {
                let _ = tx.send((peer, cm));
            }));

        // Spawn a background task to process incoming transport messages
        let state_clone = state.clone();
        let tr2 = tr.clone();
        tokio::spawn(async move {
            while let Some((peer, cm)) = rx.recv().await {
                // Dispatch to processing function (mirror websocket behavior)
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
