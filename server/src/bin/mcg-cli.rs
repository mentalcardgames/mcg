use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use futures_util::{SinkExt, StreamExt};
use mcg_server::pretty::format_state_human;
use mcg_shared::{ClientMsg, GameStatePublic, PlayerAction, ServerMsg};
use std::io::IsTerminal;
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use anyhow::Context;

#[derive(Parser, Debug, Clone)]
#[command(name = "mcg-cli", version, about = "Headless CLI for MCG poker demo", long_about = None)]
struct Cli {
    /// Base server URL (http(s)://host:port or ws(s)://host:port/ws)
    #[arg(short, long, default_value = "http://localhost:3000")]
    server: String,
 
    /// Join name to use for the single player
    #[arg(short, long, default_value = "CLI")]
    name: String,
 
    /// Connect to an iroh peer by public key (z-base-32). When set the CLI
    /// attempts to use iroh transport instead of WebSocket. (Explicit target.)
    #[arg(long)]
    iroh_peer: Option<String>,

    /// How long to wait for server state updates after sending a command (ms)
    #[arg(long, default_value_t = 1200)]
    wait_ms: u64,

    /// Output JSON instead of human-readable text
    #[arg(long, default_value_t = false)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Connect and send Join, then print the first State and exit
    Join,
    /// Request the latest State and print it
    State,
    /// Send an action (Fold, Check/Call, Bet amount)
    Action {
        #[arg(value_enum)]
        kind: ActionKind,
        /// Amount for bet action (ignored for fold/checkcall)
        #[arg(long, default_value_t = 0)]
        amount: u32,
    },
    /// Advance to the next hand
    NextHand,
    /// Reset the game with N bots
    Reset {
        #[arg(long, short = 'b', default_value_t = 1)]
        bots: usize,
    },
    /// Watch game events continuously and print them as they happen
    Watch,
}

#[derive(Debug, Clone, ValueEnum)]
enum ActionKind {
    Fold,
    CheckCall,
    Bet,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Build ws URL only if iroh_peer not requested
    let ws_url = if cli.iroh_peer.is_none() {
        Some(build_ws_url(&cli.server)?)
    } else {
        None
    };

