// Thin server module facade that re-exports submodules for external use.

pub mod http;
pub mod run;
pub mod state;
pub mod ws;

// Re-export commonly used items so other modules can continue to reference
// `crate::backend::AppState`, `crate::backend::broadcast_state`, etc.
pub use run::run_server;
pub use state::{
    broadcast_and_drive, broadcast_state, create_new_game, current_state_public,
    start_new_hand_and_print, validate_and_apply_action, handle_client_msg, AppState,
};
