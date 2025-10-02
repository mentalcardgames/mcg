// Server state management: AppState, Lobby, and helpers that operate on shared state.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use mcg_shared::{Card, CardRank, CardSuit, PlayerId};
// rand import removed; use rand::random::<f64>() for probabilistic decisions
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::bot::BotManager;
use crate::game::{Game, Player};
use crate::pretty;
use mcg_shared::GameStatePublic;

use crate::server::state::CHANNEL_BUFFER_SIZE;

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
        let (tx, _rx) = broadcast::channel(16);
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
    pub(crate) bots: Vec<PlayerId>,
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
        let gs = game.public();
        Some(gs)
    } else {
        None
    }
}

/// Broadcast the current state (and print new events to server console) to all subscribers.
///
/// Transports receive the same `ServerMsg::State` payload; the backend does not
/// embed per-connection personalization in the broadcast. If transports or a
/// future session manager needs to expose client-specific views, they should
/// compute those on the transport/session layer.
pub async fn broadcast_state(state: &AppState) {
    if let Some(gs) = current_state_public(state).await {
        // Print any newly added events to server console and update bookkeeping.
        let mut lobby = state.lobby.write().await;
        let already = lobby.last_printed_log_len;
        let total = gs.action_log.len();
        if total > already {
            for e in gs.action_log.iter().skip(already) {
                // Use the provided viewer id when formatting server-side logs.
                let line =
                    pretty::format_event_human(e, &gs.players, std::io::stdout().is_terminal());
                tracing::info!(%line);
            }
            lobby.last_printed_log_len = total;
        }
        drop(lobby);

        // Broadcast the new state to all subscribers.
        let _ = state.broadcaster.send(mcg_shared::ServerMsg::State(gs));
    }
}

/// Apply an action to the game's state. Returns Some(error_string) if the
/// underlying Game::apply_player_action returned an error, otherwise None.
pub async fn apply_action_to_game(
    state: &AppState,
    actor: usize,
    action: mcg_shared::PlayerAction,
) -> Option<String> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        if let Err(e) = game.apply_player_action(actor, action) {
            return Some(e.to_string());
        }
    }
    None
}

/// Validate that the provided player_id is currently allowed to take an action
/// and apply the action. Returns Ok(()) on success or Err(String) with an error
/// message to send back to the client.
pub async fn validate_and_apply_action(
    state: &AppState,
    player_id: PlayerId,
    action: mcg_shared::PlayerAction,
) -> Result<(), String> {
    // First, ensure a game exists
    {
        let lobby_r = state.lobby.read().await;
        if lobby_r.game.is_none() {
            return Err("No active game. Please start a new game first.".into());
        }
    }

    // Resolve provided player_id to the internal player index used by Game
    let actor_idx = {
        let lobby_r = state.lobby.read().await;
        if let Some(game) = &lobby_r.game {
            match game.players.iter().position(|p| p.id == player_id) {
                Some(idx) => idx,
                None => return Err("Unknown player id".into()),
            }
        } else {
            return Err("No active game. Please start a new game first.".into());
        }
    };

    // Ensure the requested player is allowed to act (compare against index)
    let allowed = {
        let lobby_r = state.lobby.read().await;
        if let Some(game) = &lobby_r.game {
            game.stage != mcg_shared::Stage::Showdown && game.to_act == actor_idx
        } else {
            false
        }
    };
    if !allowed {
        return Err("Not your turn".into());
    }

    // Apply the action using the existing helper. translate underlying errors to String.
    if let Some(e) = apply_action_to_game(state, actor_idx, action).await {
        return Err(e);
    }
    Ok(())
}

/// Broadcast the current state (and trigger bots if enabled).
pub async fn broadcast_and_drive(state: &AppState) {
    // Broadcast updated state to subscribers.
    broadcast_state(state).await;
    // Drive bots using the server module implementation
    let config = state.config.read().await;
    let (min_ms, max_ms) = config.bot_delay_range();
    crate::server::bot_driver::drive_bots_with_delays(state, min_ms, max_ms).await;
}

