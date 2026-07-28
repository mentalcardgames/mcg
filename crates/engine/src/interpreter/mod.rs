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

mod ir_ext;
mod quant_driver;
mod trace;
mod types;

pub use ir_ext::IrExt;
pub use trace::{TraceEntry, TraceEvent};
pub use types::{Input, InputKind, InputType, StepResult};

use crate::interpreter::ir_ext::{payload_label, rule_signature};

pub struct Interpreter {
    pub ir: Ir<LoweredPayLoad>,
    pub game_data: GameData,
    pub input_buffer: Vec<Input>,
    pub current_state: StateID,
    pub trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
    /// Ephemeral overlay of synthetic replacement edges, keyed only by
    /// synthetic `StateID`s allocated from `next_synth`. Real IR ids are
    /// never keys here, so the overlay never shadows `ir.states`. See
    /// `crate::quantifier`.
    pub pending_overlay: std::collections::HashMap<StateID, Vec<Edge<LoweredPayLoad>>>,
    /// Counter for synthetic `StateID` allocation. Seeded at `u32::MAX - 1`
    /// and decremented (via `wrapping_sub`) so synthetic ids never collide
    /// with the densely-allocated-from-0 real ids.
    pub next_synth: u32,
    /// In-flight quantifier awaiting a player-input round-trip, if any.
    pub pending_quant: Option<crate::quantifier::PendingQuant>,
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
            pending_overlay: std::collections::HashMap::new(),
            next_synth: u32::MAX - 1,
            pending_quant: None,
        }
    }

    pub fn step(&mut self) -> StepResult {
        // (A) Drop the quantifier overlay's synthetic memory slot once we're
        // back on a real IR state. The slot is written just before dispatching
        // a replacement edge and must not outlive the quantifier edge (a user
        // `.cgdsl` program might later `CreateMemory` with the same key).
        if self.ir.states.contains_key(&self.current_state)
            && !self.pending_overlay.contains_key(&self.current_state)
            && self
                .game_data
                .memories
                .contains_key(crate::quantifier::SYNTH_MEMORY_KEY)
        {
            self.game_data
                .memories
                .remove(crate::quantifier::SYNTH_MEMORY_KEY);
        }

        // (B) Overlay dispatch: a synthetic state has its replacement edge(s)
        // in the overlay. Dispatch the first through the normal execute path so
        // per-player / per-card tracing fires for each synthetic transition.
        if let Some(edge) = self
            .pending_overlay
            .get(&self.current_state)
            .and_then(|edges| edges.first().cloned())
        {
            let from = self.current_state.raw();
            let to = edge.to.raw();
            if let Payload::Action(gr) = &edge.payload {
                let (subtype, detail) = rule_signature(gr);
                if let Some(ref sender) = self.trace_sender {
                    (sender)(TraceEntry::Step {
                        from,
                        to,
                        event: TraceEvent::Action { subtype, detail },
                    });
                }
            }
            self.execute_edge(edge);
            return StepResult::Ok;
        }

        // (C0) Resume an in-flight quantifier prompt if its input has arrived.
        if let Some(resumed) = self.take_quant_resume() {
            return resumed;
        }

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
            // (C1) Quantifier preprocessor: if the edge carries a quantifier
            // site, hand off to the dedicated arms (which either issue a
            // `NeedsInput` prompt or build a synthetic fan-out chain). Real
            // IR ids and synthetic ids never collide (see `quantifier::alloc_synth`).
            let site = crate::quantifier::scan_edge(edge);
            if site != crate::quantifier::QuantSite::None {
                let edge_owned = edge.clone();
                return match site {
                    crate::quantifier::QuantSite::DestPlayerAll { pc } => {
                        self.step_dest_player_all(&edge_owned, pc)
                    }
                    crate::quantifier::QuantSite::DestPlayerAny { pc } => {
                        self.step_dest_player_any(&edge_owned, pc)
                    }
                    crate::quantifier::QuantSite::SrcCardsAnyOrRange { qty, from } => {
                        self.step_src_cards_any_or_range(&edge_owned, qty, from)
                    }
                    crate::quantifier::QuantSite::None => unreachable!(),
                };
            }
            let from = self.current_state.raw();
            let to = edge.to.raw();
            match &edge.payload {
                Payload::Action(gr) => {
                    if let front_end::ast::GameRule::SetUp { setup } = gr {
                        if crate::quantifier::setup_contains_any(setup) {
                            return StepResult::Error(
                                "quantifier 'any' is not supported in setup rules".to_string(),
                            );
                        }
                    }
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
                    let options: Vec<String> = self.ir.edge_labels(self.current_state);
                    if let Some(input) = self.input_buffer.pop() {
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
                        let max_index = options.len().saturating_sub(1);
                        StepResult::NeedsInput(InputType::Choice { options, max_index })
                    }
                }
                Payload::Optional => {
                    if let Some(input) = self.input_buffer.pop() {
                        if let Some(ref sender) = self.trace_sender {
                            let event = match &input.kind {
                                InputKind::OptionalAccept => TraceEvent::OptionalAccept,
                                InputKind::OptionalDecline => TraceEvent::OptionalDecline,
                                InputKind::Choice { .. }
                                | InputKind::ChoosePlayer { .. }
                                | InputKind::ChooseCards { .. } => {
                                    return StepResult::Error(
                                        "Unexpected input for Optional".to_string(),
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
                        let prompt = edges
                            .first()
                            .and_then(|acc| self.ir.states.get(&acc.to))
                            .and_then(|tgt| tgt.first())
                            .map(|e| {
                                format!("Do you want to: {}? (y/n)", payload_label(&e.payload))
                            })
                            .unwrap_or_else(|| {
                                "Do you want to take this optional action? (y/n)".to_string()
                            });
                        StepResult::NeedsInput(InputType::Optional(prompt))
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
                        edges.first()
                    } else {
                        edges.get(1)
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
                    self.game_data.ensure_stage_entered(stage);
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
                        edges.first()
                    } else {
                        edges.get(1)
                    };
                    if should_exit {
                        self.game_data.leave_stage(stage.clone());
                    }
                    if let Some(e) = edge {
                        self.execute_edge(e.clone());
                        StepResult::Ok
                    } else {
                        StepResult::Error("Failed to get end condition edge".to_string())
                    }
                }
                Payload::StageRoundCounter(stage) => {
                    self.game_data.ensure_stage_entered(stage);
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
