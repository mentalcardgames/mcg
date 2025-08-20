// Iroh transport listener for MCG server.
//
// This module accepts incoming iroh connections and speaks a simple
// newline-delimited JSON protocol where each JSON object is a
// ClientMsg or ServerMsg (the same types used over the WebSocket).
//
// The implementation mirrors the WebSocket handler behaviour: it expects
// the first message from the client to be ClientMsg::Join { name }, sends a
// ServerMsg::Welcome and the initial ServerMsg::State, and then processes
// subsequent ClientMsg messages by mutating the shared AppState.
//
// Note: this file is feature-gated behind the iroh Cargo feature. It attempts
// to follow the iroh API shown in the iroh docs. The exact iroh types and
// method names may differ across versions; treat this as the integration
// scaffolding that can be adjusted for the installed iroh crate.

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

use crate::backend::AppState;
use crate::transport::send_server_msg_to_writer;
use mcg_shared::{ClientMsg, ServerMsg};

/// Public entrypoint spawned by server startup
///
/// Refactored to delegate sub-tasks to smaller helper functions to improve
/// readability and make the high-level flow easier to follow.
pub async fn spawn_iroh_listener(state: AppState) -> Result<()> {
    // Keep the iroh-specific imports local to this function so the module does
    // not require iroh at compile time when the feature is disabled.
    use getrandom::getrandom;
    use iroh::SecretKey;

    // Application ALPN identifier (must match client)
    const ALPN: &[u8] = b"mcg/iroh/1";

    // Obtain or generate the node secret key (may persist to config)
    let secret_key: SecretKey = load_or_generate_iroh_secret(state.clone()).await;

    // Build and bind the iroh endpoint (advertising our ALPN)
    let endpoint = build_iroh_endpoint(secret_key, ALPN).await?;

    // Print node id for CLI users
    let pk = endpoint.node_id();
    println!("🔑 Iroh NodeId (public key): {}", pk);

    // Start the accept loop which will spawn a handler per connection
    start_iroh_accept_loop(endpoint, state.clone());

    println!(
        "🔗 Iroh listener started (ALPN {:?})",
        std::str::from_utf8(ALPN).unwrap_or("mcg/iroh/1")
    );
    Ok(())
}

/// Load an existing iroh secret key from state/config or generate a new one.
/// Mirrors the original persistence logic but kept in a focused helper.
async fn load_or_generate_iroh_secret(state: AppState) -> iroh::SecretKey {
    use getrandom::getrandom;
    use iroh::SecretKey;

    // Helper to generate a new random 32-byte key
    let generate_new_key = || -> SecretKey {
        let mut arr = [0u8; 32];
        if let Err(e) = getrandom(&mut arr) {
            eprintln!("Failed to get randomness for iroh key: {}", e);
        }
        SecretKey::from_bytes(&arr)
    };

    if let Some(cfg_path) = state.config_path.clone() {
        // First try a read lock to see if a key already exists in memory.
        {
            let cfg_r = state.config.read().await;
            if let Some(bytes) = cfg_r.iroh_key_bytes() {
                if bytes.len() >= 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes[..32]);
                    return SecretKey::from_bytes(&arr);
                } else {
                    // Invalid length in-memory config; fall through to generate-and-save below.
                    drop(cfg_r);
                    let sk = generate_new_key();
                    let mut cfg_w = state.config.write().await;
                    if let Err(e) = cfg_w.set_iroh_key_bytes_and_save(&cfg_path, &sk.to_bytes()) {
                        eprintln!(
                            "Failed to save generated iroh key to config '{}': {}",
                            cfg_path.display(),
                            e
                        );
                    } else {
                        println!(
                            "Saved generated iroh key into config '{}'",
                            cfg_path.display()
                        );
                    }
                    return sk;
                }
            } else {
                // No key in memory: upgrade to write lock and generate + persist.
                drop(cfg_r);
                let sk = generate_new_key();
                let mut cfg_w = state.config.write().await;
                // Double-check another writer didn't set the key while we waited for the write lock.
                if cfg_w.iroh_key_bytes().is_none() {
                    if let Err(e) = cfg_w.set_iroh_key_bytes_and_save(&cfg_path, &sk.to_bytes()) {
                        eprintln!(
                            "Failed to save generated iroh key to config '{}': {}",
                            cfg_path.display(),
                            e
                        );
                    } else {
                        println!(
                            "Saved generated iroh key into config '{}'",
                            cfg_path.display()
                        );
                    }
                    return sk;
                } else {
                    // Another writer added the key: use that one instead.
                    if let Some(bytes) = cfg_w.iroh_key_bytes() {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&bytes[..32]);
                        return SecretKey::from_bytes(&arr);
                    } else {
                        // Unlikely: fall back to generated key
                        return sk;
                    }
                }
            }
        }
    } else {
        // No config path available: generate an ephemeral key (do not persist).
        eprintln!("No server config path provided; generating ephemeral iroh key (not persisted).");
        generate_new_key()
    }
}

