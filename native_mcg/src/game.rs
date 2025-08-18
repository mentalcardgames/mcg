#![allow(clippy::module_inception)]
//! Refactored game module. Implementation split across multiple files for clarity.
//
//! Module ordering matters: declare utility/dealing/showdown before engine so
//! engine can reference sibling modules via `super::...`.

mod utils;
mod dealing;
mod showdown;
mod betting;
mod engine;

pub use engine::Game;
