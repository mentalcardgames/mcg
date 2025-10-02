use crate::game::Game;
use crate::pretty;
use anyhow::{Context, Result};
use mcg_shared::{Card, CardRank, CardSuit, GameStatePublic, PlayerId};
use std::io::IsTerminal;

use crate::backend::AppState;

/// Create a new game with the specified players.
pub async fn create_new_game(
    state: &AppState,
    players: Vec<mcg_shared::PlayerConfig>,
) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    let player_count = players.len();

    // Convert PlayerConfig to internal Player format. The engine's Player type
    // is agnostic about bot status; the backend tracks bot-driven IDs separately.
    let mut game_players = Vec::new();
    let mut bot_ids: Vec<PlayerId> = Vec::new();
    for config in &players {
        if config.is_bot {
            bot_ids.push(config.id);
        }
        let player = crate::game::Player {
            id: config.id,
            name: config.name.clone(),
            stack: 1000, // Default stack size
            cards: [
                Card::new(CardRank::Ace, CardSuit::Clubs),
                Card::new(CardRank::Ace, CardSuit::Clubs),
            ], // Empty cards initially
            has_folded: false,
            all_in: false,
        };
        game_players.push(player);
    }
    // Store bot ids on the lobby so backend drive logic can consult it.
    lobby.bots = bot_ids;

    // Create the game with the players
    let game = Game::with_players(game_players)
        .with_context(|| "creating new game with specified players")?;

    lobby.game = Some(game);
    tracing::info!(player_count = player_count, "created new game");

    Ok(())
}

pub async fn current_state_public(state: &AppState) -> Option<GameStatePublic> {
    let lobby_r = state.lobby.read().await;
    if let Some(game) = &lobby_r.game {
        let gs = game.public();
        Some(gs)
    } else {
        None
    }
}

/// Broadcast the current state (and print new events to server console) to all subscribers.
///
/// Transports receive the same `ServerMsg::State` payload; the backend does not
/// embed per-connection personalization in the broadcast. If transports or a
/// future session manager needs to expose client-specific views, they should
/// compute those on the transport/session layer.
pub async fn broadcast_state(state: &AppState) {
    if let Some(gs) = current_state_public(state).await {
        // Print any newly added events to server console and update bookkeeping.
        let mut lobby = state.lobby.write().await;
        let already = lobby.last_printed_log_len;
        let total = gs.action_log.len();
        if total > already {
            for e in gs.action_log.iter().skip(already) {
                // Use the provided viewer id when formatting server-side logs.
                let line =
                    pretty::format_event_human(e, &gs.players, std::io::stdout().is_terminal());
                tracing::info!(%line);
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
    player_id: PlayerId,
    action: mcg_shared::PlayerAction,
) -> Result<(), String> {
    // First, ensure a game exists
    {
        let lobby_r = state.lobby.read().await;
        if lobby_r.game.is_none() {
            return Err("No active game. Please start a new game first.".into());
        }
    }

    // Resolve provided player_id to the internal player index used by Game
    let actor_idx = {
        let lobby_r = state.lobby.read().await;
        if let Some(game) = &lobby_r.game {
            match game.players.iter().position(|p| p.id == player_id) {
                Some(idx) => idx,
                None => return Err("Unknown player id".into()),
            }
        } else {
            return Err("No active game. Please start a new game first.".into());
        }
    };

    // Ensure the requested player is allowed to act (compare against index)
    let allowed = {
        let lobby_r = state.lobby.read().await;
        if let Some(game) = &lobby_r.game {
            game.stage != mcg_shared::Stage::Showdown && game.to_act == actor_idx
        } else {
            false
        }
    };
    if !allowed {
        return Err("Not your turn".into());
    }

    // Apply the action using the existing helper. translate underlying errors to String.
    if let Some(e) = apply_action_to_game(state, actor_idx, action).await {
        return Err(e);
    }
    Ok(())
}

/// Broadcast the current state (and trigger bots if enabled).
pub async fn broadcast_and_drive(state: &AppState) {
    // Broadcast updated state to subscribers.
    broadcast_state(state).await;
    // Drive bots (drive_bots_with_delays itself respects lobby.bots_auto).
    let config = state.config.read().await;
    let (min_ms, max_ms) = config.bot_delay_range();
    super::bot_driver::drive_bots_with_delays(state, min_ms, max_ms).await;
}

/// Advance to the next hand (increment dealer, start a new hand) and print a table header.
pub async fn start_new_hand_and_print(state: &AppState) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        let n = game.players.len();
        if n > 0 {
            game.dealer_idx = (game.dealer_idx + 1) % n;
        }
        game.start_new_hand()?;
        let sb = game.sb;
        let bb = game.bb;
        // start_new_hand_and_print runs in server-side context
        // for printing the table header and tracking printed log length.
        let gs = game.public();
        lobby.last_printed_log_len = gs.action_log.len();
        let header = pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
        tracing::info!("{}", header);
    }
    Ok(())
}
