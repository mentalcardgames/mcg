use std::{cell::RefCell, rc::Rc};

use crate::articles::Post;
use mcg_shared::{GameStatePublic, ServerMsg};

/// Small, UI-friendly application state.
///
/// This simplified module now exposes a shared, mutable handle:
/// `SharedState = Rc<RefCell<AppState>>`.
///
/// Effects mutate the shared state directly. UI frames should take a cheap
/// snapshot each frame by calling `state.borrow().clone()` (AppState derives Clone)
/// and render from that snapshot. Effects that mutate state should call
/// `ctx.request_repaint()` themselves when they have access to an egui::Context.
#[derive(Clone, Debug, Default)]
pub struct Settings {
    pub name: String,
    pub server_address: String,
    pub bots: usize,
    pub bots_auto: bool,
}

#[derive(Clone, Debug)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}
impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Disconnected
    }
}

#[derive(Clone, Debug)]
pub enum ArticlesLoading {
    NotStarted,
    Loading,
    Loaded(Vec<Post>),
    Error(String),
}
impl Default for ArticlesLoading {
    fn default() -> Self {
        ArticlesLoading::NotStarted
    }
}

/// Pairing UI state moved into the central store so UI widgets remain thin.
/// Keep this minimal: just the player's name and whether they're paired.
#[derive(Clone, Debug)]
pub struct PairingPlayer {
    pub name: String,
    pub paired: bool,
}
impl PairingPlayer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            paired: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub game_state: Option<GameStatePublic>,
    pub last_error: Option<String>,
    pub last_info: Option<String>,
    pub connection_status: ConnectionStatus,
    pub settings: Settings,
    // articles-related state
    pub articles: ArticlesLoading,
    // pairing UI state
    pub pairing_players: Vec<PairingPlayer>,
    pub pairing_confirm_player: Option<String>,
    pub pairing_confirm_action: Option<bool>,
}

/// Shared, mutable handle to application state used across UI and effects.
pub type SharedState = Rc<RefCell<AppState>>;

/// Construct a new SharedState pre-populated with sensible defaults.
pub fn bootstrap_state() -> SharedState {
    let default_settings = Settings {
        name: "Player".to_string(),
        // Default server address — set to a sensible local default here.
        server_address: "127.0.0.1:3000".to_string(),
        bots: 1,
        bots_auto: true,
    };

    // initial pairing players (moved from UI default)
    let players = vec![
        PairingPlayer::new("Alice"),
        PairingPlayer::new("Bob"),
        PairingPlayer::new("Charlie"),
        PairingPlayer::new("David"),
        PairingPlayer::new("Eve"),
        PairingPlayer::new("Frank"),
        PairingPlayer::new("Grace"),
        PairingPlayer::new("Heidi"),
        PairingPlayer::new("Ivan"),
        PairingPlayer::new("Julia"),
        PairingPlayer::new("Kevin"),
        PairingPlayer::new("Laura"),
        PairingPlayer::new("Michael"),
        PairingPlayer::new("Natalie"),
        PairingPlayer::new("Oscar"),
        PairingPlayer::new("Patricia"),
    ];

    let state = AppState {
        settings: default_settings,
        pairing_players: players,
        pairing_confirm_player: None,
        pairing_confirm_action: None,
        ..Default::default()
    };

    Rc::new(RefCell::new(state))
}

/// Helper to apply an incoming ServerMsg into the shared AppState.
/// Effects may call this helper while holding the appropriate repaint context.
pub fn apply_server_msg(state: &SharedState, msg: ServerMsg) {
    let mut s = state.borrow_mut();
    match msg {
        ServerMsg::Welcome { .. } => {
            s.connection_status = ConnectionStatus::Connected;
            s.last_info = Some("Connected".into());
            s.last_error = None;
        }
        ServerMsg::State(gs) => {
            s.game_state = Some(gs.clone());
            s.last_info = None;
        }
        ServerMsg::Error(e) => {
            s.last_error = Some(e.clone());
        }
    }
}
