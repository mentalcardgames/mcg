//! Event-driven bot driver that observes public game state updates and submits decisions.

mod runner;
mod sink;

pub use runner::{pick_delay, run_bot_driver, spawn_bot_driver};
pub use sink::BotActionSink;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::BotManager;
    use mcg_shared::{PlayerAction, PlayerId, PlayerPublic, PokerStatePublic, Stage};
    use tokio::sync::{mpsc, watch};

    #[derive(Clone)]
    struct MockActionSink {
        tx: mpsc::Sender<(PlayerId, PlayerAction)>,
    }

    #[async_trait::async_trait]
    impl BotActionSink for MockActionSink {
        async fn send_bot_action(
            &self,
            player_id: PlayerId,
            action: PlayerAction,
        ) -> anyhow::Result<()> {
            self.tx.send((player_id, action)).await?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn bot_driver_submits_action_to_sink() {
        let (action_tx, mut action_rx) = mpsc::channel(16);
        let sink = MockActionSink { tx: action_tx };

        let (state_tx, state_rx) = watch::channel(None);

        let driver_task = spawn_bot_driver(sink, state_rx, BotManager::new(), (10, 20));

        let test_state = PokerStatePublic {
            players: vec![
                PlayerPublic {
                    id: PlayerId(0),
                    name: "Alice".into(),
                    stack: 1000,
                    cards: None,
                    has_folded: false,
                    all_in: false,
                    bet_this_round: 10,
                    is_bot: false,
                },
                PlayerPublic {
                    id: PlayerId(1),
                    name: "Bob".into(),
                    stack: 1000,
                    cards: None,
                    has_folded: false,
                    all_in: false,
                    bet_this_round: 5,
                    is_bot: true,
                },
            ],
            community: vec![],
            pot: 15,
            sb: 5,
            bb: 10,
            to_act: PlayerId(1),
            stage: Stage::Preflop,
            winner_ids: vec![],
            action_log: vec![],
            current_bet: 10,
            min_raise: 10,
        };

        state_tx.send(Some(test_state)).expect("send test state");

        let action = tokio::time::timeout(std::time::Duration::from_secs(1), action_rx.recv())
            .await
            .expect("bot action received within timeout")
            .expect("channel not closed");

        assert_eq!(action.0, PlayerId(1));
        assert!(matches!(
            action.1,
            PlayerAction::Fold | PlayerAction::CheckCall | PlayerAction::Bet(_)
        ));

        driver_task.abort();
    }
}
