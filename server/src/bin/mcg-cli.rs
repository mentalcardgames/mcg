mod cli;

use clap::Parser;

use mcg_shared::{ClientMsg, PlayerAction};

use cli::{Cli, Commands, TransportKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Transport now carries the address/peer. Clone once for use below.
    let transport = cli.transport.clone();

    // Pre-build websocket URL only if transport is a websocket and an address was provided.
    let ws_url = match &transport {
        TransportKind::WebSocket(addr) => Some(cli::build_ws_url(addr)?),
        _ => None,
    };

    match cli.command {
        Commands::Join => {
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(&peer, &cli.name, None, cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(&addr, &cli.name, None, cli.wait_ms).await?
                }
                TransportKind::WebSocket(_addr) => {
                    let ws_url = ws_url.as_ref().unwrap();
                    cli::run_once(ws_url, &cli.name, None, cli.wait_ms).await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::State => {
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(&peer, &cli.name, Some(ClientMsg::RequestState), cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(&addr, &cli.name, Some(ClientMsg::RequestState), cli.wait_ms).await?
                }
                TransportKind::WebSocket(_addr) => {
                    let ws_url = ws_url.as_ref().unwrap();
                    cli::run_once(ws_url, &cli.name, Some(ClientMsg::RequestState), cli.wait_ms).await?
                }
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
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(&peer, &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(&addr, &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
                }
                TransportKind::WebSocket(_addr) => {
                    let ws_url = ws_url.as_ref().unwrap();
                    cli::run_once(ws_url, &cli.name, Some(ClientMsg::Action(pa)), cli.wait_ms).await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::NextHand => {
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(&peer, &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(&addr, &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
                }
                TransportKind::WebSocket(_addr) => {
                    let ws_url = ws_url.as_ref().unwrap();
                    cli::run_once(ws_url, &cli.name, Some(ClientMsg::NextHand), cli.wait_ms).await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::Reset { bots } => {
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(&peer, &cli.name, Some(ClientMsg::ResetGame { bots }), cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(&addr, &cli.name, Some(ClientMsg::ResetGame { bots }), cli.wait_ms).await?
                }
                TransportKind::WebSocket(_addr) => {
                    let ws_url = ws_url.as_ref().unwrap();
                    cli::run_once(ws_url, &cli.name, Some(ClientMsg::ResetGame { bots }), cli.wait_ms).await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::Watch => {
            match transport {
                TransportKind::Iroh(peer) => {
                    cli::watch_iroh(&peer, &cli.name, cli.json).await?
                }
                TransportKind::Http(addr) => {
                    cli::watch_http(&addr, &cli.name, cli.json).await?
                }
                TransportKind::WebSocket(_addr) => {
                    let ws_url = ws_url.as_ref().unwrap();
                    cli::watch_ws(ws_url, &cli.name, cli.json).await?
                }
            };
        }
    }

    Ok(())
}
