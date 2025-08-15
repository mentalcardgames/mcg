use anyhow::Result;
use iroh::endpoint::Endpoint;
use std::path::PathBuf;
use std::sync::Arc;

use crate::transport::Transport;
use mcg_shared::{ClientMsg, ServerMsg};

/// Minimal Iroh transport: only create an Endpoint and expose node_id for now.
/// Full blobs and message protocols will be added incrementally.
pub struct IrohTransport {
    endpoint: Option<Endpoint>,
}

impl IrohTransport {
    pub fn new() -> Self {
        Self { endpoint: None }
    }
}

#[async_trait::async_trait]
impl Transport for IrohTransport {
    async fn start(&mut self) -> Result<()> {
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        self.endpoint = Some(endpoint);
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(endpoint) = self.endpoint.take() {
            endpoint.close().await;
        }
        Ok(())
    }

    fn node_id(&self) -> Option<String> {
        self.endpoint.as_ref().map(|e| e.node_id().to_string())
    }

    async fn send_message(&self, peer: Option<String>, msg: &ServerMsg) -> Result<()> {
        // Basic framed JSON message send using transport's connection API.
        // For now, implement a best-effort broadcast to all connected peers via the endpoint.
        if self.endpoint.is_none() {
            return Err(anyhow::anyhow!("iroh endpoint not started"));
        }
        let ep = self.endpoint.as_ref().unwrap();
        let txt = serde_json::to_string(msg)?;
        // Encode length-prefixed frame
        let frame = crate::transport::framing::encode_frame(txt.as_bytes());
        if let Some(pid) = peer {
            // Attempt to open a connection and send the frame
            if let Ok(mut conn) = ep.connect(&pid).await {
                let _ = conn.write(&frame).await;
                let _ = conn.close().await;
            }
        } else {
            // No peer specified: try to send via discovery peers (best-effort)
            // Endpoint doesn't provide a built-in broadcast; so we skip for now.
        }
        Ok(())
    }

    fn set_on_client_message(&mut self, _cb: Box<dyn Fn(String, ClientMsg) + Send + Sync>) {
        // not implemented yet - message accept/handler requires protocol plumbing
    }

    async fn advertise_blob(&self, path: PathBuf) -> Result<String> {
        // Implement simple blake3 hashing and return hex of file contents as an "advertised id".
        let data = tokio::fs::read(&path).await?;
        let hash = blake3::hash(&data);
        Ok(hex::encode(hash.as_bytes()))
    }

    async fn fetch_blob(
        &self,
        _hash: &str,
        _node_id: Option<&str>,
        _out_path: PathBuf,
    ) -> Result<()> {
        // Fetching via iroh blobs is not implemented yet; return an error explaining that.
        Err(anyhow::anyhow!(
            "fetch_blob not implemented for iroh transport yet"
        ))
    }
}
