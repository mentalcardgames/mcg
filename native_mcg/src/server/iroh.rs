// Iroh transport listener for MCG server.
//
// This module accepts incoming iroh connections and speaks a simple
// newline-delimited JSON protocol where each JSON object is a
// Frontend2BackendMsg or Backend2FrontendMsg (the same types used over the WebSocket).
//
// The implementation mirrors the WebSocket handler behaviour: clients send a
// `Frontend2BackendMsg::Subscribe` if they wish to receive broadcast state updates. After
// subscribing they are sent the current `Backend2FrontendMsg::State` (if one exists) and
// will receive future broadcasts. All other client messages delegate to shared
// backend handlers to preserve transport-agnostic behavior.
//
// Note: this file is feature-gated behind the iroh Cargo feature. It attempts
// to follow the iroh API shown in the iroh docs. The exact iroh types and
// method names may differ across versions; treat this as the integration
// scaffolding that can be adjusted for the installed iroh crate.

use anyhow::{Context, Result};

use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::sync::broadcast;

use crate::public::{path_for_config, PublicInfo};
use crate::transport::{send_server_msg_to_writer, send_peer_msg_to_writer};
use crate::server::state::{broadcast_state, send_local_frontend, AppState, subscribe_connection, PeerInfo};
use mcg_shared::{Frontend2BackendMsg, Backend2FrontendMsg, Peer2PeerMsg};

