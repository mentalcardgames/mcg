mod cli;

use clap::Parser;

use mcg_shared::{ClientMsg, PlayerAction, PlayerConfig};

use cli::{Cli, Commands, TransportKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Transport now carries the address/peer. Clone once for use below.
    let transport = cli.transport.clone();

    match cli.command {
        Commands::State => {
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(
                        &peer,
                        ClientMsg::RequestState { player_id: 0 },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(
                        &addr,
                        ClientMsg::RequestState { player_id: 0 },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(
                        &addr,
                        ClientMsg::RequestState { player_id: 0 },
                        cli.wait_ms,
                    )
                    .await?
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
                            player_id: 0,
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
                            player_id: 0,
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
                            player_id: 0,
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
                    cli::run_once_iroh(
                        &peer,
                        ClientMsg::NextHand { player_id: 0 },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(
                        &addr,
                        ClientMsg::NextHand { player_id: 0 },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(
                        &addr,
                        ClientMsg::NextHand { player_id: 0 },
                        cli.wait_ms,
                    )
                    .await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::NewGame => {
            let players = vec![
                PlayerConfig {
                    id: 0,
                    name: cli.name.clone(),
                    is_bot: false,
                },
                PlayerConfig {
                    id: 1,
                    name: "Bot 1".to_string(),
                    is_bot: true,
                },
            ];
            let latest = match transport.clone() {
                TransportKind::Iroh(peer) => {
                    cli::run_once_iroh(
                        &peer,
                        ClientMsg::NewGame {
                            players: players.clone(),
                        },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(
                        &addr,
                        ClientMsg::NewGame {
                            players: players.clone(),
                        },
                        cli.wait_ms,
                    )
                    .await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(
                        &addr,
                        ClientMsg::NewGame {
                            players: players.clone(),
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
        Commands::Watch => {
            match transport {
                TransportKind::Iroh(peer) => cli::watch_iroh(&peer, &cli.name, cli.json).await?,
                TransportKind::Http(addr) => cli::watch_http(&addr, &cli.name, cli.json).await?,
                TransportKind::WebSocket(addr) => cli::watch_ws(&addr, &cli.name, cli.json).await?,
            };
        }
    }

    Ok(())
}
