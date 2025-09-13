// Server state management: AppState, Lobby, and helpers that operate on shared state.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use mcg_shared::{Card, PlayerId};
// rand import removed; use rand::random::<f64>() for probabilistic decisions
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::bot::{BotContext, BotManager};
use crate::game::{Game, Player};
use crate::pretty;
use mcg_shared::GameStatePublic;

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
    pub fn new(
        config: crate::config::Config,
        config_path: Option<PathBuf>,
    ) -> Self {
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
pub(crate) struct Lobby {
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
        let (tx, _rx) = broadcast::channel(16);
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
            stack: 1000,               // Default stack size
            cards: [Card(0), Card(0)], // Empty cards initially
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
    // Drive bots (drive_bots_with_delays itself respects lobby.bots_auto).
    let config = state.config.read().await;
    let (min_ms, max_ms) = config.bot_delay_range();
    drive_bots_with_delays(state, min_ms, max_ms).await;
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
        mcg_shared::ServerMsg::Error(
            "No active game. Please start a new game first.".into(),
        )
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
                mcg_shared::ServerMsg::Error(
                    "No active game after starting next hand".into(),
                )
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
        mcg_shared::ClientMsg::RequestState => {
            handle_request_state(state).await
        }
        mcg_shared::ClientMsg::NextHand => {
            handle_next_hand(state).await
        }
        mcg_shared::ClientMsg::NewGame { players } => {
            handle_new_game(state, players).await
        }
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

/// Drive bots similarly to the websocket handler, but mutate shared state and
/// broadcast resulting states. Exposed so iroh transport can reuse the same behaviour.
pub async fn drive_bots_with_delays(state: &AppState, min_ms: u64, max_ms: u64) {
    // Ensure only one drive loop runs at a time.
    {
        let mut lobby = state.lobby.write().await;
        if lobby.driving {
            return;
        }
        lobby.driving = true;
    }

    // Drive loop: keep acting while bots_auto is enabled and the player to act is a bot.
    loop {
        // Exit if bots_auto disabled or no game present.
        {
            let lobby_r = state.lobby.read().await;
            if lobby_r.game.is_none() {
                break;
            }
        }

        // Determine current actor and whether it's a bot.
        let maybe_actor = {
            let lobby_r = state.lobby.read().await;
            if let Some(game) = &lobby_r.game {
                if game.stage == mcg_shared::Stage::Showdown {
                    None
                } else {
                    let idx = game.to_act;
                    game.players.get(idx).and_then(|p| {
                        if lobby_r.bots.contains(&p.id) {
                            Some((idx, p.id))
                        } else {
                            None
                        }
                    })
                }
            } else {
                None
            }
        };

        let (actor_idx, actor_id) = match maybe_actor {
            Some(v) => v,
            None => break, // no bot to act now
        };

        // Apply bot action under write lock, but re-check to_act to avoid races.
        let applied_and_advanced = {
            let mut lobby_w = state.lobby.write().await;
            
            // Clone the bot manager first to avoid borrowing conflicts
            let bot_manager = lobby_w.bot_manager.clone();
            
            if let Some(game) = &mut lobby_w.game {
                // Reconfirm conditions haven't changed.
                if game.stage == mcg_shared::Stage::Showdown || game.to_act != actor_idx {
                    // Another actor changed the state; resume loop to re-evaluate.
                    false
                } else {
                    // Generate bot action using the bot manager
                    let need = game.current_bet.saturating_sub(game.round_bets[actor_idx]);
                    let context = BotContext {
                        stack: game.players[actor_idx].stack,
                        call_amount: need,
                        current_bet: game.current_bet,
                        big_blind: game.bb,
                        stage: game.stage,
                        position: actor_idx,
                        total_players: game.players.len(),
                    };
                    
                    let action = match bot_manager.generate_action(&context) {
                        Ok(action) => action,
                        Err(e) => {
                            tracing::error!("Bot manager failed to generate action: {}", e);
                            // Fallback to a safe action
                            if need == 0 {
                                mcg_shared::PlayerAction::CheckCall
                            } else {
                                mcg_shared::PlayerAction::Fold
                            }
                        }
                    };

                    // With the refactored flow logic, we are more confident that
                    // the game state will advance correctly. The check for whether
                    // the actor changed is removed to simplify the bot driver.
                    match game.apply_player_action(actor_idx, action) {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!(
                                "[BOT] failed to apply action for bot id {}: {}",
                                actor_id, e
                            );
                            false
                        }
                    }
                }
            } else {
                false
            }
        };

        // Broadcast updated state using the acting player's id (viewer-specific state)
        // Always broadcast the state resulting from the attempted bot action so clients
        // observe the latest changes even if the bot's action didn't advance the turn.
        crate::backend::broadcast_state(state).await;

        if !applied_and_advanced {
            // Stop driving bots for now; give control back to caller/human/other logic.
            break;
        }

        // Sleep between bot actions (randomized within given bounds)
        let now_ns = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.subsec_nanos() as u64,
            Err(_) => 0u64,
        };
        let span = max_ms.saturating_sub(min_ms);
        let delay = min_ms + (now_ns % span.max(1));
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }

    // Clear driving flag
    let mut lobby = state.lobby.write().await;
    lobby.driving = false;
}
