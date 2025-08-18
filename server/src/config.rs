use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, io};
use anyhow::{Context, Result};

/// Server configuration persisted as TOML.
///
/// Fields:
/// - bots: number of bot players to start with
/// - iroh_key: optional iroh key (peer id or private key string depending on usage)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub bots: usize,
    pub iroh_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bots: 1,
            iroh_key: None,
        }
    }
}

impl Config {
    /// Load configuration from `path`. If the file does not exist, create it
    /// with reasonable defaults and return the default config.
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let s = fs::read_to_string(path)
                .with_context(|| format!("reading config file '{}'", path.display()))?;
            let cfg: Config = toml::from_str(&s)
                .with_context(|| format!("parsing TOML config '{}'", path.display()))?;
            Ok(cfg)
        } else {
            // Create directories if needed
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("creating config directory '{}'", parent.display()))?;
                }
            }

            let cfg = Config::default();
            let toml_text = toml::to_string_pretty(&cfg)
                .with_context(|| "serializing default config to TOML")?;
            fs::write(path, toml_text)
                .with_context(|| format!("writing default config to '{}'", path.display()))?;
            Ok(cfg)
        }
    }

    /// Load (or create) config and optionally override with a CLI-provided `bots` value.
    /// If an override is applied, the config file will be updated on disk to reflect it.
    pub fn load_or_create_with_override(path: &Path, cli_bots: Option<usize>) -> Result<Self> {
        let mut cfg = Self::load_or_create(path)?;
        if let Some(b) = cli_bots {
            cfg.bots = b;
            // persist change back to file
            let toml_text = toml::to_string_pretty(&cfg)
                .with_context(|| "serializing config to TOML")?;
            fs::write(path, toml_text)
                .with_context(|| format!("writing updated config to '{}'", path.display()))?;
        }
        Ok(cfg)
    }
}
