//! UI module for engine-tui

pub mod controls;
pub mod game_state;
pub mod input;
pub mod layout;
pub mod state;
pub mod trace_log;

pub use controls::ControlsPanel;
pub use game_state::GameStatePanel;
pub use input::InputPanel;
pub use layout::AppLayout;
pub use state::{PanelFocus, TuiState};
pub use trace_log::TraceLogPanel;
