/*
In interpreter.rs, we define the logic for executing a single step of the game.
The main function is step(), which takes the current game state and an input buffer, and returns a StepResult enum.
The StepResult enum has the following variants:
 - Ok: the step was executed successfully, and the game state was modified accordingly.
 - NeedsInput: the step requires input from the player, and the game state is waiting for the next input.
 - GameOver: the step resulted in the end of the game, and the game state is now in a terminal state.
 - Error: there was an error executing the step, and the game state may be in an inconsistent state.
*/

use front_end::ir::{Edge, Ir, LoweredPayLoad, Payload, StateID};

use crate::game_data::GameData;

pub struct Interpreter {
    pub ir: Ir<LoweredPayLoad>,
    pub game_data: GameData,
    pub input_buffer: Vec<Input>,
    pub current_state: StateID,
}

impl Interpreter {
    pub fn step(&mut self) -> StepResult {
        let edges: &Vec<Edge<LoweredPayLoad>> = match self.ir.states.get(&self.current_state) {
            Some(e) => e,
            None => return StepResult::Error("Current state not found in IR".to_string()),
        };

        if edges.is_empty() {
            if self.current_state == self.ir.goal {
                return StepResult::GameOver;
            }
            return StepResult::Error("No outgoing edges and not at goal state".to_string());
        }

        if let Some(edge) = edges.first() {
            match &edge.payload {
                Payload::Action(_) => {
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
                Payload::Choice => {
                    if let Some(input) = self.input_buffer.pop() {
                        if let Some(choice_edge) = edges.get(input.idx()) {
                            self.execute_edge(choice_edge.clone());
                        }
                        StepResult::Ok
                    } else {
                        // here we can later generate option labels by looking down the IR tree by one state to the next action edge for each option.
                        let options: Vec<String> = edges
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format!("Option {}", i + 1))
                            .collect();
                        StepResult::NeedsInput(InputType::Choice(options))
                    }
                }
                Payload::Optional => {
                    if let Some(input) = self.input_buffer.pop() {
                        if let Some(opt_edge) = edges.get(input.idx()) {
                            self.execute_edge(opt_edge.clone());
                        }
                        StepResult::Ok
                    } else {
                        StepResult::NeedsInput(InputType::Optional(
                            "Do you want to take this optional action? (y/n)".to_string(),
                        ))
                    }
                }
                Payload::Condition { expr, negated } => {
                    if edges.len() != 2 {
                        return StepResult::Error(
                            "Condition state must have exactly 2 edges".to_string(),
                        );
                    }
                    let result = match crate::query::Evaluator::eval_bool(expr, &self.game_data) {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    let should_take_else = result != *negated;
                    let edge = if should_take_else {
                        edges.get(1)
                    } else {
                        edges.get(0)
                    };
                    if let Some(e) = edge {
                        self.execute_edge(e.clone());
                        StepResult::Ok
                    } else {
                        StepResult::Error("Failed to get condition edge".to_string())
                    }
                }
                Payload::EndCondition {
                    expr,
                    negated,
                    stage,
                } => {
                    if edges.len() != 2 {
                        return StepResult::Error(
                            "EndCondition state must have exactly 2 edges".to_string(),
                        );
                    }
                    let result = match crate::query::Evaluator::eval_end_condition(
                        expr,
                        &self.game_data,
                        stage,
                    ) {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    let should_exit = result != *negated;
                    let edge = if should_exit {
                        edges.get(0)
                    } else {
                        edges.get(1)
                    };
                    if let Some(e) = edge {
                        self.execute_edge(e.clone());
                        StepResult::Ok
                    } else {
                        StepResult::Error("Failed to get end condition edge".to_string())
                    }
                }
                Payload::StageRoundCounter(stage) => {
                    self.game_data.increment_stage_counter(stage.clone());
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
                Payload::EndStage(stage) => {
                    self.game_data.leave_stage(stage.clone());
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
                Payload::Trigger => {
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
            }
        } else {
            StepResult::Error("No edges found".to_string())
        }
    }

    pub fn execute_edge(&mut self, edge: Edge<LoweredPayLoad>) {
        self.current_state = edge.to;
        crate::action::execute(edge.payload, &mut self.game_data);
    }

    /// Pushes input to the input buffer.
    pub fn provide_input(&mut self, input: Input) {
        self.input_buffer.push(input);
    }
}

#[derive(Clone)]
pub enum Input {
    Choice { idx: usize },
    OptionalAccept,
    OptionalDecline,
}

impl Input {
    pub fn idx(&self) -> usize {
        match self {
            Input::Choice { idx } => *idx,
            Input::OptionalAccept => 0,
            Input::OptionalDecline => 1,
        }
    }
}

pub enum StepResult {
    Ok,
    NeedsInput(InputType),
    GameOver,
    Error(String),
}

pub enum InputType {
    Choice(Vec<String>),
    Optional(String),
}
