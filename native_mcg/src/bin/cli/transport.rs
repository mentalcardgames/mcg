use std::time::Duration;

use anyhow::Context;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use mcg_shared::{ClientMsg, GameStatePublic, ServerMsg};

/// Try to build a websocket URL from a base string (like "localhost:3000" or "http://host:3000")
pub fn build_ws_url(base: &str) -> anyhow::Result<Url> {
    let mut url = Url::parse(base).or_else(|_| Url::parse(&format!("http://{}", base)))?;

    match url.scheme() {
        "http" => url.set_scheme("ws").ok(),
        "https" => url.set_scheme("wss").ok(),
        "ws" | "wss" => Some(()),
        _ => None,
    }
    .ok_or_else(|| anyhow::anyhow!("Unsupported URL scheme: {}", url.scheme()))?;

    // Force path to /ws
    if url.path() != "/ws" {
        url.set_path("/ws");
    }
    Ok(url)
}

/// Connect over websocket, send the provided ClientMsg and return the last State seen within `wait_ms`.
/// Accepts an address string (e.g. "ws://host:port/ws" or "http://host:port") and builds the ws URL internally.
pub async fn run_once_ws(
    ws_addr: &str,
    client_msg: ClientMsg,
    wait_ms: u64,
) -> anyhow::Result<Option<GameStatePublic>> {
    let ws_url = build_ws_url(ws_addr)?;
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(ws_url.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();

    // Client message to send
    {
        let txt = serde_json::to_string(&client_msg)?;
        write.send(Message::Text(txt)).await?;
    }

    // Read until timeout, return last State
    let mut latest_state: Option<GameStatePublic> = None;
    loop {
        match tokio::time::timeout(Duration::from_millis(wait_ms), read.next()).await {
            Ok(Some(Ok(Message::Text(txt)))) => {
                if let Ok(sm) = serde_json::from_str::<ServerMsg>(&txt) {
                    match sm {
                        ServerMsg::State(gs) => latest_state = Some(gs),
                        ServerMsg::Error(e) => eprintln!("Server error: {}", e),
                        ServerMsg::Welcome { .. } => {}
                    }
                }
            }
            Ok(Some(Ok(_other))) => { /* ignore */ }
            Ok(Some(Err(e))) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            Ok(None) => break, // socket closed
            Err(_) => break,   // timeout
        }
    }

    Ok(latest_state)
}

/// Minimal iroh run-once client. Mirrors the behavior of `run_once_ws` but over iroh.
/// The iroh imports are inside the function so compilation only fails if iroh is
/// actually required by the build.
pub async fn run_once_iroh(
    peer_uri: &str,
    client_msg: ClientMsg,
    wait_ms: u64,
) -> anyhow::Result<Option<GameStatePublic>> {
    // Implement a minimal iroh client that:
    // - creates a local endpoint
    // - connects to the supplied peer URI
    // - opens a bidirectional stream
    // - sends newline-delimited JSON messages (provided client message)
    // - reads newline-delimited ServerMsg responses and returns the last State seen
    //
    // Note: iroh APIs are imported inside the function to limit compile-time
    // exposure when the feature is disabled.
    use iroh::endpoint::Endpoint;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // ALPN must match the server's ALPN
    const ALPN: &[u8] = b"mcg/iroh/1";

    // Bind a local endpoint (discovery_n0 for default settings)
    let endpoint = Endpoint::builder()
        .discovery_n0()
        .bind()
        .await
        .context("binding iroh endpoint for client")?;

    // Connect to the target peer. Expect the peer_uri to be a valid iroh target.
    // Interpret peer_uri as a public key (NodeId). Use discovery/relay to dial by public key.
    use iroh::PublicKey;
    use std::str::FromStr;
    // Expect the supplied peer_uri to be a PublicKey (z-base-32). The CLI
    // accepts only the PublicKey form for dialing.
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

    let mut reader = BufReader::new(recv);

    // Client message to send
    {
        let txt = serde_json::to_string(&client_msg)?;
        send.write_all(txt.as_bytes()).await?;
        send.write_all(b"\n").await?;
        send.flush().await?;
    }

    // Read newline-delimited JSON messages until timeout; track last State
    let mut latest_state: Option<GameStatePublic> = None;
    let mut line = String::new();
    loop {
        // read_line appends to the buffer so we clear before each call
        line.clear();
        match tokio::time::timeout(
            std::time::Duration::from_millis(wait_ms),
            reader.read_line(&mut line),
        )
        .await
        {
            Ok(Ok(0)) => break, // connection closed
            Ok(Ok(_)) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(sm) = serde_json::from_str::<ServerMsg>(trimmed) {
                    match sm {
                        ServerMsg::State(gs) => latest_state = Some(gs),
                        ServerMsg::Error(e) => eprintln!("Server error: {}", e),
                        ServerMsg::Welcome { .. } => {}
                    }
                } else {
                    eprintln!("Invalid JSON from iroh peer: {}", trimmed);
                }
            }
            Ok(Err(e)) => {
                eprintln!("iroh read error: {}", e);
                break;
            }
            Err(_) => break, // timeout
        }
    }

    // Try to finish/close the send side politely if available
    let _ = send.finish();

    Ok(latest_state)
}

/// Run a single HTTP-based action sequence and attempt to GET state.
pub async fn run_once_http(
    base: &str,
    client_msg: ClientMsg,
    wait_ms: u64,
) -> anyhow::Result<Option<GameStatePublic>> {
    // Use reqwest to POST newgame and optional action, then GET state once with a timeout.
    let client = reqwest::Client::new();
    // Client message to send
    {
        let _ = client
            .post(format!("{}/api/action", base))
            .json(&client_msg)
            .send()
            .await?;
    }
    // Attempt a single GET state request with timeout equal to wait_ms
    match tokio::time::timeout(std::time::Duration::from_millis(wait_ms), async {
        client.get(format!("{}/api/state", base)).send().await
    })
    .await
    {
        Ok(Ok(r)) => {
            let sm: ServerMsg = r.json().await?;
            match sm {
                ServerMsg::State(gs) => Ok(Some(gs)),
                ServerMsg::Error(e) => {
                    eprintln!("Server error: {}", e);
                    Ok(None)
                }
                _ => Ok(None),
            }
        }
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(None), // timeout
    }
}
