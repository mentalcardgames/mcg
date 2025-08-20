use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(name = "mcg-cli", version, about = "Headless CLI for MCG poker demo", long_about = None)]
pub struct Cli {
    /// Join name to use for the single player
    #[arg(short, long, default_value = "CLI")]
    pub name: String,

    /// Transport to use and its address. Accepted forms:
    /// - Full URL starting with http:// or https:// (treated as HTTP)
    ///   (e.g. --transport 'http://localhost:3000' or '--transport https://example.com')
    /// - Full URL starting with ws:// or wss:// (treated as WebSocket)
    ///   (e.g. --transport 'ws://localhost:3000/ws' or '--transport wss://example.com/ws')
    /// - iroh prefix for Iroh peer ids:
    ///   - iroh:<PEER>            (e.g. --transport 'iroh:zb2...peerid...')
    ///
    /// Default: http://localhost:3000
    #[arg(long, default_value = "http://localhost:3000")]
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
    /// Start a new game with default players
    NewGame,
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

        // Explicit iroh: prefix
        if lower.starts_with("iroh:") {
            let peer = s["iroh:".len()..].to_string();
            if peer.is_empty() {
                return Err("iroh transport requires a peer id: --transport 'iroh:PEER'".into());
            } else {
                return Ok(TransportKind::Iroh(peer));
            }
        }

        // Full URL forms: http(s) -> Http, ws(s) -> WebSocket
        if lower.starts_with("http://") || lower.starts_with("https://") {
            return Ok(TransportKind::Http(s.to_string()));
        }
        if lower.starts_with("ws://") || lower.starts_with("wss://") {
            return Ok(TransportKind::WebSocket(s.to_string()));
        }

        // No legacy prefixed forms supported anymore
        Err(format!(
            "unknown transport '{}', expected forms: http(s)://URL, ws(s)://URL, or iroh:PEER",
            s
        ))
    }
}

impl std::fmt::Display for TransportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportKind::WebSocket(addr) => write!(f, "{}", addr),
            TransportKind::Http(addr) => write!(f, "{}", addr),
            TransportKind::Iroh(peer) => write!(f, "iroh:{}", peer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_http_and_https() {
        let h1 = TransportKind::from_str("http://localhost:3000").expect("should parse http");
        assert!(matches!(h1, TransportKind::Http(ref a) if a == "http://localhost:3000"));
        let h2 = TransportKind::from_str("https://example.com").expect("should parse https");
        assert!(matches!(h2, TransportKind::Http(ref a) if a == "https://example.com"));
    }

    #[test]
    fn parse_ws_and_wss() {
        let w1 = TransportKind::from_str("ws://localhost:3000/ws").expect("should parse ws");
        assert!(matches!(w1, TransportKind::WebSocket(ref a) if a == "ws://localhost:3000/ws"));
        let w2 = TransportKind::from_str("wss://example.com/ws").expect("should parse wss");
        assert!(matches!(w2, TransportKind::WebSocket(ref a) if a == "wss://example.com/ws"));
    }

    #[test]
    fn parse_iroh() {
        let i = TransportKind::from_str("iroh:zb2examplepeer").expect("should parse iroh");
        assert!(matches!(i, TransportKind::Iroh(ref p) if p == "zb2examplepeer"));
    }

    #[test]
    fn reject_legacy_prefixes() {
        assert!(TransportKind::from_str("http:localhost:3000").is_err());
        assert!(TransportKind::from_str("websocket:ws://localhost:3000/ws").is_err());
        assert!(TransportKind::from_str("ws:localhost:3000").is_err());
        assert!(TransportKind::from_str("web-socket:ws://localhost:3000/ws").is_err());
    }

    #[test]
    fn display_formats() {
        let h = TransportKind::from_str("http://localhost:3000").unwrap();
        assert_eq!(h.to_string(), "http://localhost:3000");
        let w = TransportKind::from_str("ws://localhost:3000/ws").unwrap();
        assert_eq!(w.to_string(), "ws://localhost:3000/ws");
        let i = TransportKind::from_str("iroh:zb2peer").unwrap();
        assert_eq!(i.to_string(), "iroh:zb2peer");
    }
}
