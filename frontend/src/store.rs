use crate::articles::Post;
use mcg_shared::{GameStatePublic, ServerMsg};
use std::collections::VecDeque;

#[derive(Clone, Default, Debug)]
pub struct Settings {
    pub name: String,
    pub server_address: String,
}

#[derive(Clone, Debug, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Clone, Debug, Default)]
pub enum ArticlesLoading {
    #[default]
    NotStarted,
    Loading,
    Loaded(Vec<Post>),
    Error(String),
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

#[derive(Clone, Debug)]
pub struct AppState {
    pub game_state: Option<GameStatePublic>,
    pub last_error: Option<String>,
    pub last_info: Option<String>,
    pub connection_status: ConnectionStatus,
    pub settings: Settings,
    /// Queue of incoming server messages to be processed on the main thread
    pub pending_messages: VecDeque<ServerMsg>,
    // articles-related state
    pub articles: ArticlesLoading,
    // pairing UI state
    pub pairing_players: Vec<PairingPlayer>,
    pub pairing_confirm_player: Option<String>,
    pub pairing_confirm_action: Option<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let default_settings = Settings {
            name: "Player".to_string(),
            // Default server address — set to a sensible local default here.
            server_address: "127.0.0.1:3000".to_string(),
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

        AppState {
            settings: default_settings,
            pairing_players: players,
            pairing_confirm_player: None,
            pairing_confirm_action: None,
            game_state: None,
            last_error: None,
            last_info: None,
            connection_status: ConnectionStatus::Disconnected,
            articles: ArticlesLoading::NotStarted,
            pending_messages: VecDeque::new(),
        }
    }

    /// Queue a server message to be processed on the main thread
    pub fn queue_server_msg(&mut self, msg: ServerMsg) {
        self.pending_messages.push_back(msg);
    }

    /// Process all pending server messages
    pub fn process_pending_messages(&mut self) {
        while let Some(msg) = self.pending_messages.pop_front() {
            self.apply_server_msg(msg);
        }
    }

    /// Helper to apply an incoming ServerMsg into the shared AppState.
    /// Effects may call this helper while holding the appropriate repaint context.
    pub fn apply_server_msg(&mut self, msg: ServerMsg) {
        match msg {
            ServerMsg::State(gs) => {
                self.connection_status = ConnectionStatus::Connected;
                self.game_state = Some(gs.clone());
                self.last_error = None;
                self.last_info = None;
            }
            ServerMsg::Error(e) => {
                self.last_error = Some(e.clone());
            }
            ServerMsg::Pong => {}
            ServerMsg::QrRes(content) => {}
        }
    }
}
