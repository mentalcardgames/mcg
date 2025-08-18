use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, io};
use anyhow::{Context, Result};

/// Server configuration persisted as TOML.
///
/// Fields:
/// - bots: number of bot players to start with
/// - iroh_key: optional iroh key stored as hex string of 32 bytes
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

    /// Save the current config state back to the provided path (overwrites).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating config directory '{}'", parent.display()))?;
            }
        }
        let toml_text = toml::to_string_pretty(&self)
            .with_context(|| "serializing config to TOML")?;
        fs::write(path, toml_text)
            .with_context(|| format!("writing config to '{}'", path.display()))?;
        Ok(())
    }


    /// Return iroh key bytes if present in config (hex-decoded).
    pub fn iroh_key_bytes(&self) -> Option<Vec<u8>> {
        if let Some(ref s) = self.iroh_key {
            if let Ok(b) = hex::decode(s) {
                return Some(b);
            }
        }
        None
    }

    /// Set iroh key from raw bytes and persist to disk (via save).
    pub fn set_iroh_key_bytes_and_save(&mut self, path: &Path, bytes: &[u8]) -> Result<()> {
        self.iroh_key = Some(hex::encode(bytes));
        self.save(path)?;
        Ok(())
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
