use std::time::Duration;

use mcg_poker::bot::BotManager;
use mcg_poker::driver::spawn_bot_driver;
use mcg_shared::{Frontend2BackendMsg, PlayerAction, PlayerConfig, PlayerId, Stage};
use native_mcg::config::Config;
use native_mcg::controller::{ControllerBuilder, ControllerHandle};
use native_mcg::network::{ConnectionId, NetworkBuilder, NetworkEvent};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

#[tokio::test]
async fn bot_driver_drives_bot_turns_automatically() {
    let (network_event_tx, _network_event_rx) = mpsc::channel(16);
    let (network, supervisor_task) =
        NetworkBuilder::new(ControllerHandle::new(network_event_tx)).spawn();

    let (state_watch_tx, mut state_watch_rx) = watch::channel(None);
    let (controller_handle, thread_handle) = ControllerBuilder::new(Config::default())
        .with_state_watch(state_watch_tx)
        .with_network(network.clone())
        .with_channel_capacity(16)
        .spawn();

    let bot_driver_task = spawn_bot_driver(
        controller_handle.clone(),
        state_watch_rx.clone(),
        BotManager::new(),
        (20, 50),
    );

    // Start game with 1 human (Alice) and 1 bot (Bob)
    controller_handle
        .send_network_event(NetworkEvent::FrontendMessage {
            connection_id: ConnectionId::new(1),
            message: Frontend2BackendMsg::NewGame {
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
            },
        })
        .await
        .expect("new game event sent");

    state_watch_rx
        .changed()
        .await
        .expect("initial state published");
    let initial_state = state_watch_rx
        .borrow()
        .clone()
        .expect("poker state present");

    // In 2-player game, Alice (SB / dealer, Player 0) acts first preflop
    assert_eq!(initial_state.to_act, PlayerId(0));

    // Alice calls/completes the small blind
    controller_handle
        .send_bot_action(PlayerId(0), PlayerAction::CheckCall)
        .await
        .expect("Alice call action sent");

    state_watch_rx
        .changed()
        .await
        .expect("alice action state published");
    let alice_after_state = state_watch_rx
        .borrow()
        .clone()
        .expect("poker state present");

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
