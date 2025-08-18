// Thin server module facade that re-exports submodules for external use.

pub mod http;
pub mod run;
pub mod state;
pub mod ws;

// Re-export commonly used items so other modules can continue to reference
// `crate::backend::AppState`, `crate::backend::broadcast_state`, etc.
pub use run::run_server;
pub use state::{
    apply_action_to_game, broadcast_state, current_state_public, drive_bots_with_delays,
    ensure_game_started, reset_game_with_bots, start_new_hand_and_print, AppState,
};
