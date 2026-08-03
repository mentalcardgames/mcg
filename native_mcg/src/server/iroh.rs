// Iroh transport listener for MCG server.
//
// This module bootstraps the iroh endpoint. Incoming peer streams are handed
// to the network supervisor, which owns their protocol processing and lifecycle.
//
// The outgoing connection path below still uses the legacy handler and is
// migrated separately from incoming connections.
//
// Note: this file is feature-gated behind the iroh Cargo feature. It attempts
// to follow the iroh API shown in the iroh docs. The exact iroh types and
// method names may differ across versions; treat this as the integration
// scaffolding that can be adjusted for the installed iroh crate.

use anyhow::{Context, Result};

use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::sync::broadcast;

use crate::network::{NetworkHandle, PeerId};
use crate::public::{path_for_config, PublicInfo};
use crate::transport::{send_server_msg_to_writer, send_peer_msg_to_writer};
use crate::server::state::{AppState, subscribe_connection, PeerInfo};
use mcg_shared::{Frontend2BackendMsg, Backend2FrontendMsg, Peer2PeerMsg};

/// Public entrypoint spawned by server startup
///
/// Refactored to delegate sub-tasks to smaller helper functions to improve
/// readability and make the high-level flow easier to follow.
pub async fn spawn_iroh_listener(state: AppState, network: NetworkHandle) -> Result<()> {
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
    network
        .configure_iroh_endpoint(endpoint.clone())
        .await
        .context("configuring Iroh endpoint in network supervisor")?;

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
        *guard = Some(ticket_str.clone());
    }

        //Add ourselves to the peer list w/ empty name, we update it later
    {
        let us = PeerInfo{
            name: "".to_string(),
            ticket: ticket_str.clone(),
        };
        state.peers.write().await.insert(pk.clone(), us);
    }

    let public_path = path_for_config(state.config_path.as_deref());
    match PublicInfo::write_iroh_node_id(&public_path, pk.to_string()) {
        Ok(_) => tracing::info!(path = %public_path.display(), "stored iroh node id"),
        Err(e) => {
            tracing::warn!(error = %e, path = %public_path.display(), "failed to persist iroh node id")
        }
    }

    // Start the accept loop which registers each peer stream with the network supervisor.
    start_iroh_accept_loop(endpoint.clone(), network);
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

