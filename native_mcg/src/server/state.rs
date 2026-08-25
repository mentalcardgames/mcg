//! Server configuration and runtime state.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

/// Shared application state exposed to handlers and background tasks.
#[derive(Clone)]
pub struct AppState {
    /// In-memory shared Config instance. Holds the authoritative configuration
    /// for the running server.
    pub config: Arc<RwLock<Config>>,
    /// Optional path to the TOML config file used by the running server.
    /// If present, transports (e.g. iroh) may persist changes to this path.
    pub config_path: Option<PathBuf>,
    /// Persisted or generated Iroh ticket for the local endpoint.
    pub ticket: Arc<RwLock<Option<String>>>,
}

impl AppState {
    /// Create a new AppState with the given config and optional config path.
    pub fn new(config: Config, config_path: Option<PathBuf>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            ticket: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
            config_path: None,
            ticket: Arc::new(RwLock::new(None)),
        }
    }
}
