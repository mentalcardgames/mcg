// Thin server module facade that re-exports submodules for external use.

pub mod run;
pub mod ws;
pub mod state;

// Re-export commonly used items so other modules can continue to reference
// `crate::server::AppState`, `crate::server::broadcast_state`, etc.
pub use state::{
    AppState, ensure_game_started, broadcast_state, drive_bots_with_delays, current_state_public,
    apply_action_to_game, start_new_hand_and_print, reset_game_with_bots,
};
pub use run::{build_router, run_server};
