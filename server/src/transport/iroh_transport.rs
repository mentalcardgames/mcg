use anyhow::Result;
use iroh::endpoint::Endpoint;
use iroh_base::{NodeAddr, NodeId};
use iroh::protocol::Router;
use futures_util::StreamExt;
use iroh::Watcher;
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
    debug: bool,
}

impl IrohTransport {
    /// Create a new IrohTransport. Pass `debug=true` to enable verbose iroh event logging.
    pub fn new(debug: bool) -> Self {
        Self { endpoint: None, on_client: IrohOnClientCallback::default(), debug }
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
        // When debug is enabled, request additional logging from iroh components
        if self.debug {
            // Note: endpoint.set_alpns is available; use endpoint.metrics or other hooks if needed.
            eprintln!("[IROH] starting endpoint in debug mode");
        }

        let router = Router::builder(endpoint.clone())
            .accept(b"/mcg/msg/1", proto)
            .spawn();

        // Router takes ownership of the endpoint internally; stash the endpoint returned by router.endpoint().
        self.endpoint = Some(router.endpoint().clone());

        // Spawn a task to watch for the endpoint's node_addr being initialized and report it.
        // This always reports a concise published status; when debug=true we also stream discovery events.
        if let Some(ep) = &self.endpoint {
            let ep = ep.clone();
            let debug = self.debug;
            tokio::spawn(async move {
                // print node_addr when it is initialized (indicates addresses were published)
                let node_addr = ep.node_addr().initialized().await;
                println!("[IROH] node_addr initialized: {:?}", node_addr);

                // print home_relay when available (print whatever the API returns)
                let home_relay = ep.home_relay().initialized().await;
                println!("[IROH] home_relay: {:?}", home_relay);

                // If debug was requested, stream verbose discovery events to stderr
                if debug {
                    eprintln!("[IROH DEBUG] starting discovery event stream");
                    let mut ds = ep.discovery_stream();
                    while let Some(item) = ds.next().await {
                        match item {
                            Ok(di) => eprintln!("[IROH DEBUG] discovery item: {:?}", di),
                            Err(e) => eprintln!("[IROH DEBUG] discovery error: {:?}", e),
                        }
                    }
                }
            });
        }

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
