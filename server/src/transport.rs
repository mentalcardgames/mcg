//! Shared transport helpers for sending ServerMsg over different transports.
//!
//! Provides small, focused helpers so websocket and iroh handlers can reuse
//! the same serialization logic and error handling.

use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use mcg_shared::ServerMsg;

/// Send a ServerMsg over an Axum WebSocket connection.
///
/// This mirrors the previous inline implementation used in server.rs.
pub async fn send_server_msg_ws(socket: &mut WebSocket, msg: &ServerMsg) {
    match serde_json::to_string(msg) {
        Ok(txt) => {
            let _ = socket.send(Message::Text(txt)).await;
        }
        Err(e) => {
            eprintln!("Failed to serialize ServerMsg for websocket send: {}", e);
        }
    }
}

/// Send a ServerMsg to an AsyncWrite sink as a newline-delimited JSON line.
///
/// Used by the iroh transport which exposes an AsyncWrite-like send handle.
pub async fn send_server_msg_to_writer<W>(writer: &mut W, msg: &ServerMsg) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    let txt = serde_json::to_string(msg)?;
    writer.write_all(txt.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}