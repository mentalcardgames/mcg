//! Event-driven bot driver that observes public game state updates and submits decisions.
//!
//! Operates fully outside the Controller domain by watching for [`PokerStatePublic`] updates
//! and submitting [`ControllerEvent::BotAction`] via [`ControllerHandle`].

use std::time::Duration;

use mcg_shared::{PlayerAction, PlayerId, PokerStatePublic, Stage};
use rand::random;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::bot::{BotContext, BotManager};
use crate::controller::ControllerHandle;

/// Spawns the bot driver background task that reacts to public state changes.
pub fn spawn_bot_driver(
    controller: ControllerHandle,
    state_rx: watch::Receiver<Option<PokerStatePublic>>,
    bot_manager: BotManager,
    delay_range: (u64, u64),
) -> JoinHandle<()> {
    tokio::spawn(run_bot_driver(
        controller,
        state_rx,
        bot_manager,
        delay_range,
    ))
}

/// Continuously drives bots whenever it is their turn based on public poker state updates.
pub async fn run_bot_driver(
    controller: ControllerHandle,
    mut state_rx: watch::Receiver<Option<PokerStatePublic>>,
    bot_manager: BotManager,
    delay_range: (u64, u64),
) {
    let mut last_logged_bot: Option<PlayerId> = None;

    loop {
        // Wait for a state change notification
        if state_rx.changed().await.is_err() {
            tracing::debug!("state receiver closed; stopping bot driver");
            break;
        }

        let state = match *state_rx.borrow_and_update() {
            Some(ref s) => s.clone(),
            None => {
                last_logged_bot = None;
                continue;
            }
        };

        if state.stage == Stage::Showdown || state.players.is_empty() {
            last_logged_bot = None;
            continue;
        }

        let to_act_id = state.to_act;
        let Some((actor_idx, player)) = state
            .players
            .iter()
            .enumerate()
            .find(|(_, p)| p.id == to_act_id)
        else {
            continue;
        };

        if !player.is_bot {
            last_logged_bot = None;
            continue;
        }

        if last_logged_bot != Some(player.id) {
            tracing::debug!(player = %player.name, player_id = ?player.id, "Bot driver: bot turn detected");
            last_logged_bot = Some(player.id);
        }

        let delay_ms = pick_delay(delay_range.0, delay_range.1);
        tracing::trace!(delay_ms, "Bot driver: sleeping before bot action");

        // Sleep for the randomized delay, but abort sleep if game state changes in the meantime
        tokio::select! {
            _ = sleep(Duration::from_millis(delay_ms)) => {}
            change = state_rx.changed() => {
                if change.is_err() {
                    break;
                }
                // Loop back to evaluate the newest state
                continue;
            }
        }

        // Re-verify that the bot is still expected to act
        let current = state_rx.borrow().clone();
        let Some(current_state) = current else {
            continue;
        };
        if current_state.stage == Stage::Showdown
            || current_state.to_act != to_act_id
            || current_state.stage != state.stage
        {
            tracing::debug!("Game state changed while bot was thinking; skipping stale action");
            continue;
        }

        let need = current_state
            .current_bet
            .saturating_sub(player.bet_this_round);
        let context = BotContext {
            stack: player.stack,
            call_amount: need,
            current_bet: current_state.current_bet,
            big_blind: current_state.bb,
            stage: current_state.stage,
            position: actor_idx,
            total_players: current_state.players.len(),
        };

        let action = match bot_manager.generate_action(&context) {
            Ok(action) => action,
            Err(e) => {
                tracing::error!("Bot manager failed to generate action: {}", e);
                if need == 0 {
                    PlayerAction::CheckCall
                } else {
                    PlayerAction::Fold
                }
            }
        };

        tracing::info!(
            "🤖 Bot {} took action: {:?} (stack: {})",
            player.name,
            action,
            player.stack
        );

        if let Err(e) = controller.send_bot_action(player.id, action).await {
            tracing::warn!(%e, "failed to submit bot action to controller");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::controller::{
        spawn_controller_command_forwarder, start_controller, ChannelControllerSink, Controller,
    };
    use crate::network::NetworkSupervisor;
    use mcg_shared::{Frontend2BackendMsg, PlayerConfig};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn bot_driver_drives_bot_turns_automatically() {
        let (network_event_tx, _network_event_rx) = mpsc::channel(16);
        let (supervisor, network) = NetworkSupervisor::new(network_event_tx);
        let supervisor_task = tokio::spawn(supervisor.run());

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let sink = ChannelControllerSink::new(command_tx);
        let controller = Controller::new(Config::default(), None);
        let (thread_handle, controller_handle) = start_controller(controller, 16, sink);

        let (state_watch_tx, state_watch_rx) = watch::channel(None);
        let _command_forwarder = spawn_controller_command_forwarder(
            command_rx,
            network.clone(),
            None,
            Some(state_watch_tx),
        );

        let bot_driver_task = spawn_bot_driver(
            controller_handle.clone(),
            state_watch_rx.clone(),
            BotManager::new(),
            (1, 5),
        );

        // Start game with 1 human (Alice) and 1 bot (Bob)
        let response = controller_handle
            .send_http_request(Frontend2BackendMsg::NewGame {
                players: vec![
                    PlayerConfig {
                        id: PlayerId(0),
                        name: "Alice".into(),
                        is_bot: false,
                    },
                    PlayerConfig {
                        id: PlayerId(1),
                        name: "Bob".into(),
                        is_bot: true,
                    },
                ],
            })
            .await
            .expect("new game response");

        let mcg_shared::Backend2FrontendMsg::UpdatePokerState(initial_state) = response else {
            panic!("expected UpdatePokerState response");
        };

        // In 2-player game, Alice (SB / dealer, Player 0) acts first preflop
        assert_eq!(initial_state.to_act, PlayerId(0));

        // Alice calls/completes the small blind
        let response2 = controller_handle
            .send_http_request(Frontend2BackendMsg::Action {
                player_id: PlayerId(0),
                action: PlayerAction::CheckCall,
            })
            .await
            .expect("Alice call response");

        let mcg_shared::Backend2FrontendMsg::UpdatePokerState(alice_after_state) = response2 else {
            panic!("expected UpdatePokerState response");
        };

        // Now it is Bob's (Bot, Player 1) turn
        assert_eq!(alice_after_state.to_act, PlayerId(1));

        // Wait for BotDriver to react, take action, and advance the turn/stage
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(50)).await;
            let current = state_watch_rx.borrow().clone();
            if let Some(state) = current {
                // Bob has acted if to_act changed to Alice or stage advanced past Preflop
                if state.to_act == PlayerId(0) || state.stage != Stage::Preflop {
                    break;
                }
            }
            attempts += 1;
            if attempts > 40 {
                panic!("timed out waiting for bot driver to act");
            }
        }

        // Cleanup
        bot_driver_task.abort();
        let _ = bot_driver_task.await;
        controller_handle
            .shutdown()
            .await
            .expect("controller shutdown");
        thread_handle.join().expect("thread join");
        network.shutdown().await.expect("network shutdown");
        let _ = supervisor_task.await;
    }
}
