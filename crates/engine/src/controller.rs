use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use front_end::ir::{Ir, LoweredPayLoad};

use crate::game_data::GameData;
use crate::interpreter::{Input, InputType, Interpreter, StepResult};

/// Where the game engine gets its player input from.
///
/// - [`Player`](InputSource::Player) — a closure that maps each [`InputType`] request to an [`Input`].
/// - [`TestFile`](InputSource::TestFile) — a file containing pre-recorded responses (one per line).
pub enum InputSource {
    Player(Box<dyn Fn(InputType) -> Input + Send + Sync>),
    TestFile(PathBuf),
}

/// Create an [`Interpreter`] from the lowered IR and drive it to completion.
///
/// `entry` is saved *before* ownership of `ir` moves into the interpreter so the
/// starting state handle can be supplied separately.
pub fn run_game(
    ir: Ir<LoweredPayLoad>,
    game_data: GameData,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
) -> Result<GameData, String> {
    let entry = ir.entry;
    let mut controller = Controller {
        interpreter: Interpreter {
            ir,
            game_data,
            input_buffer: Vec::new(),
            current_state: entry,
        },
        input_source,
        event_sender,
        line_buffer: VecDeque::new(),
        file_loaded: false,
        loaded_line_count: 0,
        input_sequence: 0,
    };
    controller.run()
}

/// Drives the [`Interpreter`] forward, supplying external input when required.
struct Controller {
    interpreter: Interpreter,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    line_buffer: VecDeque<String>,
    file_loaded: bool,
    loaded_line_count: usize,
    input_sequence: usize,
}

impl Controller {
    /// Main event loop: emit the current state, advance the interpreter, and
    /// either continue, supply input, return on game-over, or propagate an error.
    fn run(&mut self) -> Result<GameData, String> {
        loop {
            self.emit_event();

            match self.interpreter.step() {
                StepResult::Ok => continue,
                StepResult::NeedsInput(input_type) => {
                    let input = self.get_input(input_type)?;
                    self.interpreter.provide_input(input);
                }
                StepResult::GameOver => {
                    self.emit_event();
                    return Ok(self.interpreter.game_data.clone());
                }
                StepResult::Error(e) => return Err(e),
            }
        }
    }

    /// Route an input request to the active [`InputSource`].
    fn get_input(&mut self, input_type: InputType) -> Result<Input, String> {
        self.input_sequence += 1;

        let input = match &self.input_source {
            InputSource::Player(callback) => loop {
                let raw = callback(input_type.clone());
                if let Input::Choice { idx } = &raw {
                    if let InputType::Choice { max_index, .. } = &input_type {
                        if idx > max_index {
                            continue;
                        }
                    }
                }
                break raw;
            },
            InputSource::TestFile(path) => {
                let path = path.clone();
                self.read_test_file(&path)?
            }
        };

        Ok(input)
    }

