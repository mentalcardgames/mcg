// Lobby management functionality
// Currently integrated with state.rs, but separated for future expansion

use super::state::AppState;

/// Lobby management functions
pub struct LobbyManager;

impl LobbyManager {
    /// Get a reference to the lobby
    pub async fn get_lobby(
        state: &AppState,
    ) -> tokio::sync::RwLockReadGuard<'_, super::state::Lobby> {
        state.lobby.read().await
    }

    /// Get a mutable reference to the lobby
    pub async fn get_lobby_mut(
        state: &AppState,
    ) -> tokio::sync::RwLockWriteGuard<'_, super::state::Lobby> {
        state.lobby.write().await
    }
}
