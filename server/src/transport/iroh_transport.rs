use anyhow::Result;
use iroh::endpoint::Endpoint;
use iroh::protocol::Router;
use std::path::PathBuf;
use std::sync::Arc;

use crate::transport::Transport;
use mcg_shared::{ClientMsg, ServerMsg};

/// Minimal Iroh transport: only create an Endpoint and expose node_id for now.
/// Full blobs and message protocols will be added incrementally.
pub struct IrohTransport {
    endpoint: Option<Endpoint>,
    // Optional callback storage for incoming ClientMsg from protocol handler
    on_client: std::sync::Arc<tokio::sync::Mutex<Option<std::sync::Arc<dyn Fn(String, ClientMsg) + Send + Sync>>>>,
}

impl IrohTransport {
    pub fn new() -> Self {
        Self { endpoint: None, on_client: std::sync::Arc::new(tokio::sync::Mutex::new(None)) }
    }
}

#[async_trait::async_trait]
impl Transport for IrohTransport {
    async fn start(&mut self) -> Result<()> {
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;

        // Register our MsgProtocol handler so incoming connections are handled and
        // messages are forwarded to the on_client callback if set.
        let proto = crate::transport::msg_protocol::MsgProtocol::new(self.on_client.clone());
        // The iroh endpoint's Router is created via Router::builder(endpoint).spawn().
        // Create and spawn a router accepting our ALPN and handler so incoming connections
        // with that ALPN are dispatched to our MsgProtocol handler.
        let router = Router::builder(endpoint.clone())
            .accept(b"/mcg/msg/1", proto)
            .spawn();

        // Router takes ownership of the endpoint internally; stash the endpoint returned by router.endpoint().
        self.endpoint = Some(router.endpoint().clone());
        // Keep the router alive by dropping into a background task handle stored on the endpoint by Router.
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

    async fn send_message(&self, peer: Option<String>, _msg: &ServerMsg) -> Result<()> {
        // Per-current implementation constraints, sending a directed message to a
        // specific iroh peer is not implemented. We accept messages from transports
        // (via the registered protocol handler) but do not attempt outgoing dials here.
        if peer.is_some() {
            return Err(anyhow::anyhow!("send_message to a specific iroh peer is not implemented"));
        }
        // For peer=None (broadcast) there's no simple broadcast API on Endpoint; do nothing.
        Ok(())
    }

    fn set_on_client_message(&mut self, cb: Box<dyn Fn(String, ClientMsg) + Send + Sync>) {
        // Store the callback so the MsgProtocol handler can call it on incoming messages.
        let cb_arc: std::sync::Arc<dyn Fn(String, ClientMsg) + Send + Sync> = std::sync::Arc::from(cb);
        let on_client = self.on_client.clone();
        // Spawn a task to set it asynchronously
        tokio::spawn(async move {
            let mut guard = on_client.lock().await;
            *guard = Some(cb_arc);
        });
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
