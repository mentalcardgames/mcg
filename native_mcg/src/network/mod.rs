//! Transport-neutral networking primitives and connection actors.
//!
//! This module forms the boundary between asynchronous transports and the
//! backend's application logic. Transport implementations emit typed
//! internal typed events instead of accessing application state directly. The
//! supervisor enriches them into [`NetworkEvent`] values for application code.

pub mod axum;
pub mod iroh;
pub mod peer_service;
pub mod supervisor;
pub mod types;
pub mod websocket;

pub use axum::{
    build_router, health_handler, http_handler, serve_index, spa_handler, ws_handler, RouterState,
};
pub use iroh::{spawn_iroh_listener, IrohListenerTask, IROH_FRONTEND_ALPN, IROH_PEER_ALPN};
pub use peer_service::{EstablishedPeer, PeerConnectionService};
pub use supervisor::{NetworkHandle, NetworkSupervisor};
pub use types::{
    ConnectionCloseReason, ConnectionId, FrontendConnectionCommand, NetworkCommand, NetworkError,
    NetworkEvent, PeerConnectionCommand, PeerConnectionDirection, PeerConnectionError, PeerId,
    ProtocolRole, TransportKind,
};
pub use websocket::{WEBSOCKET_FRONTEND_PROTOCOL, WEBSOCKET_PEER_PROTOCOL};
