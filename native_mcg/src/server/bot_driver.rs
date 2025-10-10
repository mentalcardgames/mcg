use super::state::AppState;
use crate::bot::BotContext;
use mcg_shared::{PlayerId, Stage};
use rand::random;
use tokio::time::{sleep, Duration};

const IDLE_SLEEP_MS: u64 = 50;

/// Continuously drive bots whenever it is their turn.
///
/// This loop runs for the lifetime of the server. When no bots are scheduled to
/// act it idles with a short sleep, otherwise it produces a single bot action,
/// broadcasts the state, and waits for a randomized delay before re-checking.
pub async fn run_bot_driver(state: AppState) {
    let mut last_logged_bot: Option<PlayerId> = None;
    let mut logged_idle = false;

    loop {
        let bot_to_act = {
            let lobby = state.lobby.read().await;
            let game = match &lobby.game {
                Some(game) if game.stage != Stage::Showdown => Some(game),
                _ => {
                    last_logged_bot = None;
                    None
                }
            };

            game.and_then(|game| {
                let idx = game.to_act;
                game.players.get(idx).and_then(|player| {
                    if lobby.bots.contains(&player.id) {
                        Some((player.id, player.name.clone()))
                    } else {
                        None
                    }
                })
            })
        };

        if let Some((bot_id, bot_name)) = bot_to_act {
            if last_logged_bot != Some(bot_id) {
                tracing::debug!(player = %bot_name, player_id = ?bot_id, "Bot driver: bot turn detected");
                last_logged_bot = Some(bot_id);
            }
            logged_idle = false;

            let (min_delay, max_delay) = {
                let cfg = state.config.read().await;
                cfg.bot_delay_range()
            };

            if !process_single_bot_action(&state).await {
                tracing::warn!(player = %bot_name, player_id = ?bot_id, "Bot driver: bot action failed or skipped");
                sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
                continue;
            }

            crate::server::broadcast_state(&state).await;

            let delay_ms = pick_delay(min_delay, max_delay);
            tracing::trace!(delay_ms, "Bot driver: sleeping before next bot action");
            sleep(Duration::from_millis(delay_ms)).await;
        } else {
            if !logged_idle {
                tracing::trace!("Bot driver: idle, waiting for bot turn");
                logged_idle = true;
            }
            last_logged_bot = None;
            sleep(Duration::from_millis(IDLE_SLEEP_MS)).await;
        }
    }
}

fn pick_delay(min_ms: u64, max_ms: u64) -> u64 {
    if max_ms <= min_ms {
        return min_ms;
    }
    let span = max_ms - min_ms;
    let jitter = random::<u64>() % (span + 1);
    min_ms + jitter
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
                tracing::info!(
                    "🤖 Bot {} took action: {:?} (stack: {})",
                    player_name,
                    action_for_log,
                    player_stack
                );
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
