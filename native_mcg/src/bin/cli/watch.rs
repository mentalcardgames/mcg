use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use mcg_shared::{ClientMsg, ServerMsg};

use super::utils::handle_server_msg;

/// Watch over a websocket connection and print events as they arrive.
/// Accepts an address string (e.g. "ws://host:port/ws" or "http://host:port") and builds the ws URL internally.
pub async fn watch_ws(ws_addr: &str, json: bool) -> anyhow::Result<()> {
    let ws_url = super::transport::build_ws_url(ws_addr)?;
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(ws_url.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();

    let subscribe_txt = serde_json::to_string(&ClientMsg::Subscribe)?;
    write.send(Message::Text(subscribe_txt)).await?;

    // Read messages forever (until socket closed or error) and handle them via
    // the shared handler. Track how many log entries we've printed so we only
    // print incremental events.
    let mut last_printed: usize = 0;
    loop {
        match read.next().await {
            Some(Ok(Message::Text(txt))) => {
                if let Ok(sm) = serde_json::from_str::<ServerMsg>(&txt) {
                    handle_server_msg(&sm, json, &mut last_printed);
                }
            }
            Some(Ok(_other)) => { /* ignore non-text frames */ }
            Some(Err(e)) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            None => break, // closed
        }
    }

    Ok(())
}

/// Implement a basic long-polling watcher over the HTTP API.
pub async fn watch_http(base: &str, json: bool) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let mut last_printed: usize = 0;
    loop {
        // Long-poll GET state with a 30s timeout
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.get(format!("{}/api/state", base)).send(),
        )
        .await
        {
            Ok(Ok(resp)) => {
                if let Ok(sm) = resp.json::<ServerMsg>().await {
                    handle_server_msg(&sm, json, &mut last_printed);
                }
            }
            Ok(Err(e)) => {
                eprintln!("HTTP error: {}", e);
                break;
            }
            Err(_) => {
                // timeout, continue polling
                continue;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    Ok(())
}

/// Watch over an iroh bidirectional stream and print events as they arrive.
pub async fn watch_iroh(peer_uri: &str, json: bool) -> anyhow::Result<()> {
    // Import iroh APIs inside the function to limit compile-time exposure.
    use iroh::endpoint::Endpoint;
    use tokio::io::{AsyncBufReadExt, BufReader};

    // ALPN must match the server's ALPN
    const ALPN: &[u8] = b"mcg/iroh/1";

    // Bind a local endpoint (discovery_n0 for default settings)
    let endpoint = Endpoint::builder()
        .discovery_n0()
        .bind()
        .await
        .context("binding iroh endpoint for client")?;

    use iroh::PublicKey;
    use std::str::FromStr;
    let pk = PublicKey::from_str(peer_uri).context("parsing iroh public key (z-base-32)")?;
    let connection = endpoint
        .connect(pk, ALPN)
        .await
        .context("connecting to iroh peer (public key)")?;

    // Open a bidirectional stream
    let (mut send, recv) = connection
        .open_bi()
        .await
        .context("opening bidirectional stream")?;

    // Subscribe to broadcast updates
    {
        use tokio::io::AsyncWriteExt;
        let txt = serde_json::to_string(&ClientMsg::Subscribe)?;
        send.write_all(txt.as_bytes()).await?;
        send.write_all(b"\n").await?;
        send.flush().await?;
    }

    let mut reader = BufReader::new(recv);

    // Read newline-delimited JSON messages and handle them via shared handler.
    let mut line = String::new();
    // Track how many log entries we've already printed for this connection.
    let mut last_printed: usize = 0;
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // connection closed
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(sm) = serde_json::from_str::<ServerMsg>(trimmed) {
                    handle_server_msg(&sm, json, &mut last_printed);
                } else {
                    eprintln!("Invalid JSON from iroh peer: {}", trimmed);
                }
            }
            Err(e) => {
                eprintln!("iroh read error: {}", e);
                break;
            }
        }
    }

    // Try to finish/close the send side politely if available
    let _ = send.finish();

    Ok(())
}
