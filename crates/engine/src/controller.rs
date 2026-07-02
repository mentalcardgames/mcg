use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use front_end::ir::{Ir, LoweredPayLoad};

use crate::game_data::GameData;
use crate::interpreter::{Input, InputType, Interpreter, StepResult, TraceEntry};

#[derive(Clone)]
struct TraceLogger {
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl TraceLogger {
    fn open(path: &PathBuf) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    fn log_entry(&self, step: usize, entry: &TraceEntry) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, "[Step {:3}] {}", step, entry);
            let _ = writer.flush();
        }
    }

    fn log_header(&self, entry: &str, goal: &str, input_source_kind: &str) {
        if let Ok(mut writer) = self.writer.lock() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(writer, "=== MCG Trace Log ===");
            let _ = writeln!(writer, "Started: {}", timestamp);
            let _ = writeln!(writer, "Entry: {}", entry);
            let _ = writeln!(writer, "Goal: {}", goal);
            let _ = writeln!(writer, "Input source: {}", input_source_kind);
            let _ = writeln!(writer, "====================");
            let _ = writer.flush();
        }
    }

    fn log_footer(&self, status: &str) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, "=== {} ===", status);
            let _ = writer.flush();
        }
    }

    fn log_panic(&self, msg: &str) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, "=== Panic: {} ===", msg);
            let _ = writer.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writer.flush();
        }
    }
}

fn resolve_log_path() -> Option<PathBuf> {
    match std::env::var("MCG_TRACE_LOG") {
        Ok(val) => {
            let val = val.trim();
            if val.is_empty() || val.eq_ignore_ascii_case("off") || val.eq_ignore_ascii_case("none")
            {
                None
            } else {
                Some(PathBuf::from(val))
            }
        }
        Err(_) => {
            if cfg!(test) {
                None
            } else {
                Some(PathBuf::from("mcg-trace.log"))
            }
        }
    }
}

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
    trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
) -> Result<GameData, String> {
    let log_path = resolve_log_path();
    let logger = match &log_path {
        Some(path) => match TraceLogger::open(path) {
            Ok(logger) => Some(logger),
            Err(e) => {
                eprintln!(
                    "Warning: failed to open trace log {}: {}",
                    path.display(),
                    e
                );
                None
            }
        },
        None => None,
    };

    let input_source_kind: String = match &input_source {
        InputSource::Player(_) => "interactive".to_string(),
        InputSource::TestFile(path) => path.to_string_lossy().to_string(),
    };

    if let Some(ref logger) = logger {
        logger.log_header(
            &format!("{:?}", ir.entry.raw()),
            &format!("{:?}", ir.goal.raw()),
            &input_source_kind,
        );
    }

    let step_count = Arc::new(std::sync::Mutex::new(0usize));
    let step_count_for_closure = step_count.clone();
    let logger_for_closure = logger.clone();
    let caller_sender = trace_sender;
    let composed_sender: Option<Box<dyn Fn(TraceEntry) + Send>> =
        if logger_for_closure.is_some() || caller_sender.is_some() {
            Some(Box::new(move |entry: TraceEntry| {
                if let Some(ref logger) = logger_for_closure {
                    let step = *step_count_for_closure.lock().unwrap();
                    logger.log_entry(step, &entry);
                }
                if let Some(ref sender) = caller_sender {
                    sender(entry);
                }
            }))
        } else {
            None
        };

    let interpreter = Interpreter::new(ir, game_data, composed_sender);
    let mut controller = Controller {
        interpreter,
        input_source,
        event_sender,
        line_buffer: VecDeque::new(),
        file_loaded: false,
        loaded_line_count: 0,
        input_sequence: 0,
        step_count,
    };

    let result = if logger.is_some() {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| controller.run())).map_err(
            |payload| {
                let msg = if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = payload.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "<non-string panic>".to_string()
                };
                if let Some(ref logger) = logger {
                    logger.log_panic(&msg);
                    logger.flush();
                }
                std::panic::resume_unwind(payload);
            },
        )
    } else {
        Ok(controller.run())
    };

    match result {
        Ok(Ok(gd)) => {
            if let Some(ref logger) = logger {
                logger.log_footer("GameOver");
            }
            Ok(gd)
        }
        Ok(Err(e)) => {
            if let Some(ref logger) = logger {
                logger.log_footer(&format!("Error: {}", e));
            }
            Err(e)
        }
        Err(_) => unreachable!(),
    }
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
    step_count: Arc<std::sync::Mutex<usize>>,
}

impl Controller {
    /// Main event loop: emit the current state, advance the interpreter, and
    /// either continue, supply input, return on game-over, or propagate an error.
    fn run(&mut self) -> Result<GameData, String> {
        loop {
            self.emit_event();
            *self.step_count.lock().unwrap() += 1;

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
            let file = File::open(path)
                .map_err(|e| format!("Failed to open test file {}: {}", path.display(), e))?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line
                    .map_err(|e| format!("Failed to read test file {}: {}", path.display(), e))?;
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
            .ok_or_else(|| format!("Test input file exhausted (input #{})", self.input_sequence))?;

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
                trace_sender: None,
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
            step_count: Arc::new(std::sync::Mutex::new(0)),
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
                trace_sender: None,
            },
            input_source: InputSource::TestFile(PathBuf::from("test_input.txt")),
            event_sender: None,
            line_buffer: VecDeque::from(["1".to_string()]),
            file_loaded: true,
            loaded_line_count: 1,
            input_sequence: 0,
            step_count: Arc::new(std::sync::Mutex::new(0)),
        };

        let path = PathBuf::from("test_input.txt");
        assert!(controller.read_test_file(&path).is_ok());
        let result = controller.read_test_file(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Test input file exhausted (input #0)");
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
        let result = run_game(ir, game_data, InputSource::TestFile(input_path), None, None);

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
            None,
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
            None,
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
