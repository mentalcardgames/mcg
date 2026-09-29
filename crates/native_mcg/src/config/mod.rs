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

    if let Some(port) = cli.port {
        cfg.port = port;
    }

    if cli.strict_port {
        cfg.strict_port = true;
    }

    if let Some(ref k) = cli.iroh_key {
        cfg.iroh_key = Some(k.clone());
    }

    if cli.ephemeral {
        cfg.iroh_key = None;
    }

    if cli.persist && !cli.ephemeral {
        cfg.save(&config_path)
            .with_context(|| format!("saving updated config '{}'", config_path.display()))?;
    }

    Ok((cfg, config_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_clears_iroh_key_and_does_not_persist() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let test_id = rand::random::<u64>();
        let path = temp_dir.join(format!("mcg_test_config_{}.toml", test_id));

        // Write an initial config with an iroh_key
        let initial_cfg = Config {
            iroh_key: Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
            ),
            ..Default::default()
        };
        initial_cfg.save(&path)?;

        let cli = ServerCli {
            config: path.clone(),
            port: Some(4000),
            strict_port: true,
            iroh_key: None,
            ephemeral: true,
            persist: true,
            debug: false,
        };

        let (cfg, returned_path) = init_server_config(&cli)?;
        assert_eq!(returned_path, path);
        assert_eq!(cfg.port, 4000);
        assert!(cfg.strict_port);
        // Ephemeral must clear iroh_key
        assert!(cfg.iroh_key.is_none());

        // File on disk must retain original content because persist was suppressed
        let loaded = Config::load_or_create(&path)?;
        assert_eq!(
            loaded.iroh_key,
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into())
        );

        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
