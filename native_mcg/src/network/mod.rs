//! Transport-neutral networking primitives and connection actors.
//!
//! This module forms the boundary between asynchronous transports and the
//! backend's application logic. Transport implementations emit typed
//! internal typed events instead of accessing application state directly. The
//! supervisor enriches them into [`NetworkEvent`] values for application code.

mod iroh;
mod supervisor;
mod types;
mod websocket;

pub use iroh::{IROH_FRONTEND_ALPN, IROH_PEER_ALPN};
pub use supervisor::{NetworkError, NetworkHandle, NetworkSupervisor};
pub use types::{
    ConnectionCloseReason, ConnectionId, FrontendConnectionCommand, NetworkCommand, NetworkEvent,
    PeerConnectionCommand, PeerConnectionDirection, PeerId, ProtocolRole, TransportKind,
};
