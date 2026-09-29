use anyhow::Result;
use mcg_shared::{PlayerAction, PlayerId};

/// Abstraction for submitting bot actions to an event loop or controller.
#[async_trait::async_trait]
pub trait BotActionSink: Send + Sync + 'static {
    async fn send_bot_action(&self, player_id: PlayerId, action: PlayerAction) -> Result<()>;
}
