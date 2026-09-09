pub mod cli;
pub mod public;
pub mod server;

pub use cli::ServerCli;
pub use public::{path_for_config, PublicInfo, PUBLIC_FILE_NAME};
pub use server::Config;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Initializes configuration from CLI arguments:
/// loads or creates the config file, applies CLI overrides, and optionally persists them.
pub fn init_server_config(cli: &ServerCli) -> Result<(Config, PathBuf)> {
    let config_path = cli.config.clone();
    let mut cfg = Config::load_or_create(&config_path)
        .with_context(|| format!("loading or creating config '{}'", config_path.display()))?;

    if let Some(ref k) = cli.iroh_key {
        cfg.iroh_key = Some(k.clone());
    }

    if cli.persist {
        cfg.save(&config_path)
            .with_context(|| format!("saving updated config '{}'", config_path.display()))?;
    }

    Ok((cfg, config_path))
}
