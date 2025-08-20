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
pub async fn spawn_iroh_listener(state: AppState) -> Result<()> {
    // Import iroh types inside function to limit compile-time exposure when feature is enabled.
    // These imports are based on the iroh README snippets; they may require adjustment.
    use getrandom::getrandom;
    use iroh::endpoint::Endpoint;
    use iroh::SecretKey;

    // Choose an ALPN identifier for our application protocol.
    // Clients must use the same ALPN when connecting.
    const ALPN: &[u8] = b"mcg/iroh/1";

    // Build and bind an endpoint. discovery_n0() helps with relay/discovery defaults
    // in the upstream iroh examples.
    // Ensure the endpoint advertises/accepts our ALPN so accept() will match incoming connections.
    // Load or generate persistent secret key for stable NodeId.
    //
    // IMPORTANT: Use only the TOML config for persistence. Do NOT read or write
    // $HOME/.iroh/keypair or other filesystem locations for key storage.
    let secret_key: SecretKey = {
        // Helper to generate a new random 32-byte key
        let generate_new_key = || -> SecretKey {
            let mut arr = [0u8; 32];
            if let Err(e) = getrandom(&mut arr) {
                eprintln!("Failed to get randomness for iroh key: {}", e);
            }
            SecretKey::from_bytes(&arr)
        };

        // If a server config path is available, use the in-memory shared config in AppState
        // as the canonical store. Persist new keys to disk only when a config path is present.
        if let Some(cfg_path) = state.config_path.clone() {
            // First try a read lock to see if a key already exists in memory.
            {
                let cfg_r = state.config.read().await;
                if let Some(bytes) = cfg_r.iroh_key_bytes() {
                    if bytes.len() >= 32 {
                        let mut arr = [0u8; 32];
                        arr.copy_from_slice(&bytes[..32]);
                        SecretKey::from_bytes(&arr)
                    } else {
                        // Invalid length in-memory config; fall through to generate-and-save below.
                        // Drop read lock and acquire write lock.
                        drop(cfg_r);
                        // generate below
                        let sk = generate_new_key();
                        let mut cfg_w = state.config.write().await;
                        if let Err(e) = cfg_w.set_iroh_key_bytes_and_save(&cfg_path, &sk.to_bytes())
                        {
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
                        sk
                    }
                } else {
                    // No key in memory: upgrade to write lock and generate + persist.
                    drop(cfg_r);
                    let sk = generate_new_key();
                    let mut cfg_w = state.config.write().await;
                    // Double-check another writer didn't set the key while we waited for the write lock.
                    if cfg_w.iroh_key_bytes().is_none() {
                        if let Err(e) = cfg_w.set_iroh_key_bytes_and_save(&cfg_path, &sk.to_bytes())
                        {
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
                        sk
                    } else {
                        // Another writer added the key: use that one instead.
                        if let Some(bytes) = cfg_w.iroh_key_bytes() {
                            let mut arr = [0u8; 32];
                            arr.copy_from_slice(&bytes[..32]);
                            SecretKey::from_bytes(&arr)
                        } else {
                            // Unlikely: fall back to generated key
                            sk
                        }
                    }
                }
            }
        } else {
            // No config path available: generate an ephemeral key (do not persist).
            eprintln!(
                "No server config path provided; generating ephemeral iroh key (not persisted)."
            );
            generate_new_key()
        }
    };

    let endpoint = Endpoint::builder()
        .alpns(vec![ALPN.to_vec()])
        .secret_key(secret_key)
        .discovery_n0()
        .bind()
        .await
        .context("binding iroh endpoint")?;
    // Print the local node's public key (NodeId) so CLI users can dial by it.
    // Use the Endpoint::node_id() accessor and print its z32 representation.
    let pk = endpoint.node_id();
    println!("🔑 Iroh NodeId (public key): {}", pk);

    // Use Endpoint.accept() to receive incoming connections. The `accept()`
    // call returns an Option-like incoming value which must be awaited to
    // obtain a connected `Connection`. Spawn a background task that loops
    // accepting connections and spawning a handler task for each connection.
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

    println!(
        "🔗 Iroh listener started (ALPN {:?})",
        std::str::from_utf8(ALPN).unwrap_or("mcg/iroh/1")
    );
    Ok(())
}

// Per-connection handler which speaks newline-delimited JSON over a
// bi-directional iroh connection. Separated into its own async function
// so it can be spawned as a task from the accept loop above.
async fn handle_iroh_connection(
    state: AppState,
    connection: iroh::endpoint::Connection,
) -> Result<()> {
    // Accept a bidirectional stream (send, recv)
    let (mut send, recv) = connection.accept_bi().await?;
    // Wrap the read side in a buffered reader so we can read lines.
    let mut reader = BufReader::new(recv);

    // Read the very first line from the client; expect a Join message.
    let mut first_line = String::new();
    let n = reader.read_line(&mut first_line).await?;
    if n == 0 {
        return Ok(());
    }
    let first_trim = first_line.trim();
    let cm: ClientMsg = match serde_json::from_str(first_trim) {
        Ok(cm) => cm,
        Err(_) => {
            // Send error and close
            if let Err(e) =
                send_server_msg_to_writer(&mut send, &ServerMsg::Error("Expected Join".into()))
                    .await
            {
                eprintln!("iroh send error: {}", e);
            }
            return Ok(());
        }
    };

    let name = match cm {
        ClientMsg::Join { name } => name,
        _ => {
            if let Err(e) =
                send_server_msg_to_writer(&mut send, &ServerMsg::Error("Expected Join".into()))
                    .await
            {
                eprintln!("iroh send error: {}", e);
            }
            return Ok(());
        }
    };

    // Log connect
    println!("{} {}", "[IROH CONNECT]".bold().green(), name.bold());

    // Ensure game started similarly to the websocket handler
    if let Err(e) = crate::backend::ensure_game_started(&state, &name).await {
        if let Err(e2) = send_server_msg_to_writer(
            &mut send,
            &ServerMsg::Error(format!("Failed to start game: {}", e)),
        )
        .await
        {
            eprintln!("iroh send error: {}", e2);
        }
        return Ok(());
    }

    // Register player and obtain server-assigned you_id. If registration fails,
    // send an error back to the client and close the connection.
    let you_id = match crate::backend::state::register_player_id(&state, &name).await {
        Ok(id) => id,
        Err(e) => {
            if let Err(e2) = send_server_msg_to_writer(&mut send, &ServerMsg::Error(format!("Failed to register player: {}", e))).await {
                eprintln!("iroh send error: {}", e2);
            }
            return Ok(());
        }
    };
    if let Err(e) = send_server_msg_to_writer(&mut send, &ServerMsg::Welcome { you: you_id }).await {
        eprintln!("iroh send error: {}", e);
    }

    // Send initial state directly to this client (same behaviour as websocket)
    if let Some(gs) = crate::backend::current_state_public(&state, you_id).await {
        if let Err(e) = send_server_msg_to_writer(&mut send, &ServerMsg::State(gs)).await {
            eprintln!("iroh send error: {}", e);
        }
    }

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
                                // Process the client message and use server-side broadcast functions
                                match cm {
                                    ClientMsg::Action { player_id, action } => {
                                        println!("[IROH] Action from {}: player_id={} action={:?}", name, player_id, action);

                                        match crate::backend::validate_and_apply_action(&state, player_id, action.clone()).await {
                                            Ok(()) => {
                                                // Send updated state to this client
                                                if let Some(gs) = crate::backend::current_state_public(&state, player_id).await {
                                                    let _ = send_server_msg_to_writer(&mut send, &ServerMsg::State(gs)).await;
                                                }
                                                // Broadcast and drive via centralized helper (drive_bots respects bots_auto)
                                                crate::backend::broadcast_and_drive(&state, you_id, 500, 1500).await;
                                            }
                                            Err(e) => {
                                                let _ = send_server_msg_to_writer(&mut send, &ServerMsg::Error(e)).await;
                                            }
                                        }
                                    }
                                    ClientMsg::RequestState { player_id } => {
                                        println!("[IROH] State requested by {} for player {}", name, player_id);
                                        // Send state directly to this client
                                        if let Some(gs) = crate::backend::current_state_public(&state, player_id).await {
                                            let _ = send_server_msg_to_writer(&mut send, &ServerMsg::State(gs)).await;
                                        }
                                        crate::backend::broadcast_and_drive(&state, you_id, 500, 1500).await;
                                    }
                                    ClientMsg::NextHand { player_id } => {
                                        println!("[IROH] NextHand requested by {} for player {}", name, player_id);
                                        if let Err(e) = crate::backend::start_new_hand_and_print(&state, player_id).await {
                                            let _ = send_server_msg_to_writer(&mut send, &ServerMsg::Error(format!("Failed to start new hand: {}", e))).await;
                                        }
                                        if let Some(gs) = crate::backend::current_state_public(&state, player_id).await {
                                            let _ = send_server_msg_to_writer(&mut send, &ServerMsg::State(gs)).await;
                                        }
                                        crate::backend::broadcast_and_drive(&state, you_id, 500, 1500).await;
                                    }
                                    ClientMsg::ResetGame { bots, bots_auto } => {
                                        println!("[IROH] ResetGame requested by {}: bots={} bots_auto={}", name, bots, bots_auto);

                                        // Persist bots_auto preference in lobby
                                        {
                                            let mut lobby_w = state.lobby.write().await;
                                            lobby_w.bots_auto = bots_auto;
                                        }

                                        if let Err(e) = crate::backend::reset_game_with_bots(&state, &name, bots, you_id).await {
                                            let _ = send_server_msg_to_writer(&mut send, &ServerMsg::Error(format!("Failed to reset game: {}", e))).await;
                                        } else {
                                            crate::backend::broadcast_and_drive(&state, you_id, 500, 1500).await;
                                        }
                                    }
                                    ClientMsg::Join { .. } => {
                                        // Ignore subsequent joins
                                    }
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

    println!("{} {}", "[IROH DISCONNECT]".bold().red(), name.bold());
    // Close the send side politely if available
    let _ = send.finish();
    connection.closed().await;
    Ok(())
}
