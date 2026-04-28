//! Shared transport helpers for sending Backend2FrontendMsg over different transports.
//!
//! Provides small, focused helpers so websocket and iroh handlers can reuse
//! the same serialization logic and error handling.

use anyhow::Result;
use mcg_shared::Backend2FrontendMsg;
use mcg_shared::Peer2PeerMsg;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

/// Send a Backend2FrontendMsg to an AsyncWrite sink as a newline-delimited JSON line.
///
/// Used by the iroh transport which exposes an AsyncWrite-like send handle.
pub async fn send_server_msg_to_writer<W>(writer: &mut W, msg: &Backend2FrontendMsg) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    let txt = serde_json::to_string(msg)?;
    writer.write_all(txt.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}
/// Send a peer message back to a peer that contacted us, in the same way the above function does.
pub async fn send_peer_msg_to_writer<W>(writer: &mut W, msg: &Peer2PeerMsg) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    let txt = serde_json::to_string(msg)?;
    writer.write_all(txt.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}