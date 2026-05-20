use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use front_end::ir::{Ir, LoweredPayLoad};

use crate::game_data::GameData;
use crate::interpreter::{Input, InputType, Interpreter, StepResult};

pub enum InputSource {
    Player(Box<dyn Fn(InputType) -> Input + Send + Sync>),
    TestFile(PathBuf),
}

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

struct Controller {
    interpreter: Interpreter,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    line_buffer: Vec<String>,
    file_loaded: bool,
}

impl Controller {
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

    fn get_input(&mut self, input_type: InputType) -> Result<Input, String> {
        match &self.input_source {
            InputSource::Player(callback) => Ok(callback(input_type)),
            InputSource::TestFile(path) => {
                let path = path.clone();
                self.read_test_file(&path)
            }
        }
    }

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

    fn emit_event(&self) {
        if let Some(sender) = &self.event_sender {
            sender(&self.interpreter.game_data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
