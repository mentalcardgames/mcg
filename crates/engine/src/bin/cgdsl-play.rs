use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use cgdsl_engine::{run_game, GameData, Input, InputSource, InputType};
use front_end::validation::parse_document;

fn main() {
    let args: Vec<String> = env::args().collect();

    let prog = &args[0];
    let game_file = args.get(1).unwrap_or_else(|| {
        eprintln!("Usage: {prog} <game.cgdsl> [input.txt]", prog = prog);
        std::process::exit(1);
    });
    let input_file = args.get(2);

    let source = match std::fs::read_to_string(game_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading `{game_file}`: {e}");
            std::process::exit(1);
        }
    };

    let game = match parse_document(&source) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Parse error in `{game_file}`:\n{e}");
            std::process::exit(1);
        }
    };
    let ir = game.to_lowered_graph();

    let input_source = match input_file {
        Some(path) => InputSource::TestFile(PathBuf::from(path)),
        None => InputSource::Player(Box::new(interactive_input)),
    };

    let game_data = GameData::new();
    match run_game(ir, game_data, input_source, None) {
        Ok(state) => print_summary(&state),
        Err(e) => {
            eprintln!("Game error: {e}");
            std::process::exit(1);
        }
    }
}

fn interactive_input(input_type: InputType) -> Input {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        match &input_type {
            InputType::Choice { options, max_index } => {
                println!("\n--- Choice ---");
                for (i, opt) in options.iter().enumerate() {
                    println!("  {}. {opt}", i + 1);
                }
                print!("Enter 1-{}: ", max_index + 1);
                io::stdout().flush().ok();
                let mut line = String::new();
                match handle.read_line(&mut line) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Input error: {e}");
                        continue;
                    }
                }
                match line.trim().parse::<usize>() {
                    Ok(n) if n >= 1 && n <= max_index + 1 => return Input::Choice { idx: n - 1 },
                    _ => {
                        println!("Invalid choice, try again.");
                        continue;
                    }
                }
            }
            InputType::Optional(prompt) => {
                print!("\n{prompt} (y/n): ");
                io::stdout().flush().ok();
                let mut line = String::new();
                match handle.read_line(&mut line) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Input error: {e}");
                        continue;
                    }
                }
                match line.trim().to_lowercase().as_str() {
                    "y" | "yes" => return Input::OptionalAccept,
                    "n" | "no" => return Input::OptionalDecline,
                    _ => {
                        println!("Please enter y or n.");
                        continue;
                    }
                }
            }
        }
    }
}

fn print_summary(state: &GameData) {
    println!("\n=== Game Over ===");
    let remaining: Vec<&str> = state
        .players
        .iter()
        .filter_map(|p| {
            if p.in_game {
                Some(p.name.as_str())
            } else {
                None
            }
        })
        .collect();
    println!("Players remaining in game: {}", remaining.len());
    for p in &state.players {
        println!(
            "  {}  score: {}  {}",
            p.name,
            p.score,
            if p.in_game { "in" } else { "out" }
        );
    }
    println!("Cards in play: {}", state.cards.len());
}
