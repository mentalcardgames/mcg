use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;

use crate::backend::BackendBuilder;
use crate::config::Config;

/// Convenience function to run the full server for the given address and configuration.
pub async fn run_server(
    addr: SocketAddr,
    config: Config,
    config_path: Option<PathBuf>,
) -> Result<()> {
    BackendBuilder::new(config)
        .with_config_path_opt(config_path)
        .build()?
        .run(addr)
        .await
}
