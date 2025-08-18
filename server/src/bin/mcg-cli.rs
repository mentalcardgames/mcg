mod cli;

use clap::Parser;
use url::Url;

use mcg_shared::{ClientMsg, PlayerAction};

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Decide transport: iroh if requested, HTTP if server scheme is http(s), otherwise WebSocket.
    let use_iroh = cli.iroh_peer.is_some();
    let use_http = {
        let s = cli.server.as_str();
        s.starts_with("http://") || s.starts_with("https://")
    };
    let ws_url = if !use_iroh && !use_http {
        Some(cli::build_ws_url(&cli.server)?)
    } else {
        None
    };

    match cli.command {
        Commands::Join => {
            let latest = if use_iroh {
                cli::run_once_iroh(cli.iroh_peer.as_ref().unwrap(), &cli.name, None, cli.wait_ms).await?
            } else if use_http {
                cli::run_once_http(&cli.server, &cli.name, None, cli.wait_ms).await?
            } else {
                cli::run_once(ws_url.as_ref().unwrap(), &cli.name, None, cli.wait_ms).await?
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::State => {
            let latest = if use_iroh {
                cli::run_once_iroh(cli.iroh_peer.as_ref().unwrap(), &cli.name, Some(ClientMsg::RequestState), cli.wait_ms).await?
            } else if use_http {
                cli::run_once_http(&cli.server, &cli.name, Some(ClientMsg::RequestState), cli.wait_ms).await?
            } else {
                cli::run_once(
                    ws_url.as_ref().unwrap(),
                    &cli.name,
                    Some(ClientMsg::RequestState),
                    cli.wait_ms,
                )
                .await?
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::Action { kind, amount } => {
            let pa = match kind {
                cli::ActionKind::Fold => PlayerAction::Fold,
                cli::ActionKind::CheckCall => PlayerAction::CheckCall,
                cli::ActionKind::Bet => PlayerAction::Bet(amount),
            };
            let latest = if use_iroh {
                cli::run_once_iroh(cli.iroh_peer.as_ref().unwrap(), &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
            } else if use_http {
                cli::run_once_http(&cli.server, &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
            } else {
                cli::run_once(ws_url.as_ref().unwrap(), &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::NextHand => {
            let latest = if use_iroh {
                cli::run_once_iroh(cli.iroh_peer.as_ref().unwrap(), &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
            } else if use_http {
                cli::run_once_http(&cli.server, &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
            } else {
                cli::run_once(ws_url.as_ref().unwrap(), &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::Reset { bots } => {
            let latest = if use_iroh {
                cli::run_once_iroh(cli.iroh_peer.as_ref().unwrap(), &cli.name, Some(ClientMsg::ResetGame { bots }), cli.wait_ms).await?
            } else if use_http {
                cli::run_once_http(&cli.server, &cli.name, Some(ClientMsg::ResetGame { bots }), cli.wait_ms).await?
            } else {
                cli::run_once(
                    ws_url.as_ref().unwrap(),
                    &cli.name,
                    Some(ClientMsg::ResetGame { bots }),
                    cli.wait_ms,
                )
                .await?
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::Watch => {
            if use_iroh {
                cli::watch_iroh(cli.iroh_peer.as_ref().unwrap(), &cli.name, cli.json).await?
            } else if use_http {
                cli::watch_http(&cli.server, &cli.name, cli.json).await?
            } else {
                cli::watch_ws(ws_url.as_ref().unwrap(), &cli.name, cli.json).await?
            };
        }
    }

    Ok(())
}
