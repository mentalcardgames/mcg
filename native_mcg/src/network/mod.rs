//! Transport-neutral networking primitives and connection actors.
//!
//! This module forms the boundary between asynchronous transports and the
//! backend's application logic. Transport implementations emit typed
//! [`NetworkEvent`] values instead of accessing application state directly.

mod iroh;
mod supervisor;
mod types;
mod websocket;

pub use iroh::run_iroh_peer_actor;
pub use supervisor::{NetworkError, NetworkHandle, NetworkSupervisor};
pub use types::{
    ConnectionCloseReason, ConnectionId, ConnectionInfo, ConnectionRole, FrontendConnectionCommand,
    NetworkCommand, NetworkEvent, PeerConnectionCommand, PeerId, TransportKind,
};
pub use websocket::run_websocket_actor;
