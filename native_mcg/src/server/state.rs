use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::bot::BotManager;
use crate::game::{Game, Player};
use anyhow::{Context, Result};
use mcg_shared::{Card, CardRank, CardSuit, GameStatePublic, PlayerId};

pub const CHANNEL_BUFFER_SIZE: usize = 256;

/// Shared application state exposed to handlers.
#[derive(Clone)]
pub struct AppState {
    pub(crate) lobby: Arc<RwLock<Lobby>>,
    pub broadcaster: broadcast::Sender<mcg_shared::ServerMsg>,
    /// In-memory shared Config instance. Holds the authoritative configuration
    /// for the running server. Use tokio::sync::RwLock for concurrent access.
    pub config: std::sync::Arc<RwLock<crate::config::Config>>,
    /// Optional path to the TOML config file used by the running server.
    /// If present, transports (e.g. iroh) may persist changes to this path.
    pub config_path: Option<PathBuf>,
}

impl AppState {
    /// Create a new AppState with the given config and optional config path
    pub fn new(config: crate::config::Config, config_path: Option<PathBuf>) -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_BUFFER_SIZE);
        Self {
            lobby: Arc::new(RwLock::new(Lobby::default())),
            broadcaster: tx,
            config: std::sync::Arc::new(RwLock::new(config)),
            config_path,
        }
    }
}

#[derive(Clone)]
pub struct Lobby {
    pub(crate) game: Option<Game>,
    pub(crate) last_printed_log_len: usize,
    /// List of player IDs that should be driven by bots. Kept in the backend so
    /// the game engine remains unaware of bot status.
    pub(crate) bots: Vec<mcg_shared::PlayerId>,
    /// Indicates whether a server-side turn-driving loop is currently running.
    /// Prevents concurrent drive loops from multiple transports.
    pub(crate) driving: bool,
    /// Bot manager for AI decision making
    pub(crate) bot_manager: BotManager,
}

#[allow(clippy::derivable_impls)]
impl Default for Lobby {
    fn default() -> Self {
        Self {
            game: None,
            last_printed_log_len: 0,
            bots: Vec::new(),
            driving: false,
            bot_manager: BotManager::default(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(CHANNEL_BUFFER_SIZE);
        AppState {
            lobby: Arc::new(RwLock::new(Lobby::default())),
            broadcaster: tx,
            config: std::sync::Arc::new(RwLock::new(crate::config::Config::default())),
            config_path: None,
        }
    }
}

/// Create a new game with the specified players.
pub async fn create_new_game(
    state: &AppState,
    players: Vec<mcg_shared::PlayerConfig>,
) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    let player_count = players.len();

    // Convert PlayerConfig to internal Player format. The engine's Player type
    // is agnostic about bot status; the backend tracks bot-driven IDs separately.
    let mut game_players = Vec::new();
    let mut bot_ids: Vec<PlayerId> = Vec::new();
    for config in &players {
        if config.is_bot {
            bot_ids.push(config.id);
        }
        let player = Player {
            id: config.id,
            name: config.name.clone(),
            stack: 1000, // Default stack size
            cards: [
                Card::new(CardRank::Ace, CardSuit::Clubs),
                Card::new(CardRank::Ace, CardSuit::Clubs),
            ], // Empty cards initially
            has_folded: false,
            all_in: false,
        };
        game_players.push(player);
    }
    // Store bot ids on the lobby so backend drive logic can consult it.
    lobby.bots = bot_ids;

    // Create the game with the players
    let game = Game::with_players(game_players)
        .with_context(|| "creating new game with specified players")?;

    lobby.game = Some(game);
    tracing::info!(player_count = player_count, "created new game");

    Ok(())
}

pub async fn current_state_public(state: &AppState) -> Option<GameStatePublic> {
    let lobby_r = state.lobby.read().await;
    if let Some(game) = &lobby_r.game {
        let gs = game.public_all();
        Some(gs)
    } else {
        None
    }
}
