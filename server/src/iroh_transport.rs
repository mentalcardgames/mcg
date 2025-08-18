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
use tokio::io::{AsyncBufReadExt, AsyncWrite, BufReader};

use crate::server::AppState;
use crate::transport::send_server_msg_to_writer;
use mcg_shared::{ClientMsg, ServerMsg};

/// Public entrypoint spawned by server startup
pub async fn spawn_iroh_listener(state: AppState) -> Result<()> {
    // Import iroh types inside function to limit compile-time exposure when feature is enabled.
    // These imports are based on the iroh README snippets; they may require adjustment.
    use iroh::endpoint::Endpoint;
    use iroh::Watcher;

    // Choose an ALPN identifier for our application protocol.
    // Clients must use the same ALPN when connecting.
    const ALPN: &[u8] = b"mcg/iroh/1";

    // Build and bind an endpoint. discovery_n0() helps with relay/discovery defaults
    // in the upstream iroh examples.
    // Ensure the endpoint advertises/accepts our ALPN so accept() will match incoming connections.
    let endpoint = Endpoint::builder()
        .alpns(vec![ALPN.to_vec()])
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

// ProtocolHandler removed. Using explicit accept loop + handle_iroh_connection(...) above.

/// Per-connection handler which speaks newline-delimited JSON over a
/// bi-directional iroh connection. Separated into its own async function
/// so it can be spawned as a task from the accept loop above.
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
    if let Err(e) = crate::server::ensure_game_started(&state, &name).await {
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

    // You id for now is 0
    let you_id = 0usize;
    if let Err(e) = send_server_msg_to_writer(&mut send, &ServerMsg::Welcome { you: you_id }).await
    {
        eprintln!("iroh send error: {}", e);
    }

    // Send initial state
    if let Some(gs) = crate::server::current_state_public(&state, you_id).await {
        if let Err(e) = send_server_msg_to_writer(&mut send, &ServerMsg::State(gs)).await {
            eprintln!("iroh send error: {}", e);
        }
    }

    // Now enter a loop reading incoming JSON lines and processing ClientMsg
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            // connection closed by peer
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<ClientMsg>(trimmed) {
            Ok(cm) => {
                // Process the client message in-place (mirroring process_client_msg)
                if let Err(e) = process_client_msg_iroh(&name, &state, &mut send, cm, you_id).await
                {
                    eprintln!("iroh processing error: {}", e);
                    if let Err(e2) = send_server_msg_to_writer(
                        &mut send,
                        &ServerMsg::Error(format!("Processing error: {}", e)),
                    )
                    .await
                    {
                        eprintln!("iroh send error: {}", e2);
                    }
                    // continue processing other messages
                }
            }
            Err(e) => {
                let _ = send_server_msg_to_writer(
                    &mut send,
                    &ServerMsg::Error(format!("Invalid JSON message: {}", e)),
                )
                .await;
            }
        }
    }

    println!("{} {}", "[IROH DISCONNECT]".bold().red(), name.bold());
    // Close the send side politely if available
    let _ = send.finish();
    connection.closed().await;
    Ok(())
}



/// Process a ClientMsg received over iroh and reply over the iroh send writer.
///
/// This mirrors the behaviour of process_client_msg in the WebSocket handler,
/// but operates directly on the shared AppState and the iroh send writer.
async fn process_client_msg_iroh<W>(
    name: &str,
    state: &AppState,
    writer: &mut W,
    cm: ClientMsg,
    you_id: usize,
) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    match cm {
        ClientMsg::Action(a) => {
            println!("[IROH] Action from {}: {:?}", name, a);
            if let Some(e) = crate::server::apply_action_to_game(state, 0, a.clone()).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::Error(e)).await;
            }
            // send latest state then drive bots
            if let Some(gs) = crate::server::current_state_public(state, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
            }
            drive_bots_with_delays_iroh(writer, state, you_id, 500, 1500).await?;
        }
        ClientMsg::RequestState => {
            println!("[IROH] State requested by {}", name);
            if let Some(gs) = crate::server::current_state_public(state, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
            }
            drive_bots_with_delays_iroh(writer, state, you_id, 500, 1500).await?;
        }
        ClientMsg::NextHand => {
            println!("[IROH] NextHand requested by {}", name);
            if let Err(e) = crate::server::start_new_hand_and_print(state, you_id).await {
                let _ = send_server_msg_to_writer(
                    writer,
                    &ServerMsg::Error(format!("Failed to start new hand: {}", e)),
                )
                .await;
            }
            if let Some(gs) = crate::server::current_state_public(state, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
            }
            drive_bots_with_delays_iroh(writer, state, you_id, 500, 1500).await?;
        }
        ClientMsg::ResetGame { bots } => {
            println!("[IROH] ResetGame requested by {}: bots={} ", name, bots);
            if let Err(e) = crate::server::reset_game_with_bots(state, &name, bots, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::Error(format!("Failed to reset game: {}", e))).await;
            } else if let Some(gs) = crate::server::current_state_public(state, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
            }
            drive_bots_with_delays_iroh(writer, state, you_id, 500, 1500).await?;
        }
        ClientMsg::Join { .. } => {
            // Join is handled at connection start; ignore subsequent joins
        }
    }
    Ok(())
}

/// Drive bots similarly to the websocket handler, but send state updates over the provided writer.
async fn drive_bots_with_delays_iroh<W>(
    writer: &mut W,
    state: &AppState,
    you_id: usize,
    min_ms: u64,
    max_ms: u64,
) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    loop {
        // Perform a single bot action if it's their turn
        let did_act = {
            let mut lobby = state.lobby.write().await;
            if let Some(game) = &mut lobby.game {
                if game.stage != mcg_shared::Stage::Showdown && game.to_act != you_id {
                    let actor = game.to_act;
                    // Choose a simple bot action using the same logic as random_bot_action
                    let need = game.current_bet.saturating_sub(game.round_bets[actor]);
                    let action = if need == 0 {
                        mcg_shared::PlayerAction::Bet(game.bb)
                    } else {
                        mcg_shared::PlayerAction::CheckCall
                    };
                    let _ = game.apply_player_action(actor, action);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        // Send updated state to client
        if let Some(gs) = crate::server::current_state_public(state, you_id).await {
            let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
        }

        if !did_act {
            break;
        }
        // Sleep a pseudo-random-ish delay between actions without holding non-Send state
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
    Ok(())
}
