//! Transport-neutral networking primitives and connection actors.
//!
//! This module forms the boundary between asynchronous transports and the
//! backend's application logic. Transport implementations emit typed
//! [`NetworkEvent`] values instead of accessing application state directly.

mod supervisor;
mod types;
mod websocket;

pub use supervisor::{NetworkError, NetworkHandle, NetworkSupervisor};
pub use types::{
    ConnectionCloseReason, ConnectionId, ConnectionInfo, ConnectionRole, FrontendConnectionCommand,
    NetworkCommand, NetworkEvent, TransportKind,
};
pub use websocket::run_websocket_actor;
