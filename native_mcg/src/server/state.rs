
use crate::game::{Game, Player};
use anyhow::{Context, Result};
use mcg_shared::{Card, CardRank, CardSuit, GameStatePublic, PlayerId};

pub const CHANNEL_BUFFER_SIZE: usize = 256;

// Re-export the canonical types from backend
pub use crate::backend::AppState;
pub use crate::backend::state::Lobby;





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
