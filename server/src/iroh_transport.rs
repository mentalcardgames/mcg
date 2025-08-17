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
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWrite, BufReader};
use owo_colors::OwoColorize;
use std::io::IsTerminal;

use crate::server::AppState;
use mcg_shared::{ClientMsg, GameStatePublic, ServerMsg};
use crate::transport::send_server_msg_to_writer;

/// Public entrypoint spawned by server startup
pub async fn spawn_iroh_listener(state: AppState) -> Result<()> {
    // Import iroh types inside function to limit compile-time exposure when feature is enabled.
    // These imports are based on the iroh README snippets; they may require adjustment.
    use iroh::endpoint::Endpoint;
    use iroh::protocol::Router;

    // Choose an ALPN identifier for our application protocol.
    // Clients must use the same ALPN when connecting.
    const ALPN: &[u8] = b"mcg/iroh/1";

    // Build and bind an endpoint. discovery_n0() helps with relay/discovery defaults
    // in the upstream iroh examples.
    let endpoint = Endpoint::builder()
        .discovery_n0()
        .bind()
        .await
        .context("binding iroh endpoint")?;
    // Print the local node's public key (NodeId) so CLI users can dial by it.
    // Use the Endpoint::node_id() accessor and print its z32 representation.
    let pk = endpoint.node_id();
    println!("🔑 Iroh NodeId (public key): {}", pk);

    // Router builder: accept our ALPN and handle incoming protocol streams using
    // the ProtocolHandler implementation below.
    // spawn() returns a result handle; do not await the router itself.
    let _router = Router::builder(endpoint)
        .accept(ALPN.to_vec(), Arc::new(IrohHandler { state: state.clone() }))
        .spawn();

    // The router future runs in background; keep a handle until shutdown.
    // The spawned router will run until the endpoint is closed or an error occurs.
    // The router is spawned and running; keep the returned handle (named `_router`) alive.
    // No explicit await is required here.

    println!("🔗 Iroh listener started (ALPN {:?})", std::str::from_utf8(ALPN).unwrap_or("mcg/iroh/1"));
    Ok(())
}

/// Protocol handler for accepted iroh connections.
///
/// The iroh Router will call accept on this handler for each incoming connection.
/// The handler accepts a bidirectional stream and speaks newline-delimited JSON.
#[derive(Clone)]
struct IrohHandler {
    state: AppState,
}

impl std::fmt::Debug for IrohHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IrohHandler").finish()
    }
}
 
// The ProtocolHandler trait uses a `fn accept(...) -> impl Future` style API.
// Implement by returning a boxed future that runs our async connection handler.
impl iroh::protocol::ProtocolHandler for IrohHandler {
    fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> impl std::future::Future<Output = Result<(), iroh::protocol::AcceptError>> + Send {
        let state = self.state.clone();
        Box::pin(async move {
            use iroh::protocol::AcceptError;
            // Accept a bidirectional stream (send, recv)
            let (mut send, mut recv) = connection.accept_bi().await?;
 
            // Wrap the read side in a buffered reader so we can read lines.
            let mut reader = BufReader::new(recv);
 
            // We'll use shared helper send_server_msg_to_writer to write JSON lines to `send`.
 
            // Read the very first line from the client; expect a Join message.
            let mut first_line = String::new();
            let n = reader.read_line(&mut first_line).await.map_err(AcceptError::from_err)?;
            if n == 0 {
                return Ok(());
            }
            let first_trim = first_line.trim();
            let cm: ClientMsg = match serde_json::from_str(first_trim) {
                Ok(cm) => cm,
                Err(_) => {
                    // Send error and close
                    if let Err(e) = send_server_msg_to_writer(&mut send, &ServerMsg::Error("Expected Join".into())).await {
                        eprintln!("iroh send error: {}", e);
                    }
                    return Ok(());
                }
            };

            let name = match cm {
                ClientMsg::Join { name } => name,
                _ => {
                    if let Err(e) = send_server_msg_to_writer(&mut send, &ServerMsg::Error("Expected Join".into())).await {
                        eprintln!("iroh send error: {}", e);
                    }
                    return Ok(());
                }
            };
 
            // Log connect
            println!("{} {}", "[IROH CONNECT]".bold().green(), name.bold());
 
            // Ensure game started similarly to the websocket handler
            if let Err(e) = ensure_game_started(&state, &name).await {
                if let Err(e2) = send_server_msg_to_writer(&mut send, &ServerMsg::Error(format!("Failed to start game: {}", e))).await {
                    eprintln!("iroh send error: {}", e2);
                }
                return Ok(());
            }
 
            // You id for now is 0
            let you_id = 0usize;
            if let Err(e) = send_server_msg_to_writer(&mut send, &ServerMsg::Welcome { you: you_id }).await {
                eprintln!("iroh send error: {}", e);
            }
    
            // Send initial state
            if let Some(gs) = current_state_public(&state, you_id).await {
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
                        if let Err(e) =
                            process_client_msg_iroh(&name, &state, &mut send, cm, you_id).await
                        {
                            eprintln!("iroh processing error: {}", e);
                            if let Err(e2) = send_server_msg_to_writer(&mut send, &ServerMsg::Error(format!("Processing error: {}", e))).await {
                                eprintln!("iroh send error: {}", e2);
                            }
                            // continue processing other messages
                        }
                }
                Err(e) => {
                    let _ = send_server_msg_to_writer(&mut send, &ServerMsg::Error(format!("Invalid JSON message: {}", e))).await;
                }
            }
        }

            println!("{} {}", "[IROH DISCONNECT]".bold().red(), name.bold());
            // Close the send side politely if available
            let _ = send.finish();
            connection.closed().await;
            Ok(())
        })
    }
}

