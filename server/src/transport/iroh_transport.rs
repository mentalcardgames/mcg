use anyhow::Result;
use iroh::endpoint::Endpoint;
use iroh_base::{NodeAddr, NodeId};
use iroh::protocol::Router;
use std::path::PathBuf;

use crate::transport::Transport;
use mcg_shared::{ClientMsg, ServerMsg};

// Type alias for the on-client callback storage to reduce type complexity warnings from clippy.
type IrohOnClientCallback = std::sync::Arc<tokio::sync::Mutex<Option<std::sync::Arc<dyn Fn(String, ClientMsg) + Send + Sync>>>>;

/// Minimal Iroh transport: only create an Endpoint and expose node_id for now.
/// Full blobs and message protocols will be added incrementally.
pub struct IrohTransport {
    endpoint: Option<Endpoint>,
    // Optional callback storage for incoming ClientMsg from protocol handler
    // Callback storage for incoming ClientMsg. Use a type alias for readability.
    on_client: IrohOnClientCallback,
}

impl IrohTransport {
    pub fn new() -> Self {
        Self { endpoint: None, on_client: IrohOnClientCallback::default() }
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

    async fn send_message(&self, peer: Option<String>, msg: &ServerMsg) -> Result<()> {
        // If peer is None, per current design we do nothing (no broadcast).
        if peer.is_none() {
            return Ok(());
        }

        let pid = peer.unwrap();
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("iroh endpoint not initialized"))?;

        // Try to connect to the remote node by id and open a bi-directional stream.
        // The exact iroh API may vary; attempt to use Endpoint::connect which accepts a
        // node id/address type that implements FromStr. Map errors into anyhow for simplicity.
        // We perform a best-effort implementation and return any underlying errors.
        // Note: this will be compiled and adjusted if the iroh API requires different types.
        // Parse peer id into NodeId and build a NodeAddr for dialing. We don't have relay or addrs so
        // use NodeAddr::new(node_id) which contains only the NodeId; discovery will be used to find paths.
        let node_id: NodeId = pid
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid node id: {}", e))?;
        let node_addr = NodeAddr::new(node_id);

        let conn = endpoint
            .connect(node_addr, b"/mcg/msg/1")
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // Open a bi-directional stream and write framed JSON payload
        let (mut send, _recv) = conn
            .open_bi()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // Serialize and frame
        let bytes = serde_json::to_vec(msg)?;
        let framed = crate::transport::framing::encode_frame(&bytes);

        // Write all and finish
        use tokio::io::AsyncWriteExt;
        send.write_all(&framed).await.map_err(|e| anyhow::anyhow!(e))?;
        // finish() is synchronous in this iroh version
        send.finish().map_err(|e| anyhow::anyhow!(e))?;

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
