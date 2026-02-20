use std::io::IsTerminal;

use mcg_shared::{GameStatePublic, PlayerConfig, ServerMsg};

use native_mcg::pretty::{format_event_human, format_state_human, format_table_header};

#[derive(Clone, Copy)]
pub enum DisplayMode {
    FullState,
    Incremental,
}

pub struct MessagePrinter {
    json: bool,
    mode: DisplayMode,
    last_printed: usize,
    latest_state: Option<GameStatePublic>,
}

impl MessagePrinter {
    pub fn new(json: bool, mode: DisplayMode) -> Self {
        Self {
            json,
            mode,
            last_printed: 0,
            latest_state: None,
        }
    }

    pub fn handle(&mut self, msg: &ServerMsg) {
        match msg {
            ServerMsg::State(gs) => {
                self.latest_state = Some(gs.clone());
                match self.mode {
                    DisplayMode::FullState => self.print_full_state(gs),
                    DisplayMode::Incremental => self.print_incremental(gs),
                }
            }
            ServerMsg::Error(e) => eprintln!("Server error: {}", e),
            ServerMsg::Pong => println!("Received pong"),
            ServerMsg::QrRes(inner) => {
                println!("Qr Response: {:?}", inner);
            }
        }
    }

    fn print_full_state(&self, gs: &GameStatePublic) {
        if self.json {
            match serde_json::to_string_pretty(gs) {
                Ok(json_str) => println!("{}", json_str),
                Err(e) => eprintln!("Failed to serialize state to JSON: {}", e),
            }
        } else {
            let use_color = std::io::stdout().is_terminal();
            println!("{}", format_state_human(gs, use_color));
        }
    }

    fn print_incremental(&mut self, gs: &GameStatePublic) {
        if self.json {
            match serde_json::to_string_pretty(gs) {
                Ok(json_str) => println!("{}", json_str),
                Err(e) => eprintln!("Failed to serialize state to JSON: {}", e),
            }
            return;
        }

        let already = self.last_printed;
        let total = gs.action_log.len();
        if total < already {
            let use_color = std::io::stdout().is_terminal();
            let header = format_table_header(gs, gs.sb, gs.bb, use_color);
            println!("{}", header);
            self.last_printed = total;
        } else if total > already {
            for e in gs.action_log.iter().skip(already) {
                println!(
                    "{}",
                    format_event_human(e, &gs.players, std::io::stdout().is_terminal())
                );
            }
            self.last_printed = total;
        }
    }
}

pub fn generate_demo_players(num_players: usize) -> Vec<PlayerConfig> {
    let mut players = Vec::with_capacity(num_players);
    players.push(PlayerConfig {
        id: mcg_shared::PlayerId(0),
        name: format!("Huuman player {}", 1),
        is_bot: false,
    });
    for i in 1..num_players {
        players.push(PlayerConfig {
            id: mcg_shared::PlayerId(i),
            name: format!("Player {}", i + 1),
            is_bot: true,
        });
    }
    players
}
