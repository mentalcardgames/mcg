use std::{io::IsTerminal, num};

use mcg_shared::{GameStatePublic, PlayerConfig, ServerMsg};

use native_mcg::pretty::{format_event_human, format_state_human, format_table_header};

/// Print a state either as JSON or human-friendly text.
pub fn output_state(state: &GameStatePublic, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(state).unwrap());
    } else {
        let use_color = std::io::stdout().is_terminal();
        println!("{}", format_state_human(state, use_color));
    }
}

pub fn generate_demo_players(num_players: usize) -> Vec<PlayerConfig> {
    let mut players = Vec::with_capacity(num_players);
    players.push(PlayerConfig {
        id: mcg_shared::PlayerId(0),
        name: format!("Huuman player {}", 0 + 1),
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

/// Shared handler for server messages so the CLI doesn't duplicate logic.
///
/// Supports incremental printing by accepting a mutable `last_printed` index
/// which tracks how many log entries have already been displayed. For JSON
/// mode this still prints the full message.
pub fn handle_server_msg(sm: &ServerMsg, json: bool, last_printed: &mut usize) {
    match sm {
        ServerMsg::State(gs) => {
            if json {
                // In JSON mode, print the full state as before.
                output_state(gs, true);
            } else {
                // Print only newly appended log entries (to avoid repeating full state).
                // Handle the case where the server resets/truncates the action_log (e.g. on new hand).
                let already = *last_printed;
                let total = gs.action_log.len();
                if total < already {
                    // action_log was reset on the server (new hand); print table header like the server
                    let use_color = std::io::stdout().is_terminal();
                    let header = format_table_header(gs, gs.sb, gs.bb, use_color);
                    println!("{}", header);
                    *last_printed = total;
                } else if total > already {
                    for e in gs.action_log.iter().skip(already) {
                        println!(
                            "{}",
                            format_event_human(e, &gs.players, std::io::stdout().is_terminal())
                        );
                    }
                    *last_printed = total;
                }
            }
        }
        ServerMsg::Error(e) => eprintln!("Server error: {}", e),
        ServerMsg::Welcome { .. } => {
            if json {
                // If user wants JSON, print the welcome message as JSON.
                if let Ok(txt) = serde_json::to_string_pretty(sm) {
                    println!("{}", txt);
                }
            } else {
                println!("Backend says Welcome!")
            }
        }
    }
}
