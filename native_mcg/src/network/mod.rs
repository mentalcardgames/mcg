//! Transport-neutral networking primitives and connection actors.
//!
//! This module forms the boundary between asynchronous transports and the
//! backend's application logic. Transport implementations emit typed
//! [`NetworkEvent`] values instead of accessing application state directly.

mod types;

pub use types::{ConnectionId, NetworkEvent};
