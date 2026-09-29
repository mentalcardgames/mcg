use clap::Parser;
use std::path::PathBuf;

/// Server CLI for mcg-server
#[derive(Parser, Debug, Clone)]
#[command(name = "mcg-server", version, about = "MCG poker server")]
pub struct ServerCli {
    /// Path to config file
    #[arg(long, default_value = "mcg-server.toml")]
    pub config: PathBuf,

    /// Port to listen on (overrides config.port)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Require exact port without searching for next available port
    #[arg(long, default_value_t = false)]
    pub strict_port: bool,

    /// Iroh key as hex (overrides config.iroh_key)
    #[arg(long)]
    pub iroh_key: Option<String>,

    /// Run with an ephemeral, non-persisted Iroh identity key (useful for local multi-instance testing)
    #[arg(long, default_value_t = false)]
    pub ephemeral: bool,

    /// Persist CLI overrides back to the config file
    #[arg(long, default_value_t = false)]
    pub persist: bool,

    /// Enable verbose debug logging
    #[arg(long, short, default_value_t = false)]
    pub debug: bool,
}