/// Ensure the game is started for the connecting player.
///
/// This duplicates logic from the WS handler in server.rs to avoid cross-file
/// privatization changes. It mutates the shared AppState.
async fn ensure_game_started(state: &AppState, name: &str) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if lobby.game.is_none() {
        let g = crate::game::Game::new(name.to_string(), state.bot_count)
            .with_context(|| format!("creating new game for {}", name))?;
        lobby.game = Some(g);
        println!(
            "[GAME] Created new game for {} with {} bot(s)",
            name, state.bot_count
        );
    }
    Ok(())
}

/// Get public view of the current state for a given viewer id.
async fn current_state_public(state: &AppState, you_id: usize) -> Option<GameStatePublic> {
    let lobby = state.lobby.read().await;
    lobby.game.as_ref().map(|g| g.public_for(you_id))
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
            let mut err: Option<String> = None;
            {
                let mut lobby = state.lobby.write().await;
                if let Some(game) = &mut lobby.game {
                    if let Err(e) = game.apply_player_action(0, a.clone()) {
                        err = Some(e.to_string());
                    }
                }
            }
            if let Some(e) = err {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::Error(e)).await;
            }
            // send latest state then drive bots
            if let Some(gs) = current_state_public(state, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
            }
            drive_bots_with_delays_iroh(writer, state, you_id, 500, 1500).await?;
        }
        ClientMsg::RequestState => {
            println!("[IROH] State requested by {}", name);
            if let Some(gs) = current_state_public(state, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
            }
            drive_bots_with_delays_iroh(writer, state, you_id, 500, 1500).await?;
        }
        ClientMsg::NextHand => {
            println!("[IROH] NextHand requested by {}", name);
            {
                let mut lobby = state.lobby.write().await;
                if let Some(game) = &mut lobby.game {
                    let n = game.players.len();
                    if n > 0 {
                        game.dealer_idx = (game.dealer_idx + 1) % n;
                    }
                    if let Err(e) = game.start_new_hand() {
                        let _ = send_server_msg_to_writer(writer, &ServerMsg::Error(format!("Failed to start new hand: {}", e))).await;
                    } else {
                        // After starting a new hand, print a table header banner once
                        let sb = game.sb;
                        let bb = game.bb;
                        let gs = game.public_for(you_id);
                        lobby.last_printed_log_len = gs.action_log.len();
                        let header = mcg_server::pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
                        println!("{}", header);
                    }
                }
            }
            if let Some(gs) = current_state_public(state, you_id).await {
                let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
            }
            drive_bots_with_delays_iroh(writer, state, you_id, 500, 1500).await?;
        }
        ClientMsg::ResetGame { bots } => {
            println!("[IROH] ResetGame requested by {}: bots={} ", name, bots);
            {
                let mut lobby = state.lobby.write().await;
                match crate::game::Game::new(name.to_string(), bots) {
                    Ok(g) => {
                        lobby.game = Some(g);
                        if let Some(game) = &mut lobby.game {
                            let sb = game.sb;
                            let bb = game.bb;
                            let gs = game.public_for(you_id);
                            lobby.last_printed_log_len = gs.action_log.len();
                            let header = mcg_server::pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
                            println!("{}", header);
                        }
                    }
                    Err(e) => {
                        let _ = send_server_msg_to_writer(writer, &ServerMsg::Error(format!("Failed to reset game: {}", e))).await;
                    }
                }
            }
            if let Some(gs) = current_state_public(state, you_id).await {
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
        if let Some(gs) = current_state_public(state, you_id).await {
            let _ = send_server_msg_to_writer(writer, &ServerMsg::State(gs)).await;
        }

        if !did_act {
            break;
        }
        // Sleep a pseudo-random-ish delay between actions without holding non-Send state
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
    Ok(())
}