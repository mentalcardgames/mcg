pub mod framing;
pub mod iroh_transport;
pub mod websocket_transport;
pub mod msg_protocol;

pub use iroh_transport::IrohTransport;
pub use websocket_transport::WebSocketTransport;

use anyhow::Result;
use async_trait::async_trait;
use mcg_shared::{ClientMsg, ServerMsg};
use std::path::PathBuf;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn node_id(&self) -> Option<String>;
    async fn send_message(&self, peer: Option<String>, msg: &ServerMsg) -> Result<()>;
    fn set_on_client_message(&mut self, cb: Box<dyn Fn(String, ClientMsg) + Send + Sync>);
    async fn advertise_blob(&self, path: PathBuf) -> Result<String>;
    async fn fetch_blob(&self, hash: &str, node_id: Option<&str>, out_path: PathBuf) -> Result<()>;
}
