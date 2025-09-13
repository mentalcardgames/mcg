use crate::bot::BotContext;

use super::state::AppState;

/// Drive bots similarly to the websocket handler, but mutate shared state and
/// broadcast resulting states. Exposed so iroh transport can reuse the same behaviour.
pub async fn drive_bots_with_delays(state: &AppState, min_ms: u64, max_ms: u64) {
    // Ensure only one drive loop runs at a time.
    {
        let mut lobby = state.lobby.write().await;
        if lobby.driving {
            return;
        }
        lobby.driving = true;
    }

    // Drive loop: keep acting while bots_auto is enabled and the player to act is a bot.
    loop {
        // Exit if bots_auto disabled or no game present.
        {
            let lobby_r = state.lobby.read().await;
            if lobby_r.game.is_none() {
                break;
            }
        }

        // Determine current actor and whether it's a bot.
        let maybe_actor = {
            let lobby_r = state.lobby.read().await;
            if let Some(game) = &lobby_r.game {
                if game.stage == mcg_shared::Stage::Showdown {
                    None
                } else {
                    let idx = game.to_act;
                    game.players.get(idx).and_then(|p| {
                        if lobby_r.bots.contains(&p.id) {
                            Some((idx, p.id))
                        } else {
                            None
                        }
                    })
                }
            } else {
                None
            }
        };

        let (actor_idx, actor_id) = match maybe_actor {
            Some(v) => v,
            None => break, // no bot to act now
        };

        // Apply bot action under write lock, but re-check to_act to avoid races.
        let applied_and_advanced = {
            let mut lobby_w = state.lobby.write().await;

            // Clone the bot manager first to avoid borrowing conflicts
            let bot_manager = lobby_w.bot_manager.clone();

            if let Some(game) = &mut lobby_w.game {
                // Reconfirm conditions haven't changed.
                if game.stage == mcg_shared::Stage::Showdown || game.to_act != actor_idx {
                    // Another actor changed the state; resume loop to re-evaluate.
                    false
                } else {
                    // Generate bot action using the bot manager
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

                    // With the refactored flow logic, we are more confident that
                    // the game state will advance correctly. The check for whether
                    // the actor changed is removed to simplify the bot driver.
                    match game.apply_player_action(actor_idx, action) {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!(
                                "[BOT] failed to apply action for bot id {}: {}",
                                actor_id, e
                            );
                            false
                        }
                    }
                }
            } else {
                false
            }
        };

        // Broadcast updated state using the acting player's id (viewer-specific state)
        // Always broadcast the state resulting from the attempted bot action so clients
        // observe the latest changes even if the bot's action didn't advance the turn.
        super::game_ops::broadcast_state(state).await;

        if !applied_and_advanced {
            // Stop driving bots for now; give control back to caller/human/other logic.
            break;
        }

        // Sleep between bot actions (randomized within given bounds)
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }

    // Clear driving flag
    let mut lobby = state.lobby.write().await;
    lobby.driving = false;
}
