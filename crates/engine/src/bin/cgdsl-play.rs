use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use cgdsl_engine::{
    format_game_data, run_game_with, DebugLevel, GameData, Input, InputKind, InputSource,
    InputType, RunOptions,
};
use front_end::validation::parse_document;

/// Process exit codes, distinct per failure stage so scripts can react.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExitCode {
    Ok = 0,
    /// Bad command-line usage (unknown flag, missing game file, bad value).
    Usage = 1,
    /// The game file could not be read.
    ReadError = 2,
    /// The `.cgdsl` source failed to parse.
    ParseError = 3,
    /// The engine terminated with an `EngineError` (or panicked).
    EngineError = 4,
}

impl ExitCode {
    fn exit(self) -> ! {
        std::process::exit(self as i32);
    }
}

/// Parsed command line.
#[derive(Debug, PartialEq)]
struct Cli {
    game_file: PathBuf,
    input_file: Option<PathBuf>,
    log_path: Option<PathBuf>,
    debug_level: Option<DebugLevel>,
}

fn usage(prog: &str) -> String {
    format!(
        "\
Usage: {prog} [OPTIONS] <game.cgdsl> [input.txt]

Drives a .cgdsl game to completion: interactive (or file-driven) play,
then prints the final state summary.

Options:
  --log <path>          write the MCG trace log to <path> (overrides MCG_TRACE_LOG)
  --debug-level <L>     after the run, print the full GameData dump at
                        level `low`, `medium`, or `high`
  -h, --help            show this help

Exit codes:
  0  game completed
  1  usage error
  2  could not read the game file
  3  parse error in the .cgdsl source
  4  the engine returned an error
"
    )
}

/// Hand-rolled argument parsing: two positionals plus `--flag value` options.
fn parse_args(args: &[String]) -> Result<Cli, String> {
    let mut game_file: Option<PathBuf> = None;
    let mut input_file: Option<PathBuf> = None;
    let mut log_path: Option<PathBuf> = None;
    let mut debug_level: Option<DebugLevel> = None;

    let mut it = args.iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err("help".to_string()),
            "--log" => {
                let value = it
                    .next()
                    .ok_or_else(|| "--log requires a path argument".to_string())?;
                log_path = Some(PathBuf::from(value));
            }
            "--debug-level" => {
                let value = it
                    .next()
                    .ok_or_else(|| "--debug-level requires an argument".to_string())?;
                debug_level = Some(match value.to_ascii_lowercase().as_str() {
                    "low" => DebugLevel::Low,
                    "medium" => DebugLevel::Medium,
                    "high" => DebugLevel::High,
                    _ => {
                        return Err(format!(
                            "invalid --debug-level '{value}' (expected low|medium|high)"
                        ))
                    }
                });
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option '{arg}'"));
            }
            _ => {
                if game_file.is_none() {
                    game_file = Some(PathBuf::from(arg));
                } else if input_file.is_none() {
                    input_file = Some(PathBuf::from(arg));
                } else {
                    return Err(format!("unexpected extra argument '{arg}'"));
                }
            }
        }
    }

    let game_file = game_file.ok_or_else(|| "missing <game.cgdsl> argument".to_string())?;
    Ok(Cli {
        game_file,
        input_file,
        log_path,
        debug_level,
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog = args
        .first()
        .cloned()
        .unwrap_or_else(|| "cgdsl-play".to_string());

    let cli = match parse_args(&args) {
        Ok(cli) => cli,
        Err(msg) if msg == "help" => {
            print!("{}", usage(&prog));
            ExitCode::Ok.exit();
        }
        Err(msg) => {
            eprintln!("{prog}: {msg}");
            eprintln!("Try '{prog} --help' for usage.");
            ExitCode::Usage.exit();
        }
    };

    let game_name = cli
        .game_file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let source = match std::fs::read_to_string(&cli.game_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{prog}: error reading `{}`: {e}", cli.game_file.display());
            ExitCode::ReadError.exit();
        }
    };

    let game = match parse_document(&source) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{prog}: parse error in `{}`:\n{e}", cli.game_file.display());
            ExitCode::ParseError.exit();
        }
    };
    let ir = game.to_lowered_graph();

    let player_name: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let pn_writer = player_name.clone();
    let state_sender = Box::new(move |gd: &GameData| {
        *pn_writer.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
    }) as Box<dyn Fn(&GameData) + Send>;

    let pn_reader = player_name.clone();
    let input_source = match &cli.input_file {
        Some(path) => InputSource::TestFile(path.clone()),
        None => InputSource::Player(Box::new(move |it: InputType| {
            let name = pn_reader
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".to_string());
            interactive_input(it, &name)
        })),
    };

    let mut options = RunOptions::new().with_event_sender(state_sender);
    if let Some(path) = &cli.log_path {
        options = options.with_log_path(path.clone());
    }
    if !game_name.is_empty() {
        options = options.with_game_name(game_name);
    }

    let game_data = GameData::new();
    match run_game_with(ir, game_data, input_source, options) {
        Ok(state) => {
            print_summary(&state);
            if let Some(level) = cli.debug_level {
                println!("\n{}", format_game_data(&state, level));
            }
        }
        Err(e) => {
            eprintln!("{prog}: game error: {e}");
            ExitCode::EngineError.exit();
        }
    }
}