    /// Read a test input file into `line_buffer` on first call, then consume
    /// lines one-by-one (FIFO) and parse them as [`Input`] values.
    ///
    /// The first non-blank, non-comment line in the file supplies the first input.
    /// Blank lines and lines starting with `#` are ignored.
    ///
    /// Accepted line formats:
    /// - `y`, `yes` → `Input::OptionalAccept`
    /// - `n`, `no`  → `Input::OptionalDecline`
    /// - `<N>`      → `Input::Choice { idx: N-1 }` (1-based choice index)
    fn read_test_file(&mut self, path: &PathBuf) -> Result<Input, String> {
        if self.line_buffer.is_empty() && !self.file_loaded {
            let file = File::open(path).map_err(|e| format!("Failed to open test file: {}", e))?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.map_err(|e| format!("Failed to read test file: {}", e))?;
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    self.line_buffer.push_back(trimmed.to_string());
                }
            }
            self.loaded_line_count = self.line_buffer.len();
            self.file_loaded = true;
        }

        let line = self
            .line_buffer
            .pop_front()
            .ok_or_else(|| "Test input file exhausted".to_string())?;

        match line.to_lowercase().as_str() {
            "y" | "yes" => Ok(Input::OptionalAccept),
            "n" | "no" => Ok(Input::OptionalDecline),
            _ => {
                let idx: usize = line.parse().map_err(|_| {
                    format!(
                        "Invalid test input #{}: expected number, 'y', or 'n', got '{}'",
                        self.input_sequence, line
                    )
                })?;
                if idx == 0 {
                    return Err(format!(
                        "Invalid test input #{}: choice indices start at 1, got 0",
                        self.input_sequence
                    ));
                }
                Ok(Input::Choice { idx: idx - 1 })
            }
        }
    }

    /// Invoke the optional event callback with the current [`GameData`].
    ///
    /// This allows front-ends (GUI, CLI logging, etc.) to react to every
    /// interpreter step without polling.
    fn emit_event(&self) {
        if let Some(sender) = &self.event_sender {
            sender(&self.interpreter.game_data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that [`read_test_file`] correctly parses `y`, `n`, and numeric
    /// lines in FIFO order into the corresponding [`Input`] variants.
    #[test]
    fn test_input_parsing() {
        let default_ir = Ir::<LoweredPayLoad>::default();
        let mut controller = Controller {
            interpreter: Interpreter {
                ir: Ir {
                    states: Default::default(),
                    entry: default_ir.entry,
                    goal: default_ir.goal,
                },
                game_data: GameData::new(),
                input_buffer: Vec::new(),
                current_state: default_ir.entry,
            },
            input_source: InputSource::TestFile(PathBuf::from("/nonexistent")),
            event_sender: None,
            line_buffer: VecDeque::from([
                "1".to_string(),
                "y".to_string(),
                "2".to_string(),
                "n".to_string(),
            ]),
            file_loaded: true,
            loaded_line_count: 4,
            input_sequence: 0,
        };

        let path = PathBuf::from("/nonexistent");
        assert_eq!(
            controller.read_test_file(&path).unwrap(),
            Input::Choice { idx: 0 }
        );
        assert_eq!(
            controller.read_test_file(&path).unwrap(),
            Input::OptionalAccept
        );
        assert_eq!(
            controller.read_test_file(&path).unwrap(),
            Input::Choice { idx: 1 }
        );
        assert_eq!(
            controller.read_test_file(&path).unwrap(),
            Input::OptionalDecline
        );
    }

    /// Ensure that popping from an empty `line_buffer` produces the expected
    /// `"Test input file exhausted"` error.
    #[test]
    fn test_input_exhausted_error() {
        let default_ir = Ir::<LoweredPayLoad>::default();
        let mut controller = Controller {
            interpreter: Interpreter {
                ir: Ir {
                    states: Default::default(),
                    entry: default_ir.entry,
                    goal: default_ir.goal,
                },
                game_data: GameData::new(),
                input_buffer: Vec::new(),
                current_state: default_ir.entry,
            },
            input_source: InputSource::TestFile(PathBuf::from("test_input.txt")),
            event_sender: None,
            line_buffer: VecDeque::from(["1".to_string()]),
            file_loaded: true,
            loaded_line_count: 1,
            input_sequence: 0,
        };

        let path = PathBuf::from("test_input.txt");
        assert!(controller.read_test_file(&path).is_ok());
        let result = controller.read_test_file(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Test input file exhausted");
    }

    #[test]
    fn test_input_file_ordering_and_validation() {
        use front_end::validation::parse_document;

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
        let input_path = manifest_dir.join("test_games/ordering_test.txt");

        let source = std::fs::read_to_string(&game_path).expect("Failed to read test game file");
        let game = parse_document(&source).expect("Failed to parse game");
        let ir = game.to_lowered_graph();

        let game_data = GameData::new();
        let result = run_game(ir, game_data, InputSource::TestFile(input_path), None);

        assert!(result.is_ok(), "Game should complete successfully");
    }

    #[test]
    fn test_debug_integration_game_snapshots() {
        use crate::debug::{format_game_data, DebugLevel};
        use front_end::validation::parse_document;
        use std::sync::{Arc, RwLock};

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
        let input_path = manifest_dir.join("test_games/ordering_test.txt");

        let source = std::fs::read_to_string(&game_path).expect("Failed to read test game file");
        let game = parse_document(&source).expect("Failed to parse game");
        let ir = game.to_lowered_graph();

        let snapshots = Arc::new(RwLock::new(Vec::new()));
        let snapshots_clone = snapshots.clone();
        let game_data = GameData::new();

        let result = run_game(
            ir,
            game_data,
            InputSource::TestFile(input_path),
            Some(Box::new(move |gd| {
                snapshots_clone.write().unwrap().push(gd.clone());
            })),
        );

        assert!(result.is_ok());
        assert!(!snapshots.read().unwrap().is_empty());

        let output_low = format_game_data(&snapshots.read().unwrap()[0], DebugLevel::Low);
        assert!(!output_low.is_empty());
        assert!(output_low.contains("GAME DATA (LOW)"));

        let output_medium = format_game_data(&snapshots.read().unwrap()[0], DebugLevel::Medium);
        assert!(!output_medium.is_empty());
        assert!(output_medium.contains("GAME DATA (MEDIUM)"));

        let output_high = format_game_data(&snapshots.read().unwrap()[0], DebugLevel::High);
        assert!(!output_high.is_empty());
        assert!(output_high.contains("GAME DATA (HIGH)"));
    }

    #[test]
    fn test_debug_integration_verify_game_progression() {
        use crate::debug::{format_game_data, DebugLevel};
        use front_end::validation::parse_document;
        use std::sync::{Arc, RwLock};

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
        let input_path = manifest_dir.join("test_games/ordering_test.txt");

        let source = std::fs::read_to_string(&game_path).expect("Failed to read test game file");
        let game = parse_document(&source).expect("Failed to parse game");
        let ir = game.to_lowered_graph();

        let snapshots = Arc::new(RwLock::new(Vec::new()));
        let snapshots_clone = snapshots.clone();
        let game_data = GameData::new();

        let result = run_game(
            ir,
            game_data,
            InputSource::TestFile(input_path),
            Some(Box::new(move |gd| {
                snapshots_clone.write().unwrap().push(gd.clone());
            })),
        );

        assert!(result.is_ok());

        let first_snapshot = &snapshots.read().unwrap()[0];
        let first_output = format_game_data(first_snapshot, DebugLevel::Low);

        assert!(first_output.contains("Players:"));

        if snapshots.read().unwrap().len() > 1 {
            let second_snapshot = &snapshots.read().unwrap()[1];
            let second_output = format_game_data(second_snapshot, DebugLevel::Low);

            assert!(
                second_output.contains("Players:"),
                "All snapshots should contain player info"
            );

            assert!(
                first_output != second_output,
                "Snapshots should show different game states"
            );
        }
    }
}
