//! Shared types and data structures for the Mental Card Game (MCG).
//!
//! This crate contains all the common types used between the frontend and backend
//! components of the MCG application, including cards, game state, player information,
//! hand evaluation, and the client-server messaging protocol.

// Module declarations
pub mod cards;
pub mod communication;
pub mod game;
pub mod hand;
pub mod messages;
pub mod player;

// Re-export all public types for easy access
pub use cards::*;
pub use game::*;
pub use hand::*;
pub use messages::*;
pub use player::*;