fn interactive_input(input_type: InputType, player_name: &str) -> Input {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let player_id = player_name.to_string();
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
                    Ok(n) if n >= 1 && n <= max_index + 1 => {
                        return Input {
                            player_id,
                            kind: InputKind::Choice { idx: n - 1 },
                        };
                    }
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
                    "y" | "yes" => {
                        return Input {
                            player_id,
                            kind: InputKind::OptionalAccept,
                        };
                    }
                    "n" | "no" => {
                        return Input {
                            player_id,
                            kind: InputKind::OptionalDecline,
                        };
                    }
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
                        return Input {
                            player_id,
                            kind: InputKind::ChoosePlayer { idx: n - 1 },
                        };
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
                    return Input {
                        player_id,
                        kind: InputKind::ChooseCards {
                            selected: zero_based,
                        },
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        let mut v = vec!["cgdsl-play".to_string()];
        v.extend(items.iter().map(|s| s.to_string()));
        v
    }

    #[test]
    fn parses_positional_game_and_input() {
        let cli = parse_args(&args(&["game.cgdsl", "input.txt"])).unwrap();
        assert_eq!(cli.game_file, PathBuf::from("game.cgdsl"));
        assert_eq!(cli.input_file, Some(PathBuf::from("input.txt")));
        assert!(cli.log_path.is_none());
        assert!(cli.debug_level.is_none());
    }

    #[test]
    fn parses_flags_in_any_order() {
        let cli = parse_args(&args(&[
            "--log",
            "trace.log",
            "game.cgdsl",
            "--debug-level",
            "HIGH",
            "input.txt",
        ]))
        .unwrap();
        assert_eq!(cli.log_path, Some(PathBuf::from("trace.log")));
        assert_eq!(cli.debug_level, Some(DebugLevel::High));
        assert_eq!(cli.input_file, Some(PathBuf::from("input.txt")));
    }

    #[test]
    fn help_flag_is_reported() {
        assert_eq!(parse_args(&args(&["--help"])), Err("help".to_string()));
        assert_eq!(parse_args(&args(&["-h"])), Err("help".to_string()));
    }

    #[test]
    fn missing_game_file_is_a_usage_error() {
        assert!(parse_args(&args(&[])).is_err());
        assert!(parse_args(&args(&["--log", "x.log"])).is_err());
    }

    #[test]
    fn unknown_flag_and_bad_values_are_usage_errors() {
        assert!(parse_args(&args(&["--bogus", "game.cgdsl"])).is_err());
        assert!(parse_args(&args(&["--debug-level", "verbose", "game.cgdsl"])).is_err());
        assert!(parse_args(&args(&["--log"])).is_err());
        assert!(parse_args(&args(&["a.cgdsl", "b.txt", "c.txt"])).is_err());
    }
}
