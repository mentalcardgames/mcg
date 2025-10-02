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
        let (should_process_bot, current_player_info) = {
            let lobby_r = state.lobby.read().await;
            if lobby_r.game.is_none() {
                tracing::debug!("Bot driver: No game present, stopping");
                break;
            }

            if let Some(game) = &lobby_r.game {
                if game.stage == mcg_shared::Stage::Showdown {
                    tracing::debug!("Bot driver: Game in showdown, stopping");
                    (false, None)
                } else {
                    let idx = game.to_act;
                    if let Some(player) = game.players.get(idx) {
                        let is_bot = lobby_r.bots.contains(&player.id);
                        let info = format!("Player {} ({})", player.name, if is_bot { "BOT" } else { "HUMAN" });
                        tracing::debug!("Bot driver: Current player to act: {}, is_bot: {}", info, is_bot);
                        (is_bot, Some(info))
                    } else {
                        tracing::warn!("Bot driver: Invalid player index {}", idx);
                        (false, None)
                    }
                }
            } else {
                (false, None)
            }
        };

        if !should_process_bot {
            if let Some(info) = current_player_info {
                tracing::debug!("Bot driver: Stopping - current player is human: {}", info);
            }
            break;
        }

        // Process exactly ONE bot action
        let bot_action_result = process_single_bot_action(state).await;

        // IMMEDIATELY broadcast the state after each bot action
        tracing::debug!("Broadcasting state after bot action, result: {}", bot_action_result);
        crate::server::broadcast_state(state).await;

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
        let player_name = game.players[actor_idx].name.clone();
        let player_stack = game.players[actor_idx].stack;

        // Apply the bot action
        match game.apply_player_action(actor_idx, action) {
            Ok(_) => {
                tracing::info!("🤖 Bot {} took action: {:?} (stack: {})",
                    player_name, action_for_log, player_stack);
                true
            }
            Err(e) => {
                tracing::error!("❌ Bot {} failed to apply action: {}", player_name, e);
                false
            }
        }
    } else {
        false
    }
}
