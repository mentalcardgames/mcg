// Thin server module facade that re-exports submodules for external use.

pub mod http;
pub mod iroh;
pub mod run;
pub mod state;
pub mod ws;

// Export commonly used types and functions
pub use run::run_server;
pub use state::{broadcast_state, current_state_public, handle_client_msg, AppState};