/// Handle an Action message from a client
async fn handle_action(
    state: &AppState,
    player_id: PlayerId,
    action: mcg_shared::PlayerAction,
) -> mcg_shared::ServerMsg {
    match validate_and_apply_action(state, player_id, action.clone()).await {
        Ok(()) => {
            broadcast_and_drive(state).await;
            if let Some(gs) = current_state_public(state).await {
                mcg_shared::ServerMsg::State(gs)
            } else {
                mcg_shared::ServerMsg::Error("No active game after action".into())
            }
        }
        Err(e) => mcg_shared::ServerMsg::Error(e.to_string()),
    }
}

/// Handle a RequestState message from a client
async fn handle_request_state(state: &AppState) -> mcg_shared::ServerMsg {
    if let Some(gs) = current_state_public(state).await {
        broadcast_and_drive(state).await;
        mcg_shared::ServerMsg::State(gs)
    } else {
        mcg_shared::ServerMsg::Error("No active game. Please start a new game first.".into())
    }
}

/// Handle a NextHand message from a client
async fn handle_next_hand(state: &AppState) -> mcg_shared::ServerMsg {
    // Ensure a game exists first
    {
        let lobby_r = state.lobby.read().await;
        if lobby_r.game.is_none() {
            return mcg_shared::ServerMsg::Error(
                "No active game. Please start a new game first.".into(),
            );
        }
    }

    match start_new_hand_and_print(state).await {
        Ok(()) => {
            broadcast_and_drive(state).await;
            if let Some(gs) = current_state_public(state).await {
                mcg_shared::ServerMsg::State(gs)
            } else {
                mcg_shared::ServerMsg::Error("No active game after starting next hand".into())
            }
        }
        Err(e) => mcg_shared::ServerMsg::Error(format!("Failed to start new hand: {}", e)),
    }
}

/// Handle a NewGame message from a client
async fn handle_new_game(
    state: &AppState,
    players: Vec<mcg_shared::PlayerConfig>,
) -> mcg_shared::ServerMsg {
    match create_new_game(state, players).await {
        Ok(()) => {
            broadcast_and_drive(state).await;
            if let Some(gs) = current_state_public(state).await {
                mcg_shared::ServerMsg::State(gs)
            } else {
                mcg_shared::ServerMsg::Error(
                    "Failed to produce initial state after creating game".into(),
                )
            }
        }
        Err(e) => mcg_shared::ServerMsg::Error(format!("Failed to create new game: {}", e)),
    }
}

/// Unified handler for ClientMsg coming from any transport.
///
/// Centralizes validation, state mutation, and side-effects (broadcasting and
/// bot-driving). Returns a ServerMsg that the originating transport should send
/// back to the client. Transports should delegate to this function rather than
/// duplicating handling logic to ensure consistent behavior across transports.
pub async fn handle_client_msg(
    state: &AppState,
    cm: mcg_shared::ClientMsg,
) -> mcg_shared::ServerMsg {
    match cm {
        mcg_shared::ClientMsg::Action { player_id, action } => {
            handle_action(state, player_id, action).await
        }
        mcg_shared::ClientMsg::RequestState => handle_request_state(state).await,
        mcg_shared::ClientMsg::NextHand => handle_next_hand(state).await,
        mcg_shared::ClientMsg::NewGame { players } => handle_new_game(state, players).await,
    }
}

/// Advance to the next hand (increment dealer, start a new hand) and print a table header.
pub async fn start_new_hand_and_print(state: &AppState) -> Result<()> {
    let mut lobby = state.lobby.write().await;
    if let Some(game) = &mut lobby.game {
        let n = game.players.len();
        if n > 0 {
            game.dealer_idx = (game.dealer_idx + 1) % n;
        }
        game.start_new_hand()?;
        let sb = game.sb;
        let bb = game.bb;
        // start_new_hand_and_print runs in server-side context
        // for printing the table header and tracking printed log length.
        let gs = game.public();
        lobby.last_printed_log_len = gs.action_log.len();
        let header = pretty::format_table_header(&gs, sb, bb, std::io::stdout().is_terminal());
        tracing::info!("{}", header);
    }
    Ok(())
}


