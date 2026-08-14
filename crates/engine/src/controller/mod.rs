use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

use front_end::ir::{Ir, LoweredPayLoad};

use crate::error::EngineError;
use crate::game_data::GameData;
use crate::interpreter::{Input, InputKind, InputType, Interpreter, StepResult, TraceEntry};

mod trace_logger;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

/// Where the game engine gets its player input from.
///
/// - [`Player`](InputSource::Player) — a closure that maps each [`InputType`] request to an [`Input`].
/// - [`TestFile`](InputSource::TestFile) — a file containing pre-recorded responses (one per line).
pub enum InputSource {
    Player(Box<dyn Fn(InputType) -> Input + Send + Sync>),
    TestFile(PathBuf),
}

/// Tunables for a [`run_game_with`] invocation.
///
/// Builder-style: `RunOptions::new().with_event_sender(..).with_trace_sender(..)…`.
/// All fields are optional; `RunOptions::default()` is the no-op configuration.
#[derive(Default)]
#[allow(clippy::type_complexity)] // mirrors the run_game callback types
pub struct RunOptions {
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
    capture_panics: bool,
    log_path: Option<PathBuf>,
    game_name: Option<String>,
}

impl RunOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Per-loop-iteration `&GameData` snapshot callback (coarse-grained).
    pub fn with_event_sender(mut self, sender: Box<dyn Fn(&GameData) + Send>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    /// Per-FSM-transition `TraceEntry` callback (fine-grained).
    pub fn with_trace_sender(mut self, sender: Box<dyn Fn(TraceEntry) + Send>) -> Self {
        self.trace_sender = Some(sender);
        self
    }

    /// Whether panics inside the run loop are caught and converted to
    /// `Err(EngineError::InternalPanic)` instead of aborting the process.
    ///
    /// Default `false` preserves the legacy behavior: a panic is only caught
    /// (for trace-logging, then re-raised via `resume_unwind`) when a trace
    /// log is open; otherwise it propagates untouched. Hosts that want a
    /// guaranteed non-aborting run set this to `true`.
    pub fn capture_panics(mut self, yes: bool) -> Self {
        self.capture_panics = yes;
        self
    }

    /// Force the `MCG_TRACE_LOG` trace file to `path`, overriding the
    /// environment variable.
    ///
    /// Default (`None`): the `MCG_TRACE_LOG` env var is consulted; when it is
    /// unset no trace file is written at all (the library never creates files
    /// in the working directory on its own).
    pub fn with_log_path(mut self, path: PathBuf) -> Self {
        self.log_path = Some(path);
        self
    }

    /// Tag the trace-file header with a human-readable game name.
    ///
    /// Purely cosmetic — the name only appears in the `Game:` header line.
    pub fn with_game_name(mut self, name: impl Into<String>) -> Self {
        self.game_name = Some(name.into());
        self
    }
}

/// Create an [`Interpreter`] from the lowered IR and drive it to completion.
///
/// Convenience wrapper over [`run_game_with`] with no event/trace callbacks and
/// the legacy panic behavior (`capture_panics(false)`).
#[allow(clippy::type_complexity)] // public API: two optional callback closures
pub fn run_game(
    ir: Ir<LoweredPayLoad>,
    game_data: GameData,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
) -> Result<GameData, EngineError> {
    run_game_with(
        ir,
        game_data,
        input_source,
        RunOptions {
            event_sender,
            trace_sender,
            capture_panics: false,
            log_path: None,
            game_name: None,
        },
    )
}

