//! Bot AI logic extracted from backend state management.
//!
//! This module provides a clean interface for bot decision-making and action
//! generation, separated from the backend state management concerns.

use anyhow::Result;
use mcg_shared::{PlayerAction, Stage};
use rand::random;

/// Information about a bot player's current situation needed for decision making.
#[derive(Debug, Clone)]
pub struct BotContext {
    /// The bot's current stack
    pub stack: u32,
    /// Amount needed to call the current bet
    pub call_amount: u32,
    /// Current bet in the round
    pub current_bet: u32,
    /// Big blind amount
    pub big_blind: u32,
    /// Current stage of the game
    pub stage: Stage,
    /// Bot's position/index in the game
    pub position: usize,
    /// Total number of players
    pub total_players: usize,
}

/// Simple bot implementation using basic probabilistic decision making.
/// This implements the same logic that was previously embedded in the backend state.
#[derive(Debug, Clone)]
pub struct SimpleBot {
    /// Base probability of folding (0.0 to 1.0)
    pub base_fold_chance: f64,
    /// Maximum fold probability cap (0.0 to 1.0)
    pub max_fold_chance: f64,
}

impl Default for SimpleBot {
    fn default() -> Self {
        Self {
            base_fold_chance: 0.10, // 10% baseline fold chance
            max_fold_chance: 0.95,  // Cap at 95% fold chance
        }
    }
}

impl SimpleBot {
    /// Decide what action the bot should take given the current context.
    pub fn decide_action(&self, context: &BotContext) -> PlayerAction {
        if context.call_amount == 0 {
            // No outstanding bet: decide whether to check or make an opening bet
            if random::<f64>() < 0.3 {
                // 30% chance to check
                PlayerAction::CheckCall
            } else {
                // 70% chance to make an opening bet of varying sizes
                let bet_options = [
                    context.big_blind,                       // Min bet
                    context.big_blind * 2,                   // 2x big blind
                    context.big_blind * 3,                   // 3x big blind
                    (context.big_blind as f64 * 2.5) as u32, // 2.5x big blind
                ];

                let random_index = (random::<f32>() * bet_options.len() as f32) as usize;
                let bet_amount =
                    bet_options[random_index.min(bet_options.len() - 1)].min(context.stack);

                PlayerAction::Bet(bet_amount)
            }
        } else if context.call_amount >= context.stack {
            // Calling would require all-in: just call
            PlayerAction::CheckCall
        } else {
            // There is a bet to call. Use probabilistic decision making:
            // Higher relative bet size -> more likely to fold
            let relative_bet =
                context.call_amount as f64 / (context.stack + context.current_bet) as f64;

            // Blend base fold chance with relative bet-based chance
            let fold_chance = (self.base_fold_chance
                + relative_bet * (1.0 - self.base_fold_chance))
                .min(self.max_fold_chance);

            if random::<f64>() < fold_chance {
                PlayerAction::Fold
            } else {
                // Decide whether to call or raise
                let raise_chance = 0.2; // 20% chance to raise instead of call

                if random::<f64>() < raise_chance && context.stack > context.call_amount {
                    // Choose a raise amount randomly
                    let remaining_after_call = context.stack - context.call_amount;
                    // Ensure minimum raise is at least the big blind to prevent Bet(0)
                    let min_raise =
                        ((context.current_bet as f64 * 0.5) as u32).max(context.big_blind);
                    let max_raise = (context.current_bet as f64 * 1.5) as u32; // 1.5x pot maximum

                    let raise_options = [
                        min_raise,
                        context.current_bet.max(context.big_blind), // Pot-sized raise (min big blind)
                        max_raise.max(context.big_blind),           // Max raise (min big blind)
                        remaining_after_call / 2,                   // Half remaining stack
                        remaining_after_call,                       // All-in
                    ];

                    let random_index = (random::<f32>() * raise_options.len() as f32) as usize;
                    let raise_amount = raise_options[random_index.min(raise_options.len() - 1)]
                        .max(min_raise)
                        .min(remaining_after_call);

                    PlayerAction::Bet(raise_amount)
                } else {
                    PlayerAction::CheckCall
                }
            }
        }
    }
}

/// Bot manager that handles bot decision-making and provides the interface
/// between the backend state and bot AI implementations.
///
/// Note: Previously used `Box<dyn BotAI>` for theoretical extensibility, but since
/// only `SimpleBot` is ever used, we now embed it directly for simplicity.
#[derive(Debug, Clone, Default)]
pub struct BotManager {
    bot: SimpleBot,
}

impl BotManager {
    /// Create a new bot manager with the default SimpleBot AI.
    pub fn new() -> Self {
        Self {
            bot: SimpleBot::default(),
        }
    }

    /// Generate a bot action given the current game context.
    pub fn generate_action(&self, context: &BotContext) -> Result<PlayerAction> {
        let action = self.bot.decide_action(context);
        tracing::debug!(
            "Bot decision: {:?} (call_amount: {}, stack: {}, stage: {:?})",
            action,
            context.call_amount,
            context.stack,
            context.stage
        );
        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_bot_acts_with_no_current_bet() {
        let bot = SimpleBot::default();
        let context = BotContext {
            stack: 1000,
            call_amount: 0,
            current_bet: 0,
            big_blind: 10,
            stage: Stage::Preflop,
            position: 0,
            total_players: 4,
        };

        // Run multiple times to test both check and bet behaviors
        let mut checks = 0;
        let mut bets = 0;
        for _ in 0..100 {
            let action = bot.decide_action(&context);
            match action {
                PlayerAction::CheckCall => checks += 1,
                PlayerAction::Bet(amount) => {
                    bets += 1;
                    // Should bet at least the big blind and not more than stack
                    assert!(amount >= 10);
                    assert!(amount <= 1000);
                }
                PlayerAction::Fold => panic!("Bot should not fold with no bet to call"),
            }
        }

        // Should have both checks and bets in the distribution
        assert!(checks > 0, "Bot should check sometimes when no bet exists");
        assert!(bets > 0, "Bot should bet sometimes when no bet exists");
    }

    #[test]
    fn simple_bot_calls_when_all_in_required() {
        let bot = SimpleBot::default();
        let context = BotContext {
            stack: 50,
            call_amount: 100, // More than stack
            current_bet: 100,
            big_blind: 10,
            stage: Stage::Flop,
            position: 1,
            total_players: 4,
        };

        let action = bot.decide_action(&context);
        assert!(matches!(action, PlayerAction::CheckCall));
    }

    #[test]
    fn bot_manager_generates_actions() {
        let manager = BotManager::new();
        let context = BotContext {
            stack: 500,
            call_amount: 20,
            current_bet: 20,
            big_blind: 10,
            stage: Stage::Turn,
            position: 2,
            total_players: 3,
        };

        let result = manager.generate_action(&context);
        assert!(result.is_ok());

        // Action should be either Fold, CheckCall, or Bet based on probability
        let action = result.unwrap();
        assert!(matches!(
            action,
            PlayerAction::Fold | PlayerAction::CheckCall | PlayerAction::Bet(_)
        ));
    }
}
