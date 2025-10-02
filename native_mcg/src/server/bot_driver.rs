use crate::bot::BotContext;

use super::state::AppState;

/// Drive bots with non-blocking, event-driven architecture.
/// This function processes bot actions one at a time with immediate broadcasts.
pub async fn drive_bots_with_delays(state: &AppState, min_ms: u64, max_ms: u64) {
    // Ensure only one drive loop runs at a time.
    {
        let mut lobby = state.lobby.write().await;
        if lobby.driving {
            return;
        }
        lobby.driving = true;
    }

    // Process bots in a loop, but broadcast after each action
    loop {
        // Check if we should process a bot action
        let should_process_bot = {
            let lobby_r = state.lobby.read().await;
            if lobby_r.game.is_none() {
                break;
            }

            if let Some(game) = &lobby_r.game {
                if game.stage == mcg_shared::Stage::Showdown {
                    false
                } else {
                    let idx = game.to_act;
                    game.players.get(idx)
                        .map(|p| lobby_r.bots.contains(&p.id))
                        .unwrap_or(false)
                }
            } else {
                false
            }
        };

        if !should_process_bot {
            break;
        }

        // Process exactly ONE bot action
        let bot_action_result = process_single_bot_action(state).await;

        // IMMEDIATELY broadcast the state after each bot action
        super::game_ops::broadcast_state(state).await;

        if !bot_action_result {
            break; // Stop if bot action failed
        }

        // Calculate delay for next bot action
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));

        // Sleep between bot actions (but clients already received the update)
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }

    // Clear driving flag
    {
        let mut lobby = state.lobby.write().await;
        lobby.driving = false;
    }
}

/// Process a single bot action and return whether it was successful
async fn process_single_bot_action(state: &AppState) -> bool {
    let mut lobby_w = state.lobby.write().await;

    // Clone the bot manager first to avoid borrowing conflicts
    let bot_manager = lobby_w.bot_manager.clone();
    let bots = lobby_w.bots.clone();

    if let Some(game) = &mut lobby_w.game {
        let actor_idx = game.to_act;

        // Double-check that the current player is still a bot
        if let Some(player) = game.players.get(actor_idx) {
            if !bots.contains(&player.id) {
                return false; // Not a bot anymore
            }
        } else {
            return false; // Invalid player index
        }

        // Generate bot action
        let need = game.current_bet.saturating_sub(game.round_bets[actor_idx]);
        let context = BotContext {
            stack: game.players[actor_idx].stack,
            call_amount: need,
            current_bet: game.current_bet,
            big_blind: game.bb,
            stage: game.stage,
            position: actor_idx,
            total_players: game.players.len(),
        };

        let action = match bot_manager.generate_action(&context) {
            Ok(action) => action,
            Err(e) => {
                tracing::error!("Bot manager failed to generate action: {}", e);
                // Fallback to a safe action
                if need == 0 {
                    mcg_shared::PlayerAction::CheckCall
                } else {
                    mcg_shared::PlayerAction::Fold
                }
            }
        };

        // Clone action for logging
        let action_for_log = action.clone();
        let player_id = game.players[actor_idx].id;

        // Apply the bot action
        match game.apply_player_action(actor_idx, action) {
            Ok(_) => {
                tracing::debug!("Bot {} took action: {:?}", player_id, action_for_log);
                true
            }
            Err(e) => {
                tracing::error!("Bot {} failed to apply action: {}", player_id, e);
                false
            }
        }
    } else {
        false
    }
}
