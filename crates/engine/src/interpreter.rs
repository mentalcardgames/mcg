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

#[derive(Clone, Debug)]
pub enum TraceEntry {
    Step {
        from: u32,
        to: u32,
        event: TraceEvent,
    },
}

#[derive(Clone, Debug)]
pub enum TraceEvent {
    Action {
        subtype: String,
        detail: String,
    },
    Choice {
        chosen_idx: usize,
        options: Vec<String>,
    },
    OptionalAccept,
    OptionalDecline,
    Condition {
        expr: String,
        result: bool,
        negated: bool,
        took_else: bool,
    },
    EndCondition {
        expr: String,
        result: bool,
        stage: String,
        exited: bool,
    },
    StageRoundCounter {
        stage: String,
        new_count: u32,
    },
    EndStage {
        stage: String,
    },
    Trigger,
}

impl std::fmt::Display for TraceEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceEntry::Step { from, to, event } => write!(f, "[{from}->{to}] {event}"),
        }
    }
}

impl std::fmt::Display for TraceEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceEvent::Action { subtype, detail } => write!(f, "Action:{} {}", subtype, detail),
            TraceEvent::Choice {
                chosen_idx,
                options,
            } => {
                write!(f, "Choice: chose {} (from {:?})", chosen_idx + 1, options)
            }
            TraceEvent::OptionalAccept => write!(f, "Optional: ACCEPTED"),
            TraceEvent::OptionalDecline => write!(f, "Optional: DECLINED"),
            TraceEvent::Condition {
                expr,
                result,
                negated,
                took_else,
            } => {
                write!(
                    f,
                    "Condition: {} = {} (neg={}, else={})",
                    expr, result, negated, took_else
                )
            }
            TraceEvent::EndCondition {
                expr,
                result,
                stage,
                exited,
            } => {
                write!(
                    f,
                    "EndCondition({}): {} = {} (exited={})",
                    stage, expr, result, exited
                )
            }
            TraceEvent::StageRoundCounter { stage, new_count } => {
                write!(f, "StageRoundCounter: {} -> {}", stage, new_count)
            }
            TraceEvent::EndStage { stage } => write!(f, "EndStage: {}", stage),
            TraceEvent::Trigger => write!(f, "Trigger"),
        }
    }
}

pub struct Interpreter {
    pub ir: Ir<LoweredPayLoad>,
    pub game_data: GameData,
    pub input_buffer: Vec<Input>,
    pub current_state: StateID,
    pub trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
}

impl Interpreter {
    pub fn new(
        ir: Ir<LoweredPayLoad>,
        game_data: GameData,
        trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
    ) -> Self {
        let current_state = ir.entry;
        Self {
            ir,
            game_data,
            input_buffer: Vec::new(),
            current_state,
            trace_sender,
        }
    }