    match cli.command {
        Commands::Join => {
            let latest = if let Some(peer) = &cli.iroh_peer {
                run_once_iroh(peer, &cli.name, None, cli.wait_ms).await?
            } else {
                run_once(ws_url.as_ref().unwrap(), &cli.name, None, cli.wait_ms).await?
            };
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::State => {
            let latest = if let Some(peer) = &cli.iroh_peer {
                run_once_iroh(peer, &cli.name, Some(ClientMsg::RequestState), cli.wait_ms).await?
            } else {
                run_once(
                    ws_url.as_ref().unwrap(),
                    &cli.name,
                    Some(ClientMsg::RequestState),
                    cli.wait_ms,
                )
                .await?
            };
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::Action { kind, amount } => {
            let pa = match kind {
                ActionKind::Fold => PlayerAction::Fold,
                ActionKind::CheckCall => PlayerAction::CheckCall,
                ActionKind::Bet => PlayerAction::Bet(amount),
            };
            let latest = if let Some(peer) = &cli.iroh_peer {
                run_once_iroh(peer, &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
            } else {
                run_once(ws_url.as_ref().unwrap(), &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
            };
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::NextHand => {
            let latest = if let Some(peer) = &cli.iroh_peer {
                run_once_iroh(peer, &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
            } else {
                run_once(ws_url.as_ref().unwrap(), &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
            };
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::Reset { bots } => {
            let latest = if let Some(peer) = &cli.iroh_peer {
                run_once_iroh(peer, &cli.name, Some(ClientMsg::ResetGame { bots }), cli.wait_ms).await?
            } else {
                run_once(
                    ws_url.as_ref().unwrap(),
                    &cli.name,
                    Some(ClientMsg::ResetGame { bots }),
                    cli.wait_ms,
                )
                .await?
            };
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::Watch => {
            if let Some(peer) = &cli.iroh_peer {
                watch_iroh(peer, &cli.name, cli.json).await?
            } else {
                watch_ws(ws_url.as_ref().unwrap(), &cli.name, cli.json).await?
            };
        }
    }

    Ok(())
}
 
/// Placeholder iroh client path. Currently returns an explanatory error until
/// a full iroh client implementation is added. The function signature mirrors
/// `run_once` so the CLI can switch between transports cleanly.
async fn run_once_iroh(
    peer_uri: &str,
    name: &str,
    after_join: Option<ClientMsg>,
    wait_ms: u64,
) -> anyhow::Result<Option<GameStatePublic>> {
    // Implement a minimal iroh client that:
    // - creates a local endpoint
    // - connects to the supplied peer URI
    // - opens a bidirectional stream
    // - sends newline-delimited JSON messages (Join + optional command)
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
    // Parse the supplied peer identifier as an iroh PublicKey (NodeId) and
    // pass it to Endpoint::connect. NodeAddr implements From<PublicKey>, so
    // passing the PublicKey satisfies the connect() signature and enables
    // discovery/relay resolution within the iroh library.
    use std::str::FromStr;
    use iroh::PublicKey;
    // Expect the supplied peer_uri to be a PublicKey (z-base-32). The CLI
    // accepts only the PublicKey form for dialing.
    let pk = PublicKey::from_str(peer_uri)
        .context("parsing iroh public key (z-base-32)")?;
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

    // Send Join
    let join_txt = serde_json::to_string(&ClientMsg::Join {
        name: name.to_string(),
    })?;
    send.write_all(join_txt.as_bytes()).await?;
    send.write_all(b"\n").await?;
    send.flush().await?;

    // Optional follow-up command
    if let Some(msg) = after_join {
        let txt = serde_json::to_string(&msg)?;
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
        match tokio::time::timeout(std::time::Duration::from_millis(wait_ms), reader.read_line(&mut line)).await {
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

fn output_state(state: &GameStatePublic, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(state).unwrap());
    } else {
        let use_color = std::io::stdout().is_terminal();
        println!("{}", format_state_human(state, use_color));
    }
}

fn build_ws_url(base: &str) -> anyhow::Result<Url> {
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

async fn run_once(
    ws_url: &Url,
    name: &str,
    after_join: Option<ClientMsg>,
    wait_ms: u64,
) -> anyhow::Result<Option<GameStatePublic>> {
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(ws_url.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();

    // Always join first
    let join = serde_json::to_string(&ClientMsg::Join {
        name: name.to_string(),
    })?;
    write.send(Message::Text(join)).await?;

    // Optional follow-up command
    if let Some(msg) = after_join {
        let txt = serde_json::to_string(&msg)?;
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

/// Shared handler for server messages so the CLI doesn't duplicate logic
fn handle_server_msg(sm: &ServerMsg, json: bool) {
    match sm {
        ServerMsg::State(gs) => output_state(gs, json),
        ServerMsg::Error(e) => eprintln!("Server error: {}", e),
        ServerMsg::Welcome { .. } => {
            if json {
                // If user wants JSON, print the welcome message as JSON.
                if let Ok(txt) = serde_json::to_string_pretty(sm) {
                    println!("{}", txt);
                }
            } else {
                // For human output we don't print anything special for Welcome
            }
        }
    }
}

/// Watch over a websocket connection and print events as they arrive.
async fn watch_ws(ws_url: &Url, name: &str, json: bool) -> anyhow::Result<()> {
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(ws_url.as_str()).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send Join
    let join = serde_json::to_string(&ClientMsg::Join {
        name: name.to_string(),
    })?;
    write.send(Message::Text(join)).await?;

    // Read messages forever (until socket closed or error) and handle them via
    // the shared handler.
    loop {
        match read.next().await {
            Some(Ok(Message::Text(txt))) => {
                if let Ok(sm) = serde_json::from_str::<ServerMsg>(&txt) {
                    handle_server_msg(&sm, json);
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

/// Watch over an iroh bidirectional stream and print events as they arrive.
///
/// The function mirrors the websocket watcher in behavior but reuses the
/// same `handle_server_msg` helper so there is no duplicate message handling.
async fn watch_iroh(peer_uri: &str, name: &str, json: bool) -> anyhow::Result<()> {
    // Import iroh APIs inside the function to limit compile-time exposure.
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

    use std::str::FromStr;
    use iroh::PublicKey;
    let pk = PublicKey::from_str(peer_uri)
        .context("parsing iroh public key (z-base-32)")?;
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

    // Send Join
    let join_txt = serde_json::to_string(&ClientMsg::Join {
        name: name.to_string(),
    })?;
    send.write_all(join_txt.as_bytes()).await?;
    send.write_all(b"\n").await?;
    send.flush().await?;

    // Read newline-delimited JSON messages and handle them via shared handler.
    let mut line = String::new();
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
                    handle_server_msg(&sm, json);
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