/// Public entrypoint spawned by server startup
///
/// Refactored to delegate sub-tasks to smaller helper functions to improve
/// readability and make the high-level flow easier to follow.
pub async fn spawn_iroh_listener(state: AppState) -> Result<()> {
    // Keep the iroh-specific imports local to this function so the module does
    // not require iroh at compile time when the feature is disabled.
    // `getrandom` will be imported in `load_or_generate_iroh_secret` where it's used.
    use iroh::SecretKey;
    use iroh_tickets::{Ticket, endpoint::EndpointTicket};

    // Application ALPN identifier (must match client)
    const ALPN: &[u8] = b"mcg/iroh/1";

    // Obtain or generate the node secret key (may persist to config)
    let secret_key: SecretKey = load_or_generate_iroh_secret(state.clone()).await;

    // Build and bind the iroh endpoint (advertising our ALPN)
    let endpoint = build_iroh_endpoint(secret_key, ALPN).await?;

    // Wait for endpoint to be "online" (connected to relay, has addresses)
    // This is critical for reliable connections on restrictive networks.
    // The online() method waits until we have a home relay connection and at least one address.
    let ep_for_wait = endpoint.clone();
    match tokio::time::timeout(std::time::Duration::from_secs(30), ep_for_wait.online()).await {
        Ok(()) => tracing::info!("iroh endpoint is online (relay connected)"),
        Err(_) => {
            tracing::warn!("timeout waiting for iroh endpoint to come online; proceeding anyway")
        }
    }

    // Print endpoint id for CLI users (renamed from node_id in iroh 0.95)
    let pk = endpoint.id();

    // Nice readable banner for the user
    println!("\n\x1b[1;32m=== Iroh Endpoint Ready ===\x1b[0m");
    println!("\x1b[1mNode ID:\x1b[0m {}", pk);
    println!("\x1b[1;32m===========================\x1b[0m\n");

    //Add ourselves to the peer list w/ empty name, we update it later
    {
        let us = PeerInfo{
            name: "".to_string(),
            ourselves: true,
        };
        state.peers.write().await.insert(pk.clone(), us);
    }

    // Keep structured info for debug mode
    let addr = endpoint.addr();
    let relay_urls: Vec<_> = addr.relay_urls().collect();
    tracing::info!(iroh_node_id = %pk, iroh_addr = ?addr, relay_urls = ?relay_urls);

    //Use the addr to make a ticket to use for generating a QR code later
    let ticket = EndpointTicket::new(addr);
    println!("{ticket}");
    tracing::info!(ticket = %ticket);
    let ticket_str = ticket.serialize();
    {
        let mut guard = state.ticket.write().await;
        *guard = Some(ticket_str);
    }

    let public_path = path_for_config(state.config_path.as_deref());
    match PublicInfo::write_iroh_node_id(&public_path, pk.to_string()) {
        Ok(_) => tracing::info!(path = %public_path.display(), "stored iroh node id"),
        Err(e) => {
            tracing::warn!(error = %e, path = %public_path.display(), "failed to persist iroh node id")
        }
    }

    // Start the accept loop which will spawn a handler per connection
    start_iroh_accept_loop(endpoint.clone(), state.clone());
    start_iroh_connect_loop(endpoint.clone(), state.clone());

    tracing::info!(alpn = %std::str::from_utf8(ALPN).unwrap_or("mcg/iroh/1"), "iroh listener started");
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
            tracing::error!(error = %e, "failed to get randomness for iroh key");
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
                    SecretKey::from_bytes(&arr)
                } else {
                    // Invalid length in-memory config; fall through to generate-and-save below.
                    drop(cfg_r);
                    let sk = generate_new_key();
                    let mut cfg_w = state.config.write().await;
                    if let Err(e) = cfg_w.set_iroh_key_bytes_and_save(&cfg_path, &sk.to_bytes()) {
                        tracing::error!(error = %e, "Failed to save generated iroh key to config '{}'", cfg_path.display());
                    } else {
                        tracing::info!(config_path = %cfg_path.display(), "Saved generated iroh key into config");
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
                    if let Err(e) = cfg_w.set_iroh_key_bytes_and_save(&cfg_path, &sk.to_bytes()) {
                        tracing::error!(error = %e, "Failed to save generated iroh key to config '{}'", cfg_path.display());
                    } else {
                        tracing::info!(config_path = %cfg_path.display(), "Saved generated iroh key into config");
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
        tracing::warn!(
            "no server config path provided; generating ephemeral iroh key (not persisted)"
        );
        generate_new_key()
    }
}

/// Build and bind an iroh Endpoint advertising our ALPN.
async fn build_iroh_endpoint(
    secret_key: iroh::SecretKey,
    alpn: &[u8],
) -> Result<iroh::endpoint::Endpoint> {
    use iroh::endpoint::Endpoint;

    // Endpoint::builder() uses presets::N0 which includes:
    // - DNS discovery via iroh.link
    // - Default n0 relay servers (RelayMode::Default)
    let endpoint = Endpoint::builder()
        .alpns(vec![alpn.to_vec()])
        .secret_key(secret_key)
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
                        let remote_node_id = conn.remote_id();
                        tracing::info!(peer = %remote_node_id, "Accepted new iroh connection");
                        let state_for_conn = state_clone.clone();
                        tokio::spawn(async move {
                            if let Err(e) = manage_incoming_iroh_connection(state_for_conn, conn).await {
                                tracing::error!(error = %e, "iroh connection handler error");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "iroh accept/connect error");
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

fn start_iroh_connect_loop(endpoint: iroh::endpoint::Endpoint, state: AppState){
    let ep_connect = endpoint;
    let state_clone = state.clone();
    const ALPN: &[u8] = b"mcg/iroh/1";
    use iroh_tickets::{Ticket, endpoint::EndpointTicket};

    tokio::spawn(async move {
        let mut last_seen: Option<String> = None;
        loop {
            let current = {
                state_clone.remote_ticket.read().await.clone()
            };
            if let Some(ticket_str) = current{
                if last_seen.as_ref() != Some(&ticket_str){
                    let ticket = EndpointTicket::deserialize(ticket_str.as_str());
                    match ticket {
                        Ok(t) => {
                            let addr = t.endpoint_addr().clone();
                            let conn = ep_connect.connect(addr, ALPN).await;
                            match conn {
                                Ok(c) => {
                                    tracing::info!(peer = %c.remote_id(), "Successfully connected");
                                    let state_for_conn = state_clone.clone();
                                    let ticket_str_clone = ticket_str.clone();
                                    tokio::spawn(async move {
                                        if let Err(e) = manage_outgoing_iroh_connection(state_for_conn, c).await {
                                            tracing::error!(error = %e, ticket_str = %ticket_str_clone, "iroh connection handler error");
                                        }
                                    });
                                }
                                Err(e) => {
                                    tracing::error!(error = %e, ticket_str = %ticket_str, "Failed to connect to peer");
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, ticket_str = %ticket_str, "Failed to deserialize iroh ticket from remote_ticket");
                        }
                    }

                    last_seen = Some(ticket_str.clone());
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
}
// Per-connection handler which speaks newline-delimited JSON over a
// bi-directional iroh connection. Separated into smaller helpers to make
// the flow easier to reason about and unit-test individual parts.
async fn manage_incoming_iroh_connection(
    state: AppState,
    connection: iroh::endpoint::Connection,
) -> Result<()> {
    // Accept a bidirectional stream (send, recv) and wrap recv in a BufReader.
    let (mut send, recv) = connection.accept_bi().await?;
    let mut reader = BufReader::new(recv);

    tracing::info!(peer = %connection.remote_id(), "Iroh bi-stream established");
    let peer_id = connection.remote_id();
    let mut subscription: Option<broadcast::Receiver<Backend2FrontendMsg>> = None;

    let mut line = String::new();
    loop {
        line.clear();
        if let Some(rx) = subscription.as_mut() {
            tokio::select! {
                recv = rx.recv() => {
                    match recv {
                        Ok(sm) => {
                            if let Err(e) = send_server_msg_to_writer(&mut send, &sm).await {
                                tracing::error!(error = %e, "iroh send error while forwarding broadcast");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                res = reader.read_line(&mut line) => {
                    match res {
                        Ok(0) => break,
                        Ok(_) => {
                            if !process_iroh_line(&state, &mut send, &mut subscription, line.trim(), peer_id.clone()).await? {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "iroh read error");
                            break;
                        }
                    }
                }
            }
        } else {
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if !process_iroh_line(&state, &mut send, &mut subscription, line.trim(),peer_id.clone()).await?
                    {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "iroh read error");
                    break;
                }
            }
        }
    }

    tracing::info!("[IROH DISCONNECT] Client");
    // Close the send side politely if available
    let _ = send.finish();
    // Remove the peer in case they couldn't send a disconnect message or we hit an error
    {
        if let Some(peer) = state.peers.read().await.get(&peer_id) {
            if !peer.ourselves {
                let _ = state.broadcaster.send(
                    Backend2FrontendMsg::RemovePlayer(peer.name.clone())
                );
            }
        }
    }
    {
        state.peers.write().await.remove(&peer_id);
    }
    connection.closed().await;
    Ok(())
}

//Split the original manage_iroh_connection into two seperate functions (logic is the same)
// The reason for this is so that we can set up sending specific messages easier
async fn manage_outgoing_iroh_connection(
    state: AppState,
    connection: iroh::endpoint::Connection,
) -> Result<()> {
    // Accept a bidirectional stream (send, recv) and wrap recv in a BufReader.
    let (mut send, recv) = connection.open_bi().await?;
    let mut reader = BufReader::new(recv);

    tracing::info!(peer = %connection.remote_id(), "Iroh bi-stream established");
    let peer_id = connection.remote_id();

    let lobby = state.lobby.read().await;
    let name = lobby.our_name.clone();
    let msg = Peer2PeerMsg::Connect(name);

    if let Err(e) = send_peer_msg_to_writer(
        &mut send,
        &msg,
    ).await{
        tracing::error!(error = %e, "iroh send error while sending Connect message");
    }
    tracing::info!("Sent connect message to peer");

    let mut subscription: Option<broadcast::Receiver<Backend2FrontendMsg>> = None;

    let mut line = String::new();
    loop {
        line.clear();
        if let Some(rx) = subscription.as_mut() {
            tokio::select! {
                recv = rx.recv() => {
                    match recv {
                        Ok(sm) => {
                            if let Err(e) = send_server_msg_to_writer(&mut send, &sm).await {
                                tracing::error!(error = %e, "iroh send error while forwarding broadcast");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                res = reader.read_line(&mut line) => {
                    match res {
                        Ok(0) => break,
                        Ok(_) => {
                            if !process_iroh_line(&state, &mut send, &mut subscription, line.trim(), peer_id.clone()).await? {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "iroh read error");
                            break;
                        }
                    }
                }
            }
        } else {
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if !process_iroh_line(&state, &mut send, &mut subscription, line.trim(),peer_id.clone()).await?
                    {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "iroh read error");
                    break;
                }
            }
        }
    }

    tracing::info!("[IROH DISCONNECT] Client");
    // Remove the peer from our local list, just in case (not even sure if this is relevant like it is for
    // the accept side, but since i dont super get the architecture, better safe than sorry)
    {
        state.peers.write().await.remove(&peer_id);
    }
    // Try to notify the peer we are disconnecting
    let our_name = state.lobby.read().await.our_name.clone();
    if let Err(e) = send_peer_msg_to_writer(&mut send, &Peer2PeerMsg::Disconnect(our_name)).await {
        tracing::warn!(error = %e, "failed to send peer Disconnect on incoming connection drop");
    }
    // Close the send side politely if available
    let _ = send.finish();
    connection.closed().await;
    Ok(())
}

async fn process_iroh_line<W>(
    state: &AppState,
    send: &mut W,
    subscription: &mut Option<broadcast::Receiver<Backend2FrontendMsg>>,
    trimmed: &str,
    peer_id: iroh::EndpointId,
) -> Result<bool>
where
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    if trimmed.is_empty() {
        return Ok(true);
    }

    if let Ok(_peer_msg) = serde_json::from_str::<Peer2PeerMsg>(trimmed) {
        return process_iroh_peer_line(state, send, subscription, trimmed, peer_id.clone()).await;
    }

    match serde_json::from_str::<Frontend2BackendMsg>(trimmed) {
        Ok(Frontend2BackendMsg::Subscribe) => {
            if subscription.is_some() {
                let _ =
                    send_server_msg_to_writer(send, &Backend2FrontendMsg::Error("already subscribed".into()))
                        .await;
                return Ok(true);
            }
            let sub = subscribe_connection(state).await;
            if let Some(gs) = sub.initial_state {
                send_server_msg_to_writer(send, &Backend2FrontendMsg::State(gs)).await?;
            }
            *subscription = Some(sub.receiver);
            Ok(true)
        }
        Ok(other) => {
            tracing::debug!(client_msg = ?other, "iroh received client message");
            let resp = crate::server::dispatch_client_message(state, other).await;
            if let Err(e) = send_server_msg_to_writer(send, &resp).await {
                tracing::error!(error = %e, "iroh send error while forwarding response");
                return Err(e);
            }
            Ok(true)
        }
        Err(e) => {
            let msg = Backend2FrontendMsg::Error(format!("Invalid JSON message: {}", e));
            let _ = send_server_msg_to_writer(send, &msg).await;
            Ok(true)
        }
    }
}

///Peer Message equivalent of process_iroh_line, not using the same dispatch_client_message 
///function setup since it would cause an infinite send-receive loop of messages between peers
async fn process_iroh_peer_line<W>(
    state: &AppState,
    send: &mut W,
    subscription: &mut Option<broadcast::Receiver<Backend2FrontendMsg>>,
    trimmed: &str,
    peer_id: iroh::EndpointId,
) -> Result<bool>
where
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    if trimmed.is_empty() {
        return Ok(true);
    }

    match serde_json::from_str::<Peer2PeerMsg>(trimmed){
        Ok(Peer2PeerMsg::Connect(name)) => {
            tracing::info!(peer = %peer_id, "Peer requested connect with name '{}'", name);
            let mut name_clone = name.clone();
            // Check if the lobby is open and has room for more players before accepting the connection
            let (should_reject, lobby_open, current_players, max_players) = {
                let lobby = state.lobby.read().await;
                let peers = state.peers.read().await;
                (
                    !lobby.lobby_open || peers.len() >= lobby.max_players,
                    lobby.lobby_open,
                    peers.len(),
                    lobby.max_players,
                )
            };
            // If we should reject the connection, send a Reject message and return false to disconnect
            if should_reject {
                let msg = Peer2PeerMsg::Reject(
                    if !lobby_open {
                        "Lobby is closed".into()
                    } else {
                        "Lobby is full".into()
                    },
                );

                send_peer_msg_to_writer(send, &msg).await?;
                return Ok(false);
            }
            {
                // Rename the player in case we have someone of that name already
                let mut name_exists = false;
                let peers = state.peers.read().await;
                for peer in peers.values() {
                    if peer.name == name {
                        name_exists = true;
                        break;
                    }
                }
                if name_exists {
                    let mut counter = 2;
                    let mut new_name = format!("{} {}", name, counter);

                    // Keep incrementing the counter until we find a name that isn't taken
                    while peers.values().any(|peer| peer.name == new_name) {
                        counter += 1;
                        new_name = format!("{} {}", name, counter);
                    }
                    tracing::info!(peer = %peer_id, "Peer name '{}' already exists, renaming to '{}'", name, new_name);
                    name_clone = new_name;
                }
            }
            // If we had to rename the player, send them a NewName message so they know their assigned name
            if name_clone != name {
                let msg = Peer2PeerMsg::NewName(name_clone.clone());
                send_peer_msg_to_writer(send, &msg).await?;
            }
            {
                // Tell the new peer about all the existing peers so they can populate their peer list
                let peers_snapshot = state.peers.read().await.clone();
                let msg = Peer2PeerMsg::Peers(
                    peers_snapshot.into_iter().map(|(id, info)| (id.to_string(), info.name)).collect()
                );
                send_peer_msg_to_writer(send, &msg).await?;
            }
            // Add the new player to our list of connected peers
            let peer = PeerInfo{
                name: name_clone.clone(),
                ourselves: false,
            };
            state.peers.write().await.insert(peer_id.clone(), peer);
            // Output how many peers are currently connected for debug purposes
            tracing::info!("Now at {}/{} players", current_players, max_players);
            // Subscribe to state updates and broadcast the new player to the frontend
            let sub = subscribe_connection(state).await;
            *subscription = Some(sub.receiver);
            let _ = state.broadcaster.send(
                Backend2FrontendMsg::NewPlayer(name_clone.clone())
            );
            broadcast_state(&state).await;
            return Ok(true);
        }
        Ok(Peer2PeerMsg::Disconnect(name)) => {
            tracing::info!(peer = %peer_id, "Peer requested disconnect");
            {
                state.peers.write().await.remove(&peer_id);
            }
            let _ = state.broadcaster.send(
                Backend2FrontendMsg::RemovePlayer(name)
            );
            return Ok(false);

        }
        Ok(Peer2PeerMsg::Peers(peers)) => {
            tracing::info!(peer = %peer_id, "Received peer list from new connection: {:?}", peers);
            // Add all the peers (that aren't already in our list) to our peer list.
            let mut map = state.peers.write().await;

            for peer in peers.into_iter() {
                let Ok(new_id) = peer.0.parse() else {
                    tracing::warn!("Invalid peer id received: {}", peer.0);
                    continue;
                };

                if !map.contains_key(&new_id) {
                    let peer_info = PeerInfo {
                        name: peer.1,
                        ourselves: false,
                    };
                    map.insert(new_id, peer_info);
                }
            }
            return Ok(true);
        }
        Ok(Peer2PeerMsg::NewName(name)) => {
            // If we receive a new name, we set it
            state.lobby.write().await.our_name = name.clone();
            tracing::info!(peer = %peer_id, "Peer informed us of our assigned name: '{}'", name);
            // ... and also edit us in our peer list
            {
                let mut peers = state.peers.write().await;
                for peer in peers.iter_mut() {
                    if peer.1.ourselves {
                        peer.1.name = name.clone();
                        break;
                    }
                }
            }
            // Send the new name to the local frontend ONLY, so it can update just for us
            let msg = Backend2FrontendMsg::OurName(name.clone());
            send_local_frontend(&state, msg).await;
            return Ok(true);
        }
        Ok(Peer2PeerMsg::Reject(reason)) => {
            tracing::warn!(reason = %reason, "peer rejected our connection");

            let _ = state.broadcaster.send(
                Backend2FrontendMsg::Error(format!("Peer rejected connection: {}", reason))
            );

            {
            let mut guard = state.remote_ticket.write().await;
                *guard = None;
            }
            // Return false to break the loop and disconnect
            return Ok(false);
        }
        Ok(other) => {
            tracing::debug!(peer_msg = ?other, "iroh received peer message");
            // No dispatch for the other peer messages yet; just log them.
            return Ok(true);
        }
        Err(e) => {
            let msg = Backend2FrontendMsg::Error(format!("Invalid JSON message: {}", e));
            let _ = send_server_msg_to_writer(send, &msg).await;
            return Ok(true);
        }
    }
}