/// Build and bind an iroh Endpoint advertising our ALPN.
async fn build_iroh_endpoint(
    secret_key: iroh::SecretKey,
    alpn: &[u8],
) -> Result<iroh::endpoint::Endpoint> {
    use iroh::endpoint::Endpoint;

    let endpoint = Endpoint::builder()
        .alpns(vec![alpn.to_vec()])
        .secret_key(secret_key)
        .discovery_n0()
        .bind()
        .await
        .context("binding iroh endpoint")?;
    Ok(endpoint)
}

/// Spawn the accept loop which accepts connections and spawns a handler
/// task for each connection. This mirrors the previous inline logic but
/// keeps the accept loop isolated for clarity.
fn start_iroh_accept_loop(endpoint: iroh::endpoint::Endpoint, state: AppState) {
    let ep_accept = endpoint;
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            match ep_accept.accept().await {
                Some(connect_future) => match connect_future.await {
                    Ok(conn) => {
                        let state_for_conn = state_clone.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_iroh_connection(state_for_conn, conn).await {
                                eprintln!("iroh connection handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("iroh accept/connect error: {}", e);
                    }
                },
                None => {
                    // No incoming connection was ready; back off briefly to avoid tight loop.
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    });
}

// Per-connection handler which speaks newline-delimited JSON over a
// bi-directional iroh connection. Separated into smaller helpers to make
// the flow easier to reason about and unit-test individual parts.
async fn handle_iroh_connection(
    state: AppState,
    connection: iroh::endpoint::Connection,
) -> Result<()> {
    // Accept a bidirectional stream (send, recv) and wrap recv in a BufReader.
    let (mut send, recv) = connection.accept_bi().await?;
    let mut reader = BufReader::new(recv);

    // Log connect
    println!(
        "{} {}",
        "[IROH CONNECT]".bold().green(),
        "New Game Client".bold()
    );

    // Send welcome + initial state
    send_welcome_and_state(&state, &mut send).await;

    // Subscribe to global broadcasts so this iroh connection receives state updates
    // caused by other transports (e.g. websocket clients).
    let mut rx = state.broadcaster.subscribe();

    // Now enter a loop that waits for either incoming client lines or broadcast messages.
    let mut line = String::new();
    loop {
        line.clear();
        tokio::select! {
            // Broadcast events from server (State/Error/Welcome) forwarded to this client
            biased;
            recv = rx.recv() => {
                match recv {
                    Ok(sm) => {
                        if let Err(e) = send_server_msg_to_writer(&mut send, &sm).await {
                            eprintln!("iroh send error while forwarding broadcast: {}", e);
                            // On write failure, break the connection loop
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // missed messages, continue
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            // Incoming client messages (newline-delimited JSON)
            res = reader.read_line(&mut line) => {
                match res {
                    Ok(0) => break, // connection closed
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<ClientMsg>(trimmed) {
                            Ok(cm) => {
                                // Delegate processing of client messages to a helper.
                                if let Err(e) = process_client_msg(&state, &mut send, cm).await {
                                    eprintln!("error processing client message: {}", e);
                                    // On unexpected processing error, break the loop to close connection.
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = send_server_msg_to_writer(&mut send, &ServerMsg::Error(format!("Invalid JSON message: {}", e))).await;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("iroh read error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    println!(
        "{} {}",
        "[IROH DISCONNECT]".bold().red(),
        "New Game Client".bold()
    );
    // Close the send side politely if available
    let _ = send.finish();
    connection.closed().await;
    Ok(())
}

/// Send a Welcome and the initial State to the connected client.
/// Errors while writing are logged but do not abort the connection.
async fn send_welcome_and_state<W>(state: &AppState, send: &mut W)
where
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    if let Err(e) = send_server_msg_to_writer(
        send,
        //TODO: not 0
        &ServerMsg::Welcome {
            you: mcg_shared::PlayerId(0),
        },
    )
    .await
    {
        eprintln!("iroh send error: {}", e);
    }

    // Send initial state directly to this client (same behaviour as websocket)
    if let Some(gs) = crate::backend::current_state_public(state).await {
        if let Err(e) = send_server_msg_to_writer(send, &ServerMsg::State(gs)).await {
            eprintln!("iroh send error: {}", e);
        }
    }
}

/// Process a single ClientMsg received after the initial handshake.
/// This encapsulates the Action/RequestState/NextHand/NewGame handling.
async fn process_client_msg<W>(state: &AppState, send: &mut W, cm: ClientMsg) -> Result<()>
where
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    // Delegate handling to the centralized backend handler to ensure consistent behavior.
    println!("[IROH] Received ClientMsg: {:?}", cm);
    let resp = crate::backend::handle_client_msg(state, cm).await;

    if let Err(e) = send_server_msg_to_writer(send, &resp).await {
        eprintln!("iroh send error while forwarding response: {}", e);
    }
    Ok(())
}
