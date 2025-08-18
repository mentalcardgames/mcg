use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(name = "mcg-cli", version, about = "Headless CLI for MCG poker demo", long_about = None)]
pub struct Cli {
    /// Join name to use for the single player
    #[arg(short, long, default_value = "CLI")]
    pub name: String,

    /// Transport to use and its address. Format:
    /// - http:<ADDRESS>         (e.g. --transport 'http:http://localhost:3000')
    /// - websocket:<ADDRESS>    (e.g. --transport 'websocket:ws://localhost:3000/ws')
    /// - iroh:<PEER>            (e.g. --transport 'iroh:zb2...peerid...')
    ///
    /// Default: http:http://localhost:3000
    #[arg(long, default_value = "http:http://localhost:3000")]
    pub transport: TransportKind,

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

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ActionKind {
    Fold,
    CheckCall,
    Bet,
}

/// Transport kind for the CLI. Each variant carries an address string:
/// - Http(address)      : HTTP server base URL (e.g. http://host:port)
/// - WebSocket(address) : WebSocket URL or HTTP base that will be converted (e.g. ws://host:port/ws or http://host:port)
/// - Iroh(peer)         : Iroh peer id (z-base-32)
#[derive(Debug, Clone)]
pub enum TransportKind {
    WebSocket(String),
    Http(String),
    Iroh(String),
}

impl std::str::FromStr for TransportKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err("transport cannot be empty".into());
        }
        let lower = s.to_ascii_lowercase();
        if lower.starts_with("http:") {
            let addr = s["http:".len()..].to_string();
            if addr.is_empty() {
                Err("http transport requires an address: --transport 'http:ADDRESS'".into())
            } else {
                Ok(TransportKind::Http(addr))
            }
        } else if lower.starts_with("websocket:") || lower.starts_with("ws:") || lower.starts_with("web-socket:") {
            // preserve original case for address portion
            let prefix_len = if lower.starts_with("websocket:") {
                "websocket:".len()
            } else if lower.starts_with("web-socket:") {
                "web-socket:".len()
            } else {
                "ws:".len()
            };
            let addr = s[prefix_len..].to_string();
            if addr.is_empty() {
                Err("websocket transport requires an address: --transport 'websocket:ADDRESS'".into())
            } else {
                Ok(TransportKind::WebSocket(addr))
            }
        } else if lower.starts_with("iroh:") {
            let peer = s["iroh:".len()..].to_string();
            if peer.is_empty() {
                Err("iroh transport requires a peer id: --transport 'iroh:PEER'".into())
            } else {
                Ok(TransportKind::Iroh(peer))
            }
        } else {
            Err(format!("unknown transport '{}', expected forms: http:ADDR, websocket:ADDR, iroh:PEER", s))
        }
    }
}

impl std::fmt::Display for TransportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportKind::WebSocket(addr) => write!(f, "websocket:{}", addr),
            TransportKind::Http(addr) => write!(f, "http:{}", addr),
            TransportKind::Iroh(peer) => write!(f, "iroh:{}", peer),
        }
    }
}