    pub fn step(&mut self) -> StepResult {
        let edges: &Vec<Edge<LoweredPayLoad>> = match self.ir.states.get(&self.current_state) {
            Some(e) => e,
            None => {
                return StepResult::Error(format!(
                    "Current state {:?} not found in IR",
                    self.current_state.raw()
                ))
            }
        };

        if edges.is_empty() {
            if self.current_state == self.ir.goal {
                return StepResult::GameOver;
            }
            return StepResult::Error(format!(
                "No outgoing edges from state {:?} and not at goal state",
                self.current_state.raw()
            ));
        }

        if let Some(edge) = edges.first() {
            let from = self.current_state.raw();
            let to = edge.to.raw();
            match &edge.payload {
                Payload::Action(gr) => {
                    let (subtype, detail) = rule_signature(gr);
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::Action { subtype, detail },
                        });
                    }
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
                Payload::Choice => {
                    if let Some(input) = self.input_buffer.pop() {
                        let options: Vec<String> = edges
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format!("Option {}", i + 1))
                            .collect();
                        if let Some(ref sender) = self.trace_sender {
                            (sender)(TraceEntry::Step {
                                from,
                                to,
                                event: TraceEvent::Choice {
                                    chosen_idx: input.idx(),
                                    options,
                                },
                            });
                        }
                        if let Some(choice_edge) = edges.get(input.idx()) {
                            self.execute_edge(choice_edge.clone());
                        }
                        StepResult::Ok
                    } else {
                        let options: Vec<String> = edges
                            .iter()
                            .enumerate()
                            .map(|(i, _)| format!("Option {}", i + 1))
                            .collect();
                        let max_index = options.len().saturating_sub(1);
                        StepResult::NeedsInput(InputType::Choice { options, max_index })
                    }
                }
                Payload::Optional => {
                    if let Some(input) = self.input_buffer.pop() {
                        if let Some(ref sender) = self.trace_sender {
                            let event = match input {
                                Input::OptionalAccept => TraceEvent::OptionalAccept,
                                Input::OptionalDecline => TraceEvent::OptionalDecline,
                                Input::Choice { .. } => {
                                    return StepResult::Error(
                                        "Unexpected Choice input for Optional".to_string(),
                                    )
                                }
                            };
                            (sender)(TraceEntry::Step { from, to, event });
                        }
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
                        return StepResult::Error(format!(
                            "Condition state {:?} must have exactly 2 edges, found {}",
                            self.current_state.raw(),
                            edges.len()
                        ));
                    }
                    let result = match crate::query::Evaluator::eval_bool(expr, &self.game_data) {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    let should_take_else = result != *negated;
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::Condition {
                                expr: format!("{:?}", expr),
                                result,
                                negated: *negated,
                                took_else: should_take_else,
                            },
                        });
                    }
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
                        return StepResult::Error(format!(
                            "EndCondition state {:?} must have exactly 2 edges, found {}",
                            self.current_state.raw(),
                            edges.len()
                        ));
                    }
                    let result = match crate::query::Evaluator::eval_end_condition(
                        expr,
                        &self.game_data,
                        stage,
                    ) {
                        Ok(r) => r,
                        Err(e) => return StepResult::Error(e),
                    };
                    let should_exit = result == *negated;
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::EndCondition {
                                expr: format!("{:?}", expr),
                                result,
                                stage: stage.clone(),
                                exited: should_exit,
                            },
                        });
                    }
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
                    if self.game_data.get_stage_counter(stage.clone()) == 0 {
                        let all_player_names: Vec<String> = self.game_data.players.iter().map(|p| p.name.clone()).collect();
                        self.game_data.enter_stage(stage.clone(), all_player_names);
                    }
                    self.game_data.increment_stage_counter(stage.clone());
                    let new_count = self.game_data.get_stage_counter(stage.clone());
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::StageRoundCounter {
                                stage: stage.clone(),
                                new_count,
                            },
                        });
                    }
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
                Payload::EndStage(stage) => {
                    self.game_data.leave_stage(stage.clone());
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::EndStage {
                                stage: stage.clone(),
                            },
                        });
                    }
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
                Payload::Trigger => {
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::Trigger,
                        });
                    }
                    self.execute_edge(edge.clone());
                    StepResult::Ok
                }
            }
        } else {
            StepResult::Error(format!(
                "No edges found in state {:?}",
                self.current_state.raw()
            ))
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

pub trait IrExt {
    fn edge_labels(&self, state: StateID) -> Vec<String>;
}

impl IrExt for Ir<LoweredPayLoad> {
    fn edge_labels(&self, state: StateID) -> Vec<String> {
        let edges = match self.states.get(&state) {
            Some(e) => e,
            None => return Vec::new(),
        };
        edges
            .iter()
            .enumerate()
            .map(|(i, edge)| {
                let action_label = if let Some(target_edges) = self.states.get(&edge.to) {
                    if let Some(first_edge) = target_edges.first() {
                        if let Payload::Action(game_rule) = &first_edge.payload {
                            Some(format_action_name(game_rule))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                action_label.unwrap_or_else(|| format!("Option {}", i + 1))
            })
            .collect()
    }
}

fn rule_signature(rule: &front_end::ast::GameRule) -> (String, String) {
    match rule {
        front_end::ast::GameRule::Action { action } => {
            let subtype = match action {
                front_end::ast::ActionRule::FlipAction { .. } => "Action:FlipAction".to_string(),
                front_end::ast::ActionRule::ShuffleAction { .. } => {
                    "Action:ShuffleAction".to_string()
                }
                front_end::ast::ActionRule::OutAction { .. } => "Action:OutAction".to_string(),
                front_end::ast::ActionRule::SetMemory { .. } => "Action:SetMemory".to_string(),
                front_end::ast::ActionRule::ResetMemory { .. } => "Action:ResetMemory".to_string(),
                front_end::ast::ActionRule::CycleAction { .. } => "Action:CycleAction".to_string(),
                front_end::ast::ActionRule::BidAction { .. } => "Action:BidAction".to_string(),
                front_end::ast::ActionRule::BidMemoryAction { .. } => {
                    "Action:BidMemoryAction".to_string()
                }
                front_end::ast::ActionRule::EndAction { .. } => "Action:EndAction".to_string(),
                front_end::ast::ActionRule::DemandAction { .. } => {
                    "Action:DemandAction".to_string()
                }
                front_end::ast::ActionRule::DemandMemoryAction { .. } => {
                    "Action:DemandMemoryAction".to_string()
                }
                front_end::ast::ActionRule::Move { .. } => "Action:Move".to_string(),
            };
            (subtype, format!("{:?}", action))
        }
        front_end::ast::GameRule::SetUp { setup } => {
            let subtype = match setup {
                front_end::ast::SetUpRule::CreatePlayer { .. } => "SetUp:CreatePlayer".to_string(),
                front_end::ast::SetUpRule::CreateTeams { .. } => "SetUp:CreateTeams".to_string(),
                front_end::ast::SetUpRule::CreateTurnorder { .. } => {
                    "SetUp:CreateTurnorder".to_string()
                }
                front_end::ast::SetUpRule::CreateTurnorderRandom { .. } => {
                    "SetUp:CreateTurnorderRandom".to_string()
                }
                front_end::ast::SetUpRule::CreateLocation { .. } => {
                    "SetUp:CreateLocation".to_string()
                }
                front_end::ast::SetUpRule::CreateCardOnLocation { .. } => {
                    "SetUp:CreateCardOnLocation".to_string()
                }
                front_end::ast::SetUpRule::CreateTokenOnLocation { .. } => {
                    "SetUp:CreateTokenOnLocation".to_string()
                }
                front_end::ast::SetUpRule::CreateCombo { .. } => "SetUp:CreateCombo".to_string(),
                front_end::ast::SetUpRule::CreateMemory { .. } => "SetUp:CreateMemory".to_string(),
                front_end::ast::SetUpRule::CreateMemoryWithMemoryType { .. } => {
                    "SetUp:CreateMemoryWithMemoryType".to_string()
                }
                front_end::ast::SetUpRule::CreatePrecedence { .. } => {
                    "SetUp:CreatePrecedence".to_string()
                }
                front_end::ast::SetUpRule::CreatePointMap { .. } => {
                    "SetUp:CreatePointMap".to_string()
                }
            };
            (subtype, format!("{:?}", setup))
        }
        front_end::ast::GameRule::Scoring { scoring } => {
            let subtype = match scoring {
                front_end::ast::ScoringRule::ScoreRule { .. } => "Scoring:ScoreRule".to_string(),
                front_end::ast::ScoringRule::WinnerRule { .. } => "Scoring:WinnerRule".to_string(),
            };
            (subtype, format!("{:?}", scoring))
        }
    }
}

fn format_action_name(rule: &front_end::ast::GameRule) -> String {
    let (subtype, _) = rule_signature(rule);
    subtype
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone)]
pub enum InputType {
    Choice {
        options: Vec<String>,
        max_index: usize,
    },
    Optional(String),
}
