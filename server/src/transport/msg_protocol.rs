use iroh::protocol::{ProtocolHandler, AcceptError};
use iroh::endpoint::Connection;
use bytes::BytesMut;
use crate::transport::framing::try_parse;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use mcg_shared::ClientMsg;
use std::future::Future;

// Protocol handler that reads length-prefixed frames, parses JSON ClientMsg and forwards
// parsed messages to a tokio mpsc::UnboundedSender<(peer, ClientMsg)> for processing by the server.

type MsgSender = UnboundedSender<(String, ClientMsg)>;

pub struct MsgProtocol {
    // Sender to forward parsed messages to the server task.
    pub sender: MsgSender,
}

impl std::fmt::Debug for MsgProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MsgProtocol{sender: <mpsc::UnboundedSender>}")
    }
}

impl Clone for MsgProtocol {
    fn clone(&self) -> Self {
        MsgProtocol { sender: self.sender.clone() }
    }
}

impl MsgProtocol {
    pub fn new(sender: MsgSender) -> Self {
        MsgProtocol { sender }
    }
}

impl ProtocolHandler for MsgProtocol {
    fn accept(&self, connection: Connection) -> impl Future<Output = Result<(), AcceptError>> + Send {
        let sender = self.sender.clone();
        async move {
            // Each connection: accept a bidi stream and read until the peer finishes.
            let peer = match connection.remote_node_id() {
                Ok(id) => id.to_string(),
                Err(_) => "unknown".to_string(),
            };

            let (mut send, mut recv) = connection.accept_bi().await.map_err(AcceptError::from_err)?;
            let data = recv.read_to_end(10_000_000).await.map_err(AcceptError::from_err)?;
            let mut buf = BytesMut::from(&data[..]);

            // Parse zero or more framed messages from the buffer.
            loop {
                match try_parse(&mut buf) {
                    Ok(Some(frame)) => {
                        if let Ok(txt) = String::from_utf8(frame) {
                            if let Ok(cm) = serde_json::from_str::<ClientMsg>(&txt) {
                                // forward to server task; ignore send error if receiver closed
                                let _ = sender.send((peer.clone(), cm));
                            }
                        }
                        continue;
                    }
                    Ok(None) => break,
                    Err(e) => {
                        eprintln!("[IROH] frame parse error: {:?}", e);
                        break;
                    }
                }
            }

            // Finish the send stream (no reply payload for now) and wait for connection close.
            send.finish().map_err(AcceptError::from_err)?;
            connection.closed().await;
            Ok(())
        }
    }
}
