//! Texas Hold'em poker engine, hand evaluation, bot AI, and bot driver.

pub mod bot;
pub mod eval;
pub mod game;

#[cfg(feature = "driver")]
pub mod driver;

// Module alias for convenience
pub use eval as poker;

// Re-exports
pub use bot::{BotContext, BotManager, SimpleBot};
pub use eval::{evaluate_best_hand, pick_best_five};
pub use game::{Game, Player};

#[cfg(feature = "driver")]
pub use driver::{pick_delay, run_bot_driver, spawn_bot_driver, BotActionSink};
