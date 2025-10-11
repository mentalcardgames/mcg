use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use mcg_shared::{ClientMsg, ServerMsg};

use super::utils::{DisplayMode, MessagePrinter};

fn announce_connection(json: bool, message: &str) {
    if json {
        eprintln!("{}", message);
    } else {
        println!("{}", message);
    }
}

/// Watch over a websocket connection and print events as they arrive.
/// Accepts an address string (e.g. "ws://host:port/ws" or "http://host:port") and builds the ws URL internally.
pub async fn watch_ws(ws_addr: &str, json: bool) -> anyhow::Result<()> {
    let ws_url = super::transport::build_ws_url(ws_addr)?;
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(ws_url.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();

    let subscribe_txt = serde_json::to_string(&ClientMsg::Subscribe)?;
    write.send(Message::Text(subscribe_txt)).await?;

    announce_connection(json, &format!("Connected to WebSocket {}", ws_url));

    let mut printer = MessagePrinter::new(json, DisplayMode::Incremental);
    loop {
        match read.next().await {
            Some(Ok(Message::Text(txt))) => {
                if let Ok(sm) = serde_json::from_str::<ServerMsg>(&txt) {
                    printer.handle(&sm);
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
    announce_connection(json, &format!("Polling HTTP endpoint {}", base));
    let mut printer = MessagePrinter::new(json, DisplayMode::Incremental);
    loop {
        // Long-poll GET state with a 30s timeout
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client
                .post(format!("{}/api/message", base))
                .json(&ClientMsg::RequestState)
                .send(),
        )
        .await
        {
            Ok(Ok(resp)) => {
                if let Ok(sm) = resp.json::<ServerMsg>().await {
                    printer.handle(&sm);
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

    announce_connection(json, &format!("Connected to Iroh peer {}", peer_uri));

    let mut reader = BufReader::new(recv);

    // Read newline-delimited JSON messages and handle them via shared handler.
    let mut line = String::new();
    let mut printer = MessagePrinter::new(json, DisplayMode::Incremental);
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
                    printer.handle(&sm);
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
