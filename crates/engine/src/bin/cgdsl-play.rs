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
    match run_game(ir, game_data, input_source, None, None) {
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
            InputType::ChoosePlayer { candidates, prompt } => {
                println!("\n--- {prompt} ---");
                for (i, name) in candidates.iter().enumerate() {
                    println!("  {}. {name}", i + 1);
                }
                print!("Enter 1-{}: ", candidates.len());
                io::stdout().flush().ok();
                let mut line = String::new();
                if handle.read_line(&mut line).is_err() {
                    eprintln!("Input error");
                    continue;
                }
                match line.trim().parse::<usize>() {
                    Ok(n) if n >= 1 && n <= candidates.len() => {
                        return Input::ChoosePlayer { idx: n - 1 };
                    }
                    _ => {
                        println!("Invalid choice, try again.");
                        continue;
                    }
                }
            }
            InputType::ChooseCards {
                display,
                min,
                max,
                prompt,
            } => {
                println!("\n--- {prompt} (choose {min}-{max}) ---");
                for (i, card) in display.iter().enumerate() {
                    let desc = card
                        .get("Rank")
                        .or_else(|| card.values().next())
                        .cloned()
                        .unwrap_or_else(|| format!("card {}", i + 1));
                    println!("  {}. {desc}", i + 1);
                }
                print!("Enter comma-separated indices (e.g. 1,3): ");
                io::stdout().flush().ok();
                let mut line = String::new();
                if handle.read_line(&mut line).is_err() {
                    eprintln!("Input error");
                    continue;
                }
                let selected: Option<Vec<usize>> = line
                    .trim()
                    .split(',')
                    .map(|s| s.trim().parse::<usize>().ok())
                    .collect();
                let Some(selected) = selected else {
                    println!("Invalid selection, try again.");
                    continue;
                };
                let zero_based: Vec<usize> =
                    selected.into_iter().map(|n| n.saturating_sub(1)).collect();
                if zero_based.iter().all(|&i| i < display.len())
                    && zero_based.len() >= *min
                    && zero_based.len() <= *max
                {
                    return Input::ChooseCards {
                        selected: zero_based,
                    };
                }
                println!("Selection out of range, try again.");
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
