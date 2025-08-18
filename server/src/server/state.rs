// Server state management: AppState, Lobby, and helpers that operate on shared state.

use std::sync::Arc;
use std::io::IsTerminal;

use tokio::sync::RwLock;
use tokio::sync::broadcast;
use anyhow::{Result, Context};

use crate::game::Game;
use mcg_shared::GameStatePublic;
use crate::pretty;

/// Shared application state exposed to handlers.
#[derive(Clone)]
pub struct AppState {
    pub(crate) lobby: Arc<RwLock<Lobby>>,
    pub bot_count: usize,
    pub broadcaster: broadcast::Sender<mcg_shared::ServerMsg>,
}

#[derive(Clone, Default)]
pub(crate) struct Lobby {
    pub(crate) game: Option<Game>,
    pub(crate) last_printed_log_len: usize,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(16);
        AppState {
            lobby: Arc::new(RwLock::new(Lobby::default())),
            bot_count: 0,
            broadcaster: tx,
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

/// Advance to the next hand (increment dealer, start a new hand) and print a table header.
pub async fn start_new_hand_and_print(state: &AppState, you_id: usize) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        let n = game.players.len();
        if n > 0 {
            game.dealer_idx = (game.dealer_idx + 1) % n;
        }
        if let Err(e) = game.start_new_hand() {
            return Err(e);
        }
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
                let header = pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
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
        crate::server::broadcast_state(state, you_id).await;

        if !did_act {
            break;
        }
        // Sleep a pseudo-random-ish delay between actions without holding non-Send state
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
}
