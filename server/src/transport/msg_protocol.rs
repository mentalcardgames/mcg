use async_trait::async_trait;
use iroh::protocol::{ProtocolHandler, AcceptError};
use iroh::endpoint::Connection;

// Temporary stub: no-op ProtocolHandler so router registration compiles.
#[derive(Debug, Clone)]
pub struct MsgProtocol;

impl MsgProtocol {
    pub fn new() -> Self {
        MsgProtocol
    }
}

#[async_trait]
impl ProtocolHandler for MsgProtocol {
    async fn accept(&self, _connection: Connection) -> Result<(), AcceptError> {
        // no-op for now; full message protocol will be implemented later
        Ok(())
    }
}