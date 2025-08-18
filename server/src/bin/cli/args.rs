use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(name = "mcg-cli", version, about = "Headless CLI for MCG poker demo", long_about = None)]
pub struct Cli {
    /// Base server URL (http(s)://host:port or ws(s)://host:port/ws)
    #[arg(short, long, default_value = "http://localhost:3000")]
    pub server: String,

    /// Join name to use for the single player
    #[arg(short, long, default_value = "CLI")]
    pub name: String,

    /// Connect to an iroh peer by public key (z-base-32). When set the CLI
    /// attempts to use iroh transport instead of WebSocket. (Explicit target.)
    #[arg(long)]
    pub iroh_peer: Option<String>,

    /// How long to wait for server state updates after sending a command (ms)
    #[arg(long, default_value_t = 1200)]
    pub wait_ms: u64,

    /// Output JSON instead of human-readable text
    #[arg(long, default_value_t = false)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Connect and send Join, then print the first State and exit
    Join,
    /// Request the latest State and print it
    State,
    /// Send an action (Fold, Check/Call, Bet amount)
    Action {
        #[arg(value_enum)]
        kind: ActionKind,
        /// Amount for bet action (ignored for fold/checkcall)
        #[arg(long, default_value_t = 0)]
        amount: u32,
    },
    /// Advance to the next hand
    NextHand,
    /// Reset the game with N bots
    Reset {
        #[arg(long, short = 'b', default_value_t = 1)]
        bots: usize,
    },
    /// Watch game events continuously and print them as they happen
    Watch,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ActionKind {
    Fold,
    CheckCall,
    Bet,
}
