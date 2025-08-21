// Thin server module facade that re-exports submodules for external use.

pub mod http;
pub mod iroh;
pub mod run;
pub mod state;
pub mod ws;

// Re-export commonly used items so other modules can continue to reference
// `crate::backend::AppState`, `crate::backend::broadcast_state`, etc.
pub use run::run_server;
pub use state::{broadcast_state, current_state_public, handle_client_msg, AppState};
