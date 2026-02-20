use crate::articles::Post;
use mcg_shared::{GameStatePublic, ServerMsg};
use std::collections::VecDeque;

#[derive(Clone, Default, Debug)]
pub struct ClientSettings {
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
pub struct GameSessionState {
    pub game_state: Option<GameStatePublic>,
}

#[derive(Clone, Debug, Default)]
pub struct ConnectionState {
    pub connection_status: ConnectionStatus,
    pub pending_messages: VecDeque<ServerMsg>,
}

#[derive(Clone, Debug, Default)]
pub struct UIState {
    pub last_error: Option<String>,
    pub last_info: Option<String>,
    pub articles: ArticlesLoading,
    pub pairing_players: Vec<PairingPlayer>,
    pub pairing_confirm_player: Option<String>,
    pub pairing_confirm_action: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct ClientState {
    pub session: GameSessionState,
    pub connection: ConnectionState,
    pub ui: UIState,
    pub settings: ClientSettings,
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientState {
    pub fn new() -> Self {
        let default_settings = ClientSettings {
            name: "Player".to_string(),
            server_address: "127.0.0.1:3000".to_string(),
        };

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

        ClientState {
            settings: default_settings,
            session: GameSessionState { game_state: None },
            connection: ConnectionState {
                connection_status: ConnectionStatus::Disconnected,
                pending_messages: VecDeque::new(),
            },
            ui: UIState {
                last_error: None,
                last_info: None,
                articles: ArticlesLoading::NotStarted,
                pairing_players: players,
                pairing_confirm_player: None,
                pairing_confirm_action: None,
            },
        }
    }

    pub fn queue_server_msg(&mut self, msg: ServerMsg) {
        self.connection.pending_messages.push_back(msg);
    }

    pub fn dispatch_pending_messages(&mut self) {
        while let Some(msg) = self.connection.pending_messages.pop_front() {
            self.apply_server_msg(msg);
        }
    }

    pub fn apply_server_msg(&mut self, msg: ServerMsg) {
        match msg {
            ServerMsg::State(gs) => {
                self.connection.connection_status = ConnectionStatus::Connected;
                self.session.game_state = Some(gs.clone());
                self.ui.last_error = None;
                self.ui.last_info = None;
            }
            ServerMsg::Error(e) => {
                self.ui.last_error = Some(e.clone());
            }
            ServerMsg::Pong => {}
            ServerMsg::QrRes(_content) => {}
        }
    }
}
