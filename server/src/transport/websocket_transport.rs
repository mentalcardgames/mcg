use crate::transport::Transport;
use anyhow::Result;
use futures_util::SinkExt;
use mcg_shared::{ClientMsg, ServerMsg};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

/// A thin wrapper that reuses the existing websocket handler logic but exposes the Transport trait.
// Reduce type complexity warning by providing a type alias for the peers map.
type PeersMap = std::collections::HashMap<
    String,
    futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>,
>;

pub struct WebSocketTransport {
    // map of peer id -> sender
    peers: Arc<Mutex<PeersMap>>,
    on_client: Option<Arc<dyn Fn(String, ClientMsg) + Send + Sync>>,
}

impl WebSocketTransport {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
            on_client: None,
        }
    }
}

#[async_trait::async_trait]
impl Transport for WebSocketTransport {
    async fn start(&mut self) -> Result<()> {
        // WebSocket transport uses existing axum route; nothing to start here.
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn node_id(&self) -> String {
        String::new()
    }

    async fn send_message(&self, peer: Option<String>, msg: &ServerMsg) -> Result<()> {
        let txt = serde_json::to_string(msg)?;
        let send = Message::Text(txt);
        let mut peers = self.peers.lock().await;
        if let Some(pid) = peer {
            if let Some(sink) = peers.get_mut(&pid) {
                let _ = sink.send(send).await;
            }
        } else {
            for (_k, sink) in peers.iter_mut() {
                let _ = sink.send(send.clone()).await;
            }
        }
        Ok(())
    }


    async fn advertise_blob(&self, path: PathBuf) -> Result<String> {
        // For parity, implement a filesystem-backed blob hashing using blake3 and return hex
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
        // No distributed fetch for websocket mode — in parity mode, clients should upload blobs via HTTP.
        // Here we return an error to indicate not available.
        Err(anyhow::anyhow!(
            "fetch_blob not supported for websocket transport"
        ))
    }
}
