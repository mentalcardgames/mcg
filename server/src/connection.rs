use async_trait::async_trait;
use anyhow::Result;
use mcg_shared::ServerMsg;

/// Minimal sink abstraction used by server-side connection handlers to send ServerMsg
/// back to clients. The read side (receiving ClientMsg) is handled by the transport-specific
/// accept loops which call process_client_msg_generic when a ClientMsg is received.
#[async_trait]
pub trait ServerSink: Send + Sync {
    async fn send(&mut self, msg: &ServerMsg) -> Result<()>;
}