/// Spawn the accept loop which accepts connections and registers their first
/// bidirectional stream with the network supervisor.
fn start_iroh_accept_loop(endpoint: iroh::endpoint::Endpoint, network: NetworkHandle) {
    let ep_accept = endpoint;
    tokio::spawn(async move {
        loop {
            match ep_accept.accept().await {
                Some(connect_future) => match connect_future.await {
                    Ok(conn) => {
                        let remote_node_id = conn.remote_id();
                        tracing::info!(peer = %remote_node_id, "Accepted new iroh connection");
                        let network = network.clone();
                        tokio::spawn(async move {
                            if let Err(e) = register_incoming_iroh_connection(network, conn).await {
                                tracing::error!(peer = %remote_node_id, error = %e, "failed to register incoming iroh connection");
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

async fn register_incoming_iroh_connection(
    network: NetworkHandle,
    connection: iroh::endpoint::Connection,
) -> Result<()> {
    let peer_id = PeerId::new(connection.remote_id().to_string());
    let (writer, reader) = connection
        .accept_bi()
        .await
        .context("accepting incoming Iroh bidirectional stream")?;
    let connection_id = network
        .register_iroh_peer(peer_id.clone(), reader, writer)
        .await
        .context("registering incoming Iroh peer with network supervisor")?;

    tracing::info!(%connection_id, %peer_id, "incoming Iroh peer registered");
    Ok(())
}

fn start_iroh_connect_loop(endpoint: iroh::endpoint::Endpoint, state: AppState){
    let ep_connect = endpoint;
    let state_clone = state.clone();
    const ALPN: &[u8] = b"mcg/iroh/1";
    use iroh_tickets::{Ticket, endpoint::EndpointTicket};

    tokio::spawn(async move {
        loop {
            // Consume the remote_ticket immediately if present.
            // We take a write lock and remove the ticket (so we won't re-process it).
            let ticket_opt = {
                let mut guard = state_clone.remote_ticket.write().await;
                guard.take()
            };

            if let Some(ticket_str) = ticket_opt {
                match EndpointTicket::deserialize(ticket_str.as_str()) {
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
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
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

    let name = {
        let lobby = state.lobby.read().await;
        lobby.our_name.clone()
    };
    let ticket = {
        let guard = state.ticket.read().await;
        guard.clone()
    };
    let msg = Peer2PeerMsg::Connect(name, ticket);

    if let Err(e) = send_peer_msg_to_writer(
        &mut send,
        &msg,
    ).await{
        tracing::error!(error = %e, "iroh send error while sending Connect message");
    }
    tracing::info!("Sent connect message to peer");

    let mut subscription: Option<broadcast::Receiver<Backend2FrontendMsg>> = None;
    // Receiver for peer broadcasts
    let mut peer_rx = state.peer_broadcaster.subscribe();

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
                peer = peer_rx.recv() => {
                    match peer {
                        Ok(pm) => {
                            if let Err(e) = send_peer_msg_to_writer(&mut send, &pm).await {
                                tracing::error!(error = %e, "iroh send error while forwarding peer broadcast");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
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
            // subscription == None: still poll peer_rx so peer messages are forwarded
            tokio::select! {
                peer = peer_rx.recv() => {
                    match peer {
                        Ok(pm) => {
                            if let Err(e) = send_peer_msg_to_writer(&mut send, &pm).await {
                                tracing::error!(error = %e, "iroh send error while forwarding peer broadcast");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                res = reader.read_line(&mut line) => {
                    match res {
                        Ok(0) => break,
                        Ok(_) => {
                            if !process_iroh_line(&state, &mut send, &mut subscription, line.trim(),peer_id.clone()).await? {
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
        }
    }

    tracing::info!("[IROH DISCONNECT] Client");
    // Remove the peer from our local list, just in case (not even sure if this is relevant like it is for
    // the accept side, but since i dont super get the architecture, better safe than sorry)
    {
        let name_to_remove = {
            let peers = state.peers.read().await;
            peers.get(&peer_id)
                .map(|p| p.name.clone())
        };

        if let Some(name) = name_to_remove {
            let _ = state.broadcaster.send(
                Backend2FrontendMsg::RemovePlayer(name)
            );
        }
    }
    {
        state.peers.write().await.remove(&peer_id);
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
        return process_iroh_peer_line(state, send, trimmed, peer_id.clone()).await;
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
                send_server_msg_to_writer(send, &Backend2FrontendMsg::UpdatePokerState(gs)).await?;
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
        Ok(Peer2PeerMsg::Connect(name, ticket)) => {
            tracing::info!(peer = %peer_id, "Peer requested connect with name '{}'", name);
            let mut name_clone = name.clone();
            let mut new_name = None;
            // Check if the lobby is open and has room for more players before accepting the connection
            let (should_reject, lobby_open, _current_players, max_players, game_running) = {
                let lobby = state.lobby.read().await;
                let peers = state.peers.read().await;
                (
                    !lobby.lobby_open || peers.len() >= lobby.max_players || lobby.game_running,
                    lobby.lobby_open,
                    peers.len(),
                    lobby.max_players,
                    lobby.game_running,
                )
            };
            // If we should reject the connection, send a Reject message and return false to disconnect
            if should_reject {
                let msg = Peer2PeerMsg::Reject(
                    if !lobby_open {
                        "Lobby is closed".into()
                    } else if game_running {
                        "Game is already running, wait until it finishes and try again".into()
                    } else {
                        "Lobby is full".into()
                    },
                );

                send_peer_msg_to_writer(send, &msg).await?;
                return Ok(false);
            }
            // Tell the peer they are now in the lobby, and what the max player count and game type is
            {
                let lobby = state.lobby.read().await;
                let msg = Peer2PeerMsg::LobbyAccept(max_players, lobby.game_type.clone());
                send_peer_msg_to_writer(send, &msg).await?;
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
                    let mut candidate = format!("{} {}", name, counter);

                    // Keep incrementing the counter until we find a name that isn't taken
                    while peers.values().any(|peer| peer.name == candidate) {
                        counter += 1;
                        candidate = format!("{} {}", name, counter);
                    }
                    new_name = Some(candidate);
                }
            }
            if let Some(n) = new_name {
                // Inform the peer of their new name if we had to change it
                tracing::info!(peer = %peer_id, "Name '{}' already exists, renaming to '{}'", name, n);
                name_clone = n.clone();
                let msg = Peer2PeerMsg::NewName(n);
                send_peer_msg_to_writer(send, &msg).await?;
            }
            {
                // Tell the new peer about all the existing peers so they can populate their peer list
                let peers_snapshot = state.peers.read().await.clone();
                let msg = Peer2PeerMsg::Peers(
                    peers_snapshot.into_iter().map(|(id, info)| (id.to_string(), (info.name, info.ticket))).collect()
                );
                send_peer_msg_to_writer(send, &msg).await?;
            }
            // Add the new player to our list of connected peers
            let peer = PeerInfo{
                name: name_clone.clone(),
                ticket: ticket.clone().unwrap_or_default(),
            };
            state.peers.write().await.insert(peer_id.clone(), peer);
            // Output how many peers are currently connected for debug purposes
            let current_players = state.peers.read().await.len();
            tracing::info!("Now at {}/{} players", current_players, max_players);
            // Broadcast the new player to our frontend
            let _ = state.broadcaster.send(
                Backend2FrontendMsg::NewPlayer(name_clone.clone())
            );
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
            tracing::info!(peer = %peer_id, "Received peer list from new connection");
            // Add all the peers (that aren't already in our list) to our peer list.
            let mut map = state.peers.write().await;

            for peer in peers.into_iter() {
                let Ok(new_id) = peer.0.parse() else {
                    tracing::warn!("Invalid peer id received: {}", peer.0);
                    continue;
                };

                if !map.contains_key(&new_id) {
                    let peer_info = PeerInfo {
                        name: peer.1.0,
                        ticket: peer.1.1,
                    };
                    map.insert(new_id, peer_info.clone());

                    // Broadcast the new peer to our frontend so it can update its peer list
                    let name = peer_info.name.clone();
                    let ticket = peer_info.ticket.clone();
                    tracing::info!(peer_id = %peer.0, peer_name = %peer_info.name, "Added peer from received peer list");
                    let _ = state.broadcaster.send(
                        Backend2FrontendMsg::NewPlayer(name)
                    );

                    // For all the peers in the list that aren't the one that just sent us the list,
                    // update our remote_ticket so that we can attempt to connect to them if we aren't already connected
                    if new_id != peer_id {
                        state.remote_ticket.write().await.replace(ticket);
                    }
                }
            }
            tracing::info!("Peer list updated with new connections");
            return Ok(true);
        }
        Ok(Peer2PeerMsg::NewName(name)) => {
            // If we receive a new name, we set it
            {
                state.lobby.write().await.our_name = name.clone();
            }
            tracing::info!(peer = %peer_id, "Peer informed us of our assigned name: '{}'", name);
            // ... and also edit us in our peer list
            {
                let mut peers = state.peers.write().await;
                for peer in peers.iter_mut() {
                    if peer.1.ticket == state.ticket.read().await.clone().unwrap_or_default() {
                        peer.1.name = name.clone();
                        break;
                    }
                }
            }
            let _ = state.broadcaster.send(
                Backend2FrontendMsg::OurName(name)
            );
            return Ok(true);
        }
        Ok(Peer2PeerMsg::LobbyAccept(max_players, game_type)) => {
            tracing::info!(peer = %peer_id, "Peer accepted our connection; lobby max players: {}, game type: {}", max_players, game_type);
            let mut lobby = state.lobby.write().await;
            lobby.lobby_open = true;
            lobby.max_players = max_players;
            lobby.game_type = game_type;
            //Use pong as a generic message to switch screens on the frontend without needing to make a new message type just for that
            let msg = Backend2FrontendMsg::Pong;
            let _ = state.broadcaster.send(msg);
            return Ok(true);
        }
        Ok(Peer2PeerMsg::RequestReady) => {
            tracing::info!(peer = %peer_id, "Peer requested ready status");
            let msg = {
                let lobby = state.lobby.read().await;
                Peer2PeerMsg::PeerReady(lobby.our_name.clone(), lobby.ready)
            };
            send_peer_msg_to_writer(send, &msg).await?;
            return Ok(true);
        }
        Ok(Peer2PeerMsg::PeerReady(name, ready) ) => {
            tracing::info!(peer = %peer_id, "Peer '{}' is now {}", name, if ready { "ready" } else { "not ready" });
            let _ = state.broadcaster.send(
                Backend2FrontendMsg::PlayerReady(name, ready)
            );
            return Ok(true);
        }
        Ok(Peer2PeerMsg::Reject(reason)) => {
            tracing::warn!(reason = %reason, "peer rejected our connection");

            let _ = state.broadcaster.send(
                Backend2FrontendMsg::Error(format!("Peer rejected connection: {}", reason))
            );

            // Return false to break the loop and disconnect
            return Ok(false);
        }
        Ok(other) => {
            tracing::debug!(peer_msg = ?other, "iroh received peer message");
            // No dispatch for the other peer messages yet; just log them.
            return Ok(true);
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to deserialize peer message");
            return Ok(true);
        }
    }
}
