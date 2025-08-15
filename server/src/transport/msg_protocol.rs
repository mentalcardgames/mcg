use iroh::protocol::{ProtocolHandler, AcceptError};
use iroh::endpoint::Connection;
use bytes::BytesMut;
use crate::transport::framing::try_parse;
use std::sync::Arc;
use tokio::sync::Mutex;
use mcg_shared::ClientMsg;
use std::future::Future;

// Protocol handler that reads length-prefixed frames, parses JSON ClientMsg and invokes
// a user-provided callback with the peer id and ClientMsg.
// Reduce type complexity by introducing a type alias for the callback storage.
type OnClientCallback = Arc<Mutex<Option<Arc<dyn Fn(String, ClientMsg) + Send + Sync>>>>;

pub struct MsgProtocol {
    // Shared callback storage. If None, incoming messages are ignored.
    pub on_client: OnClientCallback,
}

impl std::fmt::Debug for MsgProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MsgProtocol{on_client: <callback>}")
    }
}

impl Clone for MsgProtocol {
    fn clone(&self) -> Self {
        MsgProtocol { on_client: self.on_client.clone() }
    }
}

impl MsgProtocol {
    pub fn new(on_client: OnClientCallback) -> Self {
        MsgProtocol { on_client }
    }
}

// Provide a convenient alias to satisfy complexity lints for the protocol handler signature
#[allow(clippy::type_complexity)]
type DynOnClient = Arc<dyn Fn(String, ClientMsg) + Send + Sync>;

impl ProtocolHandler for MsgProtocol {
    fn accept(&self, connection: Connection) -> impl Future<Output = Result<(), AcceptError>> + Send {
        let on_client = self.on_client.clone();
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
                                let guard = on_client.lock().await;
                                if let Some(cb) = guard.as_ref() {
                                    cb(peer.clone(), cm);
                                }
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
