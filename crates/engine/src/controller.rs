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
        line_buffer: Vec::new(),
        file_loaded: false,
    };
    controller.run()
}

/// Drives the [`Interpreter`] forward, supplying external input when required.
struct Controller {
    /// The core interpreter that executes the state machine.
    interpreter: Interpreter,
    /// Where to obtain player decisions when the interpreter asks for one.
    input_source: InputSource,
    /// Optional callback invoked after every interpreter step so callers can
    /// react to the evolving game state (e.g. push a UI update).
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    /// Lines read from a test file, consumed in LIFO order (stack).
    line_buffer: Vec<String>,
    /// Whether the test file has already been read into `line_buffer`.
    file_loaded: bool,
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
        match &self.input_source {
            InputSource::Player(callback) => Ok(callback(input_type)),
            InputSource::TestFile(path) => {
                let path = path.clone();
                self.read_test_file(&path)
            }
        }
    }

    /// Read a test input file into `line_buffer` on first call, then pop lines
    /// one-by-one and parse them as [`Input`] values.
    ///
    /// Lines are consumed in **reverse** order (LIFO) because the buffer is used
    /// as a stack. Blank lines and lines starting with `#` are ignored.
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
                    self.line_buffer.push(trimmed.to_string());
                }
            }
            self.file_loaded = true;
        }

        let line = self
            .line_buffer
            .pop()
            .ok_or_else(|| "Test input file exhausted".to_string())?;

        let consumed_lines = self.line_buffer.len() + 1;

        match line.to_lowercase().as_str() {
            "y" | "yes" => Ok(Input::OptionalAccept),
            "n" | "no" => Ok(Input::OptionalDecline),
            _ => {
                let idx: usize = line.parse().map_err(|_| {
                    format!(
                        "Invalid test input at line {}: expected number, 'y', or 'n', got '{}'",
                        consumed_lines, line
                    )
                })?;
                if idx == 0 {
                    return Err(format!(
                        "Invalid test input at line {}: choice indices start at 1, got 0",
                        consumed_lines
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
    /// lines into the corresponding [`Input`] variants (LIFO order).
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
            line_buffer: vec![
                "n".to_string(),
                "2".to_string(),
                "y".to_string(),
                "1".to_string(),
            ],
            file_loaded: true,
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
            line_buffer: vec!["1".to_string()],
            file_loaded: true,
        };

        let path = PathBuf::from("test_input.txt");
        assert!(controller.read_test_file(&path).is_ok());
        let result = controller.read_test_file(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Test input file exhausted");
    }
}