/// Create an [`Interpreter`] from the lowered IR and drive it to completion,
/// with explicit [`RunOptions`].
///
/// `entry` is saved *before* ownership of `ir` moves into the interpreter so the
/// starting state handle can be supplied separately.
#[allow(clippy::type_complexity)] // mirrors the run_game callback types
pub fn run_game_with(
    ir: Ir<LoweredPayLoad>,
    game_data: GameData,
    input_source: InputSource,
    options: RunOptions,
) -> Result<GameData, EngineError> {
    let RunOptions {
        event_sender,
        trace_sender,
        capture_panics,
        log_path,
        game_name,
    } = options;

    let log_path = trace_logger::resolve_log_path(log_path.as_deref());
    let logger = match &log_path {
        Some(path) => match trace_logger::TraceLogger::open(path) {
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
            game_name.as_deref(),
        );
    }

    let step_count = Arc::new(std::sync::Mutex::new(0usize));
    let step_count_for_closure = step_count.clone();
    let steps_reporter = step_count.clone();
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
        input_sequence: 0,
        step_count,
    };

    let should_catch = capture_panics || logger.is_some();
    let run_result = if should_catch {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| controller.run())) {
            Ok(result) => result,
            Err(payload) => {
                // NOTE: pass `payload.as_ref()` (not `&payload`) — the
                // `&Box<dyn Any>` -> `&dyn Any` coercion yields the wrong
                // vtable and every downcast fails.
                let message = panic_message(payload.as_ref());
                if let Some(ref logger) = logger {
                    let steps = *steps_reporter.lock().unwrap();
                    logger.log_panic(&format!("{} (after {} steps)", message, steps));
                    logger.flush();
                }
                if capture_panics {
                    Err(EngineError::InternalPanic { message })
                } else {
                    std::panic::resume_unwind(payload)
                }
            }
        }
    } else {
        controller.run()
    };

    let steps = *steps_reporter.lock().unwrap();
    match run_result {
        Ok(gd) => {
            if let Some(ref logger) = logger {
                // The winner set in the footer (2026-08-10): winners =
                // every player still in game; "none" when nobody won.
                let winners = gd.winner_names();
                let winner_text = if winners.is_empty() {
                    "none".to_string()
                } else {
                    winners.join(", ")
                };
                logger.log_footer(&format!(
                    "GameOver after {} steps — winners: {}",
                    steps, winner_text
                ));
            }
            Ok(gd)
        }
        Err(e) => {
            if let Some(ref logger) = logger {
                logger.log_footer(&format!("Error: {} (after {} steps)", e, steps));
            }
            Err(e)
        }
    }
}

/// Best-effort extraction of the panic message from a `catch_unwind` payload.
fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "<non-string panic>".to_string()
    }
}

/// Drives the [`Interpreter`] forward, supplying external input when required.
#[allow(clippy::type_complexity)] // mirrors the run_game callback types
struct Controller {
    interpreter: Interpreter,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    line_buffer: VecDeque<String>,
    file_loaded: bool,
    input_sequence: usize,
    step_count: Arc<std::sync::Mutex<usize>>,
}

