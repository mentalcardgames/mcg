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

/// Trait defining the interface for bot AI implementations.
pub trait BotAI {
    /// Decide what action the bot should take given the current context.
    fn decide_action(&self, context: &BotContext) -> PlayerAction;
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

impl BotAI for SimpleBot {
    fn decide_action(&self, context: &BotContext) -> PlayerAction {
        if context.call_amount == 0 {
            // No outstanding bet: make an opening bet of the big blind
            PlayerAction::Bet(context.big_blind)
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
                PlayerAction::CheckCall
            }
        }
    }
}

/// Bot manager that handles bot decision-making and provides the interface
/// between the backend state and bot AI implementations.
pub struct BotManager {
    ai: Box<dyn BotAI + Send + Sync>,
}

impl std::fmt::Debug for BotManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotManager")
            .field("ai", &"<BotAI implementation>")
            .finish()
    }
}

impl Clone for BotManager {
    fn clone(&self) -> Self {
        // Clone by creating a new instance with the same AI type
        // For now, we'll always use SimpleBot for clones
        Self {
            ai: Box::new(SimpleBot::default()),
        }
    }
}

impl BotManager {
    /// Create a new bot manager with the default SimpleBot AI.
    pub fn new() -> Self {
        Self {
            ai: Box::new(SimpleBot::default()),
        }
    }

    /// Create a new bot manager with a custom AI implementation.
    pub fn with_ai(ai: Box<dyn BotAI + Send + Sync>) -> Self {
        Self { ai }
    }

    /// Generate a bot action given the current game context.
    pub fn generate_action(&self, context: &BotContext) -> Result<PlayerAction> {
        let action = self.ai.decide_action(context);
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

impl Default for BotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_bot_bets_with_no_current_bet() {
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

        let action = bot.decide_action(&context);
        assert!(matches!(action, PlayerAction::Bet(10)));
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

        // Action should be either Fold or CheckCall based on probability
        let action = result.unwrap();
        assert!(matches!(
            action,
            PlayerAction::Fold | PlayerAction::CheckCall
        ));
    }
}
