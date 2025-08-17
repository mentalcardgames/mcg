use anyhow::Result;
use futures_util::StreamExt;
use iroh::endpoint::Endpoint;
use iroh::protocol::Router;
use iroh::Watcher;
use iroh_base::{NodeAddr, NodeId};
use std::path::PathBuf;

use crate::transport::Transport;
use mcg_shared::{ClientMsg, ServerMsg};

use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

/// Minimal Iroh transport: only create an Endpoint and expose node_id for now.
/// Full blobs and message protocols will be added incrementally.
pub struct IrohTransport {
    endpoint: Endpoint,
    // debug flag retained for optional verbose discovery streaming
    debug: bool,
}

impl IrohTransport {
    /// Create and initialize a new IrohTransport. Returns the transport and a receiver
    /// which yields (peer_node_id, ClientMsg) tuples parsed by the protocol handler.
    pub async fn new(debug: bool) -> Result<(Self, UnboundedReceiver<(String, ClientMsg)>)> {
        if debug {
            eprintln!("[IROH] creating endpoint in debug mode");
        }

        // Build endpoint
        let endpoint = Endpoint::builder()
            .alpns(vec![b"/mcg/msg/1".to_vec()])
            .discovery_n0()
            .bind()
            .await?;

        // Create an unbounded channel to receive parsed messages from the protocol handler
        let (tx, rx) = unbounded_channel::<(String, mcg_shared::ClientMsg)>();
        let proto = crate::transport::msg_protocol::MsgProtocol::new(tx.clone());

        let router = Router::builder(endpoint.clone())
            .accept(b"/mcg/msg/1", proto)
            .spawn();
        let endpoint = router.endpoint().clone();

        // Wait for initialization of node_addr/home_relay so callers can rely on published info
        endpoint.node_addr().initialized().await;
        let _ = endpoint.home_relay().initialized().await;

        // Spawn discovery stream if debug
        if debug {
            let ep_debug = endpoint.clone();
            tokio::spawn(async move {
                eprintln!("[IROH DEBUG] starting discovery event stream");
                let mut ds = ep_debug.discovery_stream();
                while let Some(item) = ds.next().await {
                    match item {
                        Ok(di) => eprintln!("[IROH DEBUG] discovery item: {:?}", di),
                        Err(e) => eprintln!("[IROH DEBUG] discovery error: {:?}", e),
                    }
                }
            });
        } else {
            // print concise published node_addr/home_relay once
            let ep_once = endpoint.clone();
            tokio::spawn(async move {
                let node_addr = ep_once.node_addr().initialized().await;
                println!("[IROH] node_addr initialized: {:?}", node_addr);
                let home_relay = ep_once.home_relay().initialized().await;
                println!("[IROH] home_relay: {:?}", home_relay);
            });
        }

        Ok((Self { endpoint, debug }, rx))
    }


    /// Return the full NodeTicket string (ticket) for the endpoint, waiting for initialization if necessary.
    /// Always returns a String (may be empty only in pathological cases where initialization failed).
    pub async fn node_addr_string(&self) -> String {
        let na = self.endpoint.node_addr().initialized().await;
        let ticket = iroh_base::ticket::NodeTicket::from(na);
        ticket.to_string()
    }
}

#[async_trait::async_trait]
impl Transport for IrohTransport {
    async fn start(&mut self) -> Result<()> {
        // Endpoint is initialized during IrohTransport::new(), so start() is a no-op here.
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.endpoint.close().await;
        Ok(())
    }

    fn node_id(&self) -> String {
        self.endpoint.node_id().to_string()
    }

    async fn send_message(&self, peer: Option<String>, msg: &ServerMsg) -> Result<()> {
        // If peer is None, per current design we do nothing (no broadcast).
        if peer.is_none() {
            return Ok(());
        }

        let pid = peer.unwrap();
        let endpoint = &self.endpoint;
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
        let (mut send, _recv) = conn.open_bi().await.map_err(|e| anyhow::anyhow!(e))?;
        // Serialize and frame
        let bytes = serde_json::to_vec(msg)?;
        let framed = crate::transport::framing::encode_frame(&bytes);

        // Write all and finish
        send.write_all(&framed)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        // finish() is synchronous in this iroh version
        send.finish().map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
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
