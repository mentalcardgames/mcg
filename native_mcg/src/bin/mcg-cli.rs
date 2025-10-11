mod cli;

use anyhow::anyhow;
use clap::Parser;
use cli::{generate_demo_players, Cli, Commands, TransportKind};
use mcg_shared::{ClientMsg, PlayerAction};
use native_mcg::public::PublicInfo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Transport now carries the address/peer. Clone once for use below.
    let transport = cli.transport.clone();
    let resolved_iroh_peer = match &transport {
        TransportKind::Iroh { peer } => Some(resolve_iroh_peer(peer.clone())?),
        _ => None,
    };

    match cli.command {
        Commands::State => {
            let latest = match &transport {
                TransportKind::Iroh { .. } => {
                    let peer = resolved_iroh_peer
                        .as_ref()
                        .ok_or_else(|| anyhow!("iroh node id unavailable"))?;
                    cli::run_once_iroh(peer, ClientMsg::RequestState, cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(addr, ClientMsg::RequestState, cli.wait_ms).await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(addr, ClientMsg::RequestState, cli.wait_ms).await?
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
            let latest = match &transport {
                TransportKind::Iroh { .. } => {
                    let peer = resolved_iroh_peer
                        .as_ref()
                        .ok_or_else(|| anyhow!("iroh node id unavailable"))?;
                    cli::run_once_iroh(
                        peer,
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
                        addr,
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
                        addr,
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
            let latest = match &transport {
                TransportKind::Iroh { .. } => {
                    let peer = resolved_iroh_peer
                        .as_ref()
                        .ok_or_else(|| anyhow!("iroh node id unavailable"))?;
                    cli::run_once_iroh(peer, ClientMsg::NextHand, cli.wait_ms).await?
                }
                TransportKind::Http(addr) => {
                    cli::run_once_http(addr, ClientMsg::NextHand, cli.wait_ms).await?
                }
                TransportKind::WebSocket(addr) => {
                    cli::run_once_ws(addr, ClientMsg::NextHand, cli.wait_ms).await?
                }
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::NewGame => {
            let players = generate_demo_players(3);
            let msg = ClientMsg::NewGame { players };
            let latest = match &transport {
                TransportKind::Iroh { .. } => {
                    let peer = resolved_iroh_peer
                        .as_ref()
                        .ok_or_else(|| anyhow!("iroh node id unavailable"))?;
                    cli::run_once_iroh(peer, msg, cli.wait_ms).await?
                }
                TransportKind::Http(addr) => cli::run_once_http(addr, msg, cli.wait_ms).await?,
                TransportKind::WebSocket(addr) => cli::run_once_ws(addr, msg, cli.wait_ms).await?,
            };
            if let Some(state) = latest {
                cli::output_state(&state, cli.json);
            }
        }
        Commands::Watch => {
            match &transport {
                TransportKind::Iroh { .. } => {
                    let peer = resolved_iroh_peer
                        .as_ref()
                        .ok_or_else(|| anyhow!("iroh node id unavailable"))?;
                    cli::watch_iroh(peer, cli.json).await?
                }
                TransportKind::Http(addr) => cli::watch_http(addr, cli.json).await?,
                TransportKind::WebSocket(addr) => cli::watch_ws(addr, cli.json).await?,
            };
        }
        Commands::Ping => match &transport {
            TransportKind::Iroh { .. } => {
                let peer = resolved_iroh_peer
                    .as_ref()
                    .ok_or_else(|| anyhow!("iroh node id unavailable"))?;
                let _ = cli::run_once_iroh(peer, ClientMsg::Ping, cli.wait_ms).await?;
            }
            TransportKind::Http(addr) => {
                let _ = cli::run_once_http(addr, ClientMsg::Ping, cli.wait_ms).await?;
            }
            TransportKind::WebSocket(addr) => {
                let _ = cli::run_once_ws(addr, ClientMsg::Ping, cli.wait_ms).await?;
            }
        },
    }

    Ok(())
}

fn resolve_iroh_peer(peer: Option<String>) -> anyhow::Result<String> {
    if let Some(value) = peer.and_then(|p| {
        let trimmed = p.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }) {
        return Ok(value);
    }

    let path = PublicInfo::default_path();
    let info = PublicInfo::load(&path)?;
    info.iroh_node_id.ok_or_else(|| {
        anyhow!(
            "no iroh node id provided. pass --transport iroh:<PEER> or ensure '{}' contains an 'iroh_node_id' value",
            path.display()
        )
    })
}
