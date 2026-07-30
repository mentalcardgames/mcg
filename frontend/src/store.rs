use crate::screens::articles_screen::Post;

#[derive(Clone, Default, Debug)]
pub struct ClientSettings {
    pub name: String,
    pub server_address: String,
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
}
