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

    // Build ws URL
    let ws_url = build_ws_url(&cli.server)?;

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
