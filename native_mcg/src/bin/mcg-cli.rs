mod cli;

use clap::Parser;
use cli::{generate_demo_players, Cli, Commands, TransportKind};
use mcg_shared::{ClientMsg, PlayerAction};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Transport now carries the address/peer. Clone once for use below.
    let transport = cli.transport.clone();

    match cli.command {
        Commands::State => {
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(&peer, ClientMsg::RequestState, cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(&addr, ClientMsg::RequestState, cli.wait_ms).await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(&addr, ClientMsg::RequestState, cli.wait_ms).await?
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
                    cli::run_once_iroh(
                        &peer,
                        ClientMsg::Action {
                            player_id: mcg_shared::PlayerId(0),
                            action: pa,
                        },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(
                        &addr,
                        ClientMsg::Action {
                            player_id: mcg_shared::PlayerId(0),
                            action: pa,
                        },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(
                        &addr,
                        ClientMsg::Action {
                            player_id: mcg_shared::PlayerId(0),
                            action: pa,
                        },
                        cli.wait_ms,
                    )
                    .await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::NextHand => {
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(&peer, ClientMsg::NextHand, cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(&addr, ClientMsg::NextHand, cli.wait_ms).await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(&addr, ClientMsg::NextHand, cli.wait_ms).await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::NewGame => {
            let players = generate_demo_players(3);
            let msg = ClientMsg::NewGame { players };
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => cli::run_once_iroh(&peer, msg, cli.wait_ms).await?,
                TransportKind::Http(addr) => cli::run_once_http(&addr, msg, cli.wait_ms).await?,
                TransportKind::WebSocket(addr) => cli::run_once_ws(&addr, msg, cli.wait_ms).await?,
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::Watch => {
            match transport {
                TransportKind::Iroh(peer) => cli::watch_iroh(&peer, cli.json).await?,
                TransportKind::Http(addr) => cli::watch_http(&addr, cli.json).await?,
                TransportKind::WebSocket(addr) => cli::watch_ws(&addr, cli.json).await?,
            };
        }
    }

    Ok(())
}
