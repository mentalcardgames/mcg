use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use futures_util::{SinkExt, StreamExt};
use mcg_server::pretty::format_state_human;
use mcg_shared::{ClientMsg, GameStatePublic, PlayerAction, ServerMsg};
use std::io::IsTerminal;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(name = "mcg-cli", version, about = "Headless CLI for MCG poker demo", long_about = None)]
struct Cli {
    /// Base server URL (http(s)://host:port or ws(s)://host:port/ws)
    #[arg(short, long, default_value = "http://localhost:3000")]
    server: String,

    /// Transport to use: websocket or iroh
    #[arg(long, default_value = "websocket")]
    #[arg(long, value_enum, default_value = "websocket")]
    transport: TransportKind,

    /// If using iroh transport, optional node id to target (otherwise auto-detect)
    #[arg(long)]
    iroh_node_id: Option<String>,

    /// Join name to use for the single player
    #[arg(short, long, default_value = "CLI")]
    name: String,

    /// How long to wait for server state updates after sending a command (ms)
    #[arg(long, default_value_t = 1200)]
    wait_ms: u64,

    /// Output JSON instead of human-readable text
    #[arg(long, default_value_t = false)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum TransportKind {
    Websocket,
    Iroh,
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

    // Build ws URL (used when websocket transport is selected)
    let ws_url = build_ws_url(&cli.server)?;

    // If using iroh transport, perform iroh-based run logic (medium-agnostic CLI)
    if let TransportKind::Iroh = cli.transport {
        // Determine server node id (either provided or detect via admin endpoint)
        let server_node_id = if let Some(id) = cli.iroh_node_id.clone() {
            id
        } else {
            match detect_iroh_node_id(&cli.server).await? {
                Some(id) => id,
                None => {
                    eprintln!("Could not detect iroh node id from server at {}", cli.server);
                    return Ok(());
                }
            }
        };

        // Run using iroh transport
        let result = run_once_iroh(&server_node_id, &cli.name, map_after_join(&cli.command), cli.wait_ms).await?;
        if let Some(state) = result {
            output_state(&state, cli.json);
        }
        return Ok(());
    }

    match cli.command {
        Commands::Join => {
            let latest = run_once(&ws_url, &cli.name, None, cli.wait_ms).await?;
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::State => {
            let latest = run_once(
                &ws_url,
                &cli.name,
                Some(ClientMsg::RequestState),
                cli.wait_ms,
            )
            .await?;
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
            let latest =
                run_once(&ws_url, &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?;
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::NextHand => {
            let latest =
                run_once(&ws_url, &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?;
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
        Commands::Reset { bots } => {
            let latest = run_once(
                &ws_url,
                &cli.name,
                Some(ClientMsg::ResetGame { bots }),
                cli.wait_ms,
            )
            .await?;
            if let Some(state) = latest {
                output_state(&state, cli.json);
            }
        }
    }

    Ok(())
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

// Map a CLI command to an optional ClientMsg to send after Join
fn map_after_join(cmd: &Commands) -> Option<ClientMsg> {
    match cmd {
        Commands::Join => None,
        Commands::State => Some(ClientMsg::RequestState),
        Commands::NextHand => Some(ClientMsg::NextHand),
        Commands::Reset { bots } => Some(ClientMsg::ResetGame { bots: *bots }),
        Commands::Action { kind, amount } => {
            let pa = match kind {
                ActionKind::Fold => PlayerAction::Fold,
                ActionKind::CheckCall => PlayerAction::CheckCall,
                ActionKind::Bet => PlayerAction::Bet(*amount),
            };
            Some(ClientMsg::Action(pa))
        }
    }
}

/// Run a single command via iroh by dialing the server node id and sending a framed ClientMsg.
async fn run_once_iroh(
    server_node_id: &str,
    name: &str,
    after_join: Option<ClientMsg>,
    wait_ms: u64,
) -> anyhow::Result<Option<GameStatePublic>> {
    use iroh::endpoint::Endpoint;
    use iroh_base::NodeAddr;

    // Build an Endpoint, dial the server node id and send a single bi stream with Join + optional msg
    let endpoint = Endpoint::builder().discovery_n0().bind().await?;
    let node_id: iroh_base::NodeId = server_node_id.parse().map_err(|e| anyhow::anyhow!(e))?;
    let node_addr = NodeAddr::new(node_id);
    let conn = endpoint.connect(node_addr, b"/mcg/msg/1").await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    // Compose frames: Join then optional after_join
    let mut payloads: Vec<Vec<u8>> = Vec::new();
    payloads.push(serde_json::to_vec(&ClientMsg::Join { name: name.to_string() })?);
    if let Some(msg) = after_join {
        payloads.push(serde_json::to_vec(&msg)?);
    }

    // Write all frames concatenated using the same framing
    for p in payloads.iter() {
        let framed = crate::transport::framing::encode_frame(p);
        use tokio::io::AsyncWriteExt;
        send.write_all(&framed).await?;
    }
    // finish() is synchronous in this iroh version
    send.finish()?;

    // Read responses until timeout and parse any ServerMsg::State frames
    let data = recv.read_to_end(10_000_000).await?;
    let mut buf = bytes::BytesMut::from(&data[..]);
    let mut latest_state: Option<GameStatePublic> = None;
    loop {
        match crate::transport::framing::try_parse(&mut buf) {
            Ok(Some(frame)) => {
                if let Ok(txt) = String::from_utf8(frame) {
                    if let Ok(sm) = serde_json::from_str::<ServerMsg>(&txt) {
                        if let ServerMsg::State(gs) = sm {
                            latest_state = Some(gs);
                        }
                    }
                }
                continue;
            }
            Ok(None) => break,
            Err(e) => {
                eprintln!("[IROH] frame parse error: {:?}", e);
                break;
            }
        }
    }

    Ok(latest_state)
}

async fn detect_iroh_node_id(server_base: &str) -> anyhow::Result<Option<String>> {
    // Try the admin endpoint at /admin/iroh/node_id
    let url = if server_base.ends_with('/') {
        format!("{}admin/iroh/node_id", server_base)
    } else {
        format!("{}/admin/iroh/node_id", server_base)
    };
    if let Ok(resp) = reqwest::get(&url).await {
        if resp.status().is_success() {
            if let Ok(body) = resp.text().await {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(id) = v.get("node_id") {
                        if id.is_string() {
                            return Ok(Some(id.as_str().unwrap().to_string()));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}
