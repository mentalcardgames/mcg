// Server state management: AppState, Lobby, and helpers that operate on shared state.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::game::Game;
use crate::pretty;
use mcg_shared::GameStatePublic;

/// Shared application state exposed to handlers.
#[derive(Clone)]
pub struct AppState {
    pub(crate) lobby: Arc<RwLock<Lobby>>,
    pub bot_count: usize,
    pub broadcaster: broadcast::Sender<mcg_shared::ServerMsg>,
    /// In-memory shared Config instance. Holds the authoritative configuration
    /// for the running server. Use tokio::sync::RwLock for concurrent access.
    pub config: std::sync::Arc<RwLock<crate::config::Config>>,
    /// Optional path to the TOML config file used by the running server.
    /// If present, transports (e.g. iroh) may persist changes to this path.
    pub config_path: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct Lobby {
    pub(crate) game: Option<Game>,
    pub(crate) last_printed_log_len: usize,
    pub(crate) bots_auto: bool,
}

impl Default for Lobby {
    fn default() -> Self {
        Lobby {
            game: None,
            last_printed_log_len: 0,
            bots_auto: true,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(16);
        AppState {
            lobby: Arc::new(RwLock::new(Lobby::default())),
            bot_count: 0,
            broadcaster: tx,
            config: std::sync::Arc::new(RwLock::new(crate::config::Config::default())),
            config_path: None,
        }
    }
}

/// Ensure a game exists for the given player name (create if missing).
pub async fn ensure_game_started(state: &AppState, name: &str) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if lobby.game.is_none() {
        let g = Game::new(name.to_string(), state.bot_count)
            .with_context(|| format!("creating new game for {}", name))?;
        lobby.game = Some(g);
        println!(
            "[GAME] Created new game for {} with {} bot(s)",
            name, state.bot_count
        );
    }
    Ok(())
}

pub async fn current_state_public(state: &AppState, you_id: usize) -> Option<GameStatePublic> {
    let lobby = state.lobby.read().await;
    lobby.game.as_ref().map(|g| g.public_for(you_id))
}

/// Broadcast the current state (and print new events to server console) to all subscribers.
pub async fn broadcast_state(state: &AppState, you_id: usize) {
    if let Some(gs) = current_state_public(state, you_id).await {
        // Print any newly added events to server console and update bookkeeping.
        let mut lobby = state.lobby.write().await;
        let already = lobby.last_printed_log_len;
        let total = gs.action_log.len();
        if total > already {
            for e in gs.action_log.iter().skip(already) {
                let line = pretty::format_event_human(
                    e,
                    &gs.players,
                    gs.you_id,
                    std::io::stdout().is_terminal(),
                );
                println!("{}", line);
            }
            lobby.last_printed_log_len = total;
        }
        drop(lobby);

        // Broadcast the new state to all subscribers.
        let _ = state.broadcaster.send(mcg_shared::ServerMsg::State(gs));
    }
}

/// Apply an action to the game's state. Returns Some(error_string) if the
/// underlying Game::apply_player_action returned an error, otherwise None.
pub async fn apply_action_to_game(
    state: &AppState,
    actor: usize,
    action: mcg_shared::PlayerAction,
) -> Option<String> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        if let Err(e) = game.apply_player_action(actor, action) {
            return Some(e.to_string());
        }
    }
    None
}

/// Validate that the provided player_id is currently allowed to take an action
/// and apply the action. Returns Ok(()) on success or Err(String) with an error
/// message to send back to the client.
pub async fn validate_and_apply_action(
    state: &AppState,
    player_id: usize,
    action: mcg_shared::PlayerAction,
) -> Result<(), String> {
    // Ensure a game exists and that the requested player is allowed to act.
    let allowed = {
        let lobby_r = state.lobby.read().await;
        if let Some(game) = &lobby_r.game {
            game.stage != mcg_shared::Stage::Showdown && game.to_act == player_id
        } else {
            false
        }
    };
    if !allowed {
        return Err("Not your turn".into());
    }

    // Apply the action using the existing helper. translate underlying errors to String.
    if let Some(e) = apply_action_to_game(state, player_id, action).await {
        return Err(e);
    }
    Ok(())
}

/// Broadcast the current state (and trigger bots if enabled).
pub async fn broadcast_and_drive(state: &AppState, you_id: usize, min_ms: u64, max_ms: u64) {
    // Broadcast updated state to subscribers.
    broadcast_state(state, you_id).await;
    // Drive bots (drive_bots_with_delays itself respects lobby.bots_auto).
    drive_bots_with_delays(state, you_id, min_ms, max_ms).await;
}

/// Advance to the next hand (increment dealer, start a new hand) and print a table header.
pub async fn start_new_hand_and_print(state: &AppState, you_id: usize) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        let n = game.players.len();
        if n > 0 {
            game.dealer_idx = (game.dealer_idx + 1) % n;
        }
        game.start_new_hand()?;
        let sb = game.sb;
        let bb = game.bb;
        let gs = game.public_for(you_id);
        lobby.last_printed_log_len = gs.action_log.len();
        let header = pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
        println!("{}", header);
    }
    Ok(())
}

/// Reset the game with a new Game created for `name` with `bots` bots, and print header.
pub async fn reset_game_with_bots(
    state: &AppState,
    name: &str,
    bots: usize,
    you_id: usize,
) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    match Game::new(name.to_string(), bots) {
        Ok(g) => {
            lobby.game = Some(g);
            if let Some(game) = &mut lobby.game {
                let sb = game.sb;
                let bb = game.bb;
                let gs = game.public_for(you_id);
                lobby.last_printed_log_len = gs.action_log.len();
                let header =
                    pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
                println!("{}", header);
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}

/// Drive bots similarly to the websocket handler, but mutate shared state and
/// broadcast resulting states. Exposed so iroh transport can reuse the same behaviour.
pub async fn drive_bots_with_delays(state: &AppState, you_id: usize, min_ms: u64, max_ms: u64) {
    // Respect per-game bots_auto setting: if false, do not run bots automatically.
    {
        let lobby_r = state.lobby.read().await;
        if !lobby_r.bots_auto {
            return;
        }
    }

    loop {
        // Perform a single bot action if it's their turn
        let did_act = {
            let mut lobby = state.lobby.write().await;
            if let Some(game) = &mut lobby.game {
                if game.stage != mcg_shared::Stage::Showdown && game.to_act != you_id {
                    let actor = game.to_act;
                    // Choose a simple bot action using the same logic as random_bot_action
                    let need = game.current_bet.saturating_sub(game.round_bets[actor]);
                    let action = if need == 0 {
                        mcg_shared::PlayerAction::Bet(game.bb)
                    } else {
                        mcg_shared::PlayerAction::CheckCall
                    };
                    let _ = game.apply_player_action(actor, action);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        // Broadcast updated state to all subscribers
        crate::backend::broadcast_state(state, you_id).await;

        if !did_act {
            break;
        }
        // Sleep a pseudo-random-ish delay between actions without holding non-Send state
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
}
