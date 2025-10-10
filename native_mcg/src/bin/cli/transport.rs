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
    // Note: keep iroh imports local to avoid compile-time requirement when feature is disabled.
    use iroh::endpoint::Endpoint;
    use iroh::PublicKey;
    use std::str::FromStr;
    use tokio::io::BufReader;

    const ALPN: &[u8] = b"mcg/iroh/1";

    // Build and bind local endpoint
    let endpoint = Endpoint::builder()
        .discovery_n0()
        .bind()
        .await
        .context("binding iroh endpoint for client")?;

    // Resolve peer public key and open connection + bidirectional stream
    let pk = PublicKey::from_str(peer_uri).context("parsing iroh public key (z-base-32)")?;
    let connection = endpoint
        .connect(pk, ALPN)
        .await
        .context("connecting to iroh peer (public key)")?;

    let (mut send, recv) = connection
        .open_bi()
        .await
        .context("opening bidirectional stream")?;

    let mut reader = BufReader::new(recv);

    // Send the client message using a small helper for clarity
    send_client_msg_over_stream(&mut send, &client_msg).await?;

    // Read responses until timeout using a dedicated helper
    let server_msg_opt = read_iroh_responses_until_timeout(&mut reader, wait_ms).await?;

    let latest = match server_msg_opt {
        Some(ServerMsg::State(gs)) => Some(gs),
        Some(ServerMsg::Error(e)) => {
            eprintln!("Server error: {}", e);
            None
        }
        _ => None,
    };

    // Try to finish/close the send side politely if available
    let _ = send.finish();

    Ok(latest)
}

/// Write the provided ClientMsg as newline-delimited JSON to the given writer.
async fn send_client_msg_over_stream<W>(send: &mut W, client_msg: &ClientMsg) -> anyhow::Result<()>
where
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    use tokio::io::AsyncWriteExt;
    let txt = serde_json::to_string(client_msg)?;
    send.write_all(txt.as_bytes()).await?;
    send.write_all(b"\n").await?;
    send.flush().await?;
    Ok(())
}

/// Read newline-delimited ServerMsg responses from `reader` until the timeout (ms)
/// Returns the last ServerMsg received (if any) and leaves handling to the caller.
async fn read_iroh_responses_until_timeout<R>(
    reader: &mut R,
    wait_ms: u64,
) -> anyhow::Result<Option<ServerMsg>>
where
    R: tokio::io::AsyncBufRead + Unpin,
{
    use tokio::io::AsyncBufReadExt;

    let mut last_sm: Option<ServerMsg> = None;
    let mut line = String::new();

    loop {
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
                match serde_json::from_str::<ServerMsg>(trimmed) {
                    Ok(sm) => {
                        // Store the last ServerMsg and leave handling to caller
                        last_sm = Some(sm);
                    }
                    Err(_) => {
                        eprintln!("Invalid JSON from iroh peer: {}", trimmed);
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("iroh read error: {}", e);
                break;
            }
            Err(_) => break, // timeout
        }
    }

    Ok(last_sm)
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
            }
        }
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(None), // timeout
    }
}