impl Controller {
    /// Main event loop: emit the current state, advance the interpreter, and
    /// either continue, supply input, return on game-over, or propagate an error.
    fn run(&mut self) -> Result<GameData, EngineError> {
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
    fn get_input(&mut self, input_type: InputType) -> Result<Input, EngineError> {
        let current_name = self
            .interpreter
            .game_data
            .get_current_player()
            .map(|p| p.name.as_str())
            .unwrap_or("");

        self.input_sequence += 1;

        let input = match &self.input_source {
            InputSource::Player(callback) => loop {
                let raw = callback(input_type.clone());
                if validate_player_input(&raw, &input_type, current_name) {
                    break raw;
                }
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
    /// - `y`, `yes` → `InputKind::OptionalAccept`
    /// - `n`, `no`  → `InputKind::OptionalDecline`
    /// - `<N>`      → `InputKind::Choice { idx: N-1 }` (1-based choice index)
    /// - `p <N>`    → `InputKind::ChoosePlayer { idx: N-1 }` (1-based candidate index)
    /// - `c <csv>`  → `InputKind::ChooseCards { selected: [..] }` (1-based, comma-separated)
    /// - `n <N>`    → `InputKind::Number { value: N }` (numeric prompt, 2026-08-10)
    ///
    /// Each line may optionally start with a `Name:` prefix (e.g. `P2:y`, `P3:c 1,3`).
    /// Lines without a prefix default to player `"P1"`.
    fn read_test_file(&mut self, path: &PathBuf) -> Result<Input, EngineError> {
        if self.line_buffer.is_empty() && !self.file_loaded {
            let file = File::open(path).map_err(|source| EngineError::TestFileOpen {
                path: path.display().to_string(),
                source,
            })?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.map_err(|source| EngineError::TestFileRead {
                    path: path.display().to_string(),
                    source,
                })?;
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    self.line_buffer.push_back(trimmed.to_string());
                }
            }
            self.file_loaded = true;
        }

        let line = self
            .line_buffer
            .pop_front()
            .ok_or(EngineError::TestInputExhausted {
                input_sequence: self.input_sequence,
            })?;

        let (player_id, body) = if let Some(colon) = line.find(':') {
            if colon > 0 && colon + 1 < line.len() {
                (
                    line[..colon].to_string(),
                    line[colon + 1..].trim_start().to_string(),
                )
            } else {
                ("P1".to_string(), line.clone())
            }
        } else {
            ("P1".to_string(), line.clone())
        };

        let lower = body.to_lowercase();
        let kind = if let Some(rest) = lower.strip_prefix("p ") {
            let rest = rest.trim();
            let n: usize = rest.parse().map_err(|_| EngineError::InvalidTestInputP {
                input_sequence: self.input_sequence,
                line: line.clone(),
            })?;
            if n == 0 {
                return Err(EngineError::InvalidTestInputPlayerZero {
                    input_sequence: self.input_sequence,
                });
            }
            InputKind::ChoosePlayer { idx: n - 1 }
        } else if let Some(rest) = lower.strip_prefix("c ") {
            let rest = rest.trim();
            let selected: Vec<usize> = rest
                .split(',')
                .map(|s| s.trim().parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| EngineError::InvalidTestInputC {
                    input_sequence: self.input_sequence,
                    line: line.clone(),
                })?;
            if selected.contains(&0) {
                return Err(EngineError::InvalidTestInputCardZero {
                    input_sequence: self.input_sequence,
                });
            }
            InputKind::ChooseCards {
                selected: selected.into_iter().map(|n| n - 1).collect(),
            }
        } else if let Some(rest) = lower.strip_prefix("n ") {
            let rest = rest.trim();
            let value: i32 = rest
                .parse()
                .map_err(|_| EngineError::InvalidTestInputNumber {
                    input_sequence: self.input_sequence,
                    line: line.clone(),
                })?;
            InputKind::Number { value }
        } else {
            match lower.as_str() {
                "y" | "yes" => InputKind::OptionalAccept,
                "n" | "no" => InputKind::OptionalDecline,
                _ => {
                    let idx: usize =
                        line.parse()
                            .map_err(|_| EngineError::InvalidTestInputNumber {
                                input_sequence: self.input_sequence,
                                line: line.clone(),
                            })?;
                    if idx == 0 {
                        return Err(EngineError::InvalidTestInputChoiceZero {
                            input_sequence: self.input_sequence,
                        });
                    }
                    InputKind::Choice { idx: idx - 1 }
                }
            }
        };

        Ok(Input { player_id, kind })
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

/// Validates a `Player`-sourced [`Input`] against the [`InputType`] that
/// requested it. Returns `true` to accept (and `break` the loop), `false` to
/// `continue` and re-prompt. Only the `(Input, InputType)` pairs exercised by
/// `get_input`'s `Player` branch are validated; any other combination is
/// accepted, preserving the original behavior. See Stage 6 / sub-task B2.
fn validate_player_input(input: &Input, input_type: &InputType, current_player_name: &str) -> bool {
    if !current_player_name.is_empty() && input.player_id != current_player_name {
        return false;
    }
    match (&input.kind, input_type) {
        (InputKind::Choice { idx }, InputType::Choice { max_index, .. }) => *idx <= *max_index,
        (InputKind::ChoosePlayer { idx }, InputType::ChoosePlayer { candidates, .. }) => {
            *idx < candidates.len()
        }
        (
            InputKind::ChooseCards { selected },
            InputType::ChooseCards {
                display, min, max, ..
            },
        ) => {
            !selected.iter().any(|&i| i >= display.len())
                && selected.len() >= *min
                && selected.len() <= *max
        }
        (InputKind::Number { value }, InputType::Number { min, max, .. }) => {
            min.is_none_or(|m| *value >= m) && max.is_none_or(|m| *value <= m)
        }
        _ => true,
    }
}
