use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const PUBLIC_FILE_NAME: &str = "mcg-server-public.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublicInfo {
    #[serde(default)]
    pub iroh_node_id: Option<String>,
}

impl PublicInfo {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let text = fs::read_to_string(path)
                .with_context(|| format!("reading public info '{}':", path.display()))?;
            let info = toml::from_str(&text)
                .with_context(|| format!("parsing public info '{}':", path.display()))?;
            Ok(info)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("creating public info directory '{}':", parent.display())
                })?;
            }
        }
        let text = toml::to_string_pretty(self).with_context(|| "serializing public info")?;
        fs::write(path, text)
            .with_context(|| format!("writing public info '{}':", path.display()))?;
        Ok(())
    }

    pub fn write_iroh_node_id(path: &Path, node_id: impl Into<String>) -> Result<Self> {
        let mut info = Self::load(path)?;
        info.iroh_node_id = Some(node_id.into());
        info.save(path)?;
        Ok(info)
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from(PUBLIC_FILE_NAME)
    }
}

pub fn path_for_config(config_path: Option<&Path>) -> PathBuf {
    match config_path {
        Some(path) => {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                path.with_file_name(format!("{stem}-public.toml"))
            } else {
                path.with_file_name(PUBLIC_FILE_NAME)
            }
        }
        None => PublicInfo::default_path(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_file_name_uses_hyphens() {
        assert_eq!(PUBLIC_FILE_NAME, "mcg-server-public.toml");
        assert_eq!(
            PublicInfo::default_path(),
            PathBuf::from("mcg-server-public.toml")
        );
    }

    #[test]
    fn test_path_for_config_derivation() {
        assert_eq!(
            path_for_config(None),
            PathBuf::from("mcg-server-public.toml")
        );
        assert_eq!(
            path_for_config(Some(Path::new("mcg-server.toml"))),
            PathBuf::from("mcg-server-public.toml")
        );
        assert_eq!(
            path_for_config(Some(Path::new("custom/mcg-server-2.toml"))),
            PathBuf::from("custom/mcg-server-2-public.toml")
        );
    }
}
