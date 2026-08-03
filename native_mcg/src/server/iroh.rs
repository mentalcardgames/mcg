// Iroh transport bootstrap for the MCG server.
//
// This module owns the endpoint and hands incoming peer streams to the network
// supervisor. Outgoing connections are initiated through `NetworkHandle` by
// network-aware application handlers.

use anyhow::{Context, Result};

use crate::network::{NetworkHandle, PeerId};
use crate::public::{path_for_config, PublicInfo};
use crate::server::state::{AppState, PeerInfo};

/// Creates the Iroh endpoint and starts accepting peer connections.
pub async fn spawn_iroh_listener(state: AppState, network: NetworkHandle) -> Result<()> {
    use iroh::SecretKey;
    use iroh_tickets::{endpoint::EndpointTicket, Ticket};

    const ALPN: &[u8] = b"mcg/iroh/1";

    let secret_key: SecretKey = load_or_generate_iroh_secret(state.clone()).await;
    let endpoint = build_iroh_endpoint(secret_key, ALPN).await?;
    network
        .configure_iroh_endpoint(endpoint.clone())
        .await
        .context("configuring Iroh endpoint in network supervisor")?;

    match tokio::time::timeout(std::time::Duration::from_secs(30), endpoint.online()).await {
        Ok(()) => tracing::info!("iroh endpoint is online (relay connected)"),
        Err(_) => {
            tracing::warn!("timeout waiting for iroh endpoint to come online; proceeding anyway")
        }
    }

    let endpoint_id = endpoint.id();
    println!("\n\x1b[1;32m=== Iroh Endpoint Ready ===\x1b[0m");
    println!("\x1b[1mNode ID:\x1b[0m {endpoint_id}");
    println!("\x1b[1;32m===========================\x1b[0m\n");

    let addr = endpoint.addr();
    let relay_urls: Vec<_> = addr.relay_urls().collect();
    tracing::info!(iroh_node_id = %endpoint_id, iroh_addr = ?addr, relay_urls = ?relay_urls);

    let ticket = EndpointTicket::new(addr);
    println!("{ticket}");
    tracing::info!(ticket = %ticket);
    let ticket = ticket.serialize();
    *state.ticket.write().await = Some(ticket.clone());

    state.peers.write().await.insert(
        endpoint_id,
        PeerInfo {
            name: String::new(),
            ticket,
        },
    );

    let public_path = path_for_config(state.config_path.as_deref());
    match PublicInfo::write_iroh_node_id(&public_path, endpoint_id.to_string()) {
        Ok(_) => tracing::info!(path = %public_path.display(), "stored iroh node id"),
        Err(error) => {
            tracing::warn!(%error, path = %public_path.display(), "failed to persist iroh node id")
        }
    }

    start_iroh_accept_loop(endpoint, network);

    tracing::info!(
        alpn = %std::str::from_utf8(ALPN).unwrap_or("mcg/iroh/1"),
        "iroh listener started"
    );
    Ok(())
}

async fn load_or_generate_iroh_secret(state: AppState) -> iroh::SecretKey {
    use getrandom::getrandom;
    use iroh::SecretKey;

    let generate_new_key = || -> SecretKey {
        let mut bytes = [0u8; 32];
        if let Err(error) = getrandom(&mut bytes) {
            tracing::error!(%error, "failed to get randomness for iroh key");
        }
        SecretKey::from_bytes(&bytes)
    };

    if let Some(config_path) = state.config_path.clone() {
        let config = state.config.read().await;
        if let Some(bytes) = config.iroh_key_bytes() {
            if bytes.len() >= 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[..32]);
                return SecretKey::from_bytes(&key);
            }
        }
        drop(config);

        let secret_key = generate_new_key();
        let mut config = state.config.write().await;
        if let Some(bytes) = config.iroh_key_bytes() {
            if bytes.len() >= 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[..32]);
                return SecretKey::from_bytes(&key);
            }
        }

        if let Err(error) = config.set_iroh_key_bytes_and_save(&config_path, &secret_key.to_bytes())
        {
            tracing::error!(%error, "failed to save generated Iroh key to config '{}'", config_path.display());
        } else {
            tracing::info!(config_path = %config_path.display(), "saved generated Iroh key into config");
        }
        secret_key
    } else {
        tracing::warn!(
            "no server config path provided; generating ephemeral iroh key (not persisted)"
        );
        generate_new_key()
    }
}

async fn build_iroh_endpoint(
    secret_key: iroh::SecretKey,
    alpn: &[u8],
) -> Result<iroh::endpoint::Endpoint> {
    use iroh::endpoint::Endpoint;

    Endpoint::builder()
        .alpns(vec![alpn.to_vec()])
        .secret_key(secret_key)
        .bind()
        .await
        .context("binding iroh endpoint")
}

fn start_iroh_accept_loop(endpoint: iroh::endpoint::Endpoint, network: NetworkHandle) {
    tokio::spawn(async move {
        loop {
            match endpoint.accept().await {
                Some(connect_future) => match connect_future.await {
                    Ok(connection) => {
                        let remote_id = connection.remote_id();
                        tracing::info!(peer = %remote_id, "accepted new Iroh connection");
                        let network = network.clone();
                        tokio::spawn(async move {
                            if let Err(error) =
                                register_incoming_iroh_connection(network, connection).await
                            {
                                tracing::error!(peer = %remote_id, %error, "failed to register incoming Iroh connection");
                            }
                        });
                    }
                    Err(error) => {
                        tracing::error!(%error, "Iroh accept/connect error");
                    }
                },
                None => {
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
