/*
In interpreter.rs, we define the logic for executing a single step of the game.
The main function is step(), which takes the current game state and an input buffer, and returns a StepResult enum.
The StepResult enum has the following variants:
 - Ok: the step was executed successfully, and the game state was modified accordingly.
 - NeedsInput: the step requires input from the player, and the game state is waiting for the next input.
 - GameOver: the step resulted in the end of the game, and the game state is now in a terminal state.
 - Error: there was an error executing the step, and the game state may be in an inconsistent state.
*/

use front_end::ast::GameRule;
use front_end::ir::{Edge, Ir, LoweredPayLoad, Payload, StateID};

use crate::error::EngineError;
use crate::game_data::GameData;
mod ir_ext;
mod quant_driver;
mod trace;
mod types;

#[cfg(feature = "tracing")]
mod trace_tracing;
#[cfg(feature = "tracing")]
pub use trace_tracing::tracing_trace_sender;

pub use ir_ext::IrExt;
pub use trace::{TraceEntry, TraceEvent};
pub use types::{Input, InputKind, InputType, StepResult};

use crate::interpreter::ir_ext::payload_label;

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
                .contains_key(&format!("Table_{}", crate::quantifier::SYNTH_MEMORY_KEY))
        {
            self.game_data
                .memories
                .remove(&format!("Table_{}", crate::quantifier::SYNTH_MEMORY_KEY));
        }

        // (S) Ineligible-player instruction skip (2026-08-10).
        // A player who is out of the game, or out of the current stage, is
        // never offered prompts and none of their instructions run: every
        // skippable edge (moves, scores, conditions, choices, optionals,
        // triggers, quantifier sites) is advanced through without executing.
        // Only cycle/end actions and the stage bookkeeping (end-condition,
        // round counter, stage exit) still execute, so the stage loops back
        // and the next eligible player (moved into place by the cycle) takes
        // over. A stage with no players left auto-exits at its end-condition,
        // and a game with no players left runs out to the goal with an empty
        // winner set — see the `EndCondition` arm below.
        let skip = self
            .pending_overlay
            .get(&self.current_state)
            .or_else(|| self.ir.states.get(&self.current_state))
            .and_then(|edges| edges.first())
            .map(|edge| payload_is_skippable(&edge.payload) && self.current_player_ineligible())
            .unwrap_or(false);
        if skip {
            let edge = self
                .pending_overlay
                .get(&self.current_state)
                .or_else(|| self.ir.states.get(&self.current_state))
                .and_then(|edges| edges.first())
                .cloned()
                .expect("skip implies an outgoing edge exists");
            if let Some(ref sender) = self.trace_sender {
                (sender)(TraceEntry::Step {
                    from: self.current_state.raw(),
                    to: edge.to.raw(),
                    event: TraceEvent::Skipped {
                        player: self
                            .game_data
                            .get_current_player()
                            .map(|p| p.name.clone())
                            .unwrap_or_default(),
                        stage: self.game_data.get_current_stage().unwrap_or_default(),
                    },
                });
            }
            self.current_state = edge.to;
            return StepResult::Ok;
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
                if let Some(ref sender) = self.trace_sender {
                    (sender)(TraceEntry::Step {
                        from,
                        to,
                        event: TraceEvent::Action { rule: gr.clone() },
                    });
                }
            }
            return self.dispatch(edge);
        }

        // (C0) Resume an in-flight quantifier prompt if its input has arrived.
        if let Some(resumed) = self.take_quant_resume() {
            return resumed;
        }

        let edges: &Vec<Edge<LoweredPayLoad>> = match self.ir.states.get(&self.current_state) {
            Some(e) => e,
            None => {
                return StepResult::Error(EngineError::CurrentStateNotFoundInIr {
                    state: self.current_state.raw(),
                })
            }
        };

        if edges.is_empty() {
            if self.current_state == self.ir.goal {
                // Emit the winner set once, on the transition into GameOver
                // (2026-08-10): winners = every player still in game.
                if let Some(ref sender) = self.trace_sender {
                    (sender)(TraceEntry::Step {
                        from: self.current_state.raw(),
                        to: self.current_state.raw(),
                        event: TraceEvent::GameOver {
                            winners: self.game_data.winner_names(),
                        },
                    });
                }
                return StepResult::GameOver;
            }
            return StepResult::Error(EngineError::NoOutgoingEdges {
                state: self.current_state.raw(),
            });
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
                    crate::quantifier::QuantSite::SourcePlayerAny { pc } => {
                        self.step_source_player_any(&edge_owned, pc)
                    }
                    crate::quantifier::QuantSite::ComboSource { combo, from } => {
                        self.step_combo_source(&edge_owned, combo, from)
                    }
                    crate::quantifier::QuantSite::SrcCardsAnyOrRange { qty, from } => {
                        self.step_src_cards_any_or_range(&edge_owned, qty, from)
                    }
                    crate::quantifier::QuantSite::SrcCardsExactN { qty, from } => {
                        self.step_src_cards_exact_n(&edge_owned, qty, from)
                    }
                    crate::quantifier::QuantSite::DealCount { qty, from } => {
                        self.step_deal_count(&edge_owned, qty, from)
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
                            // Setup-`Any` (I-20, relaxed 2026-08-10): prompt
                            // for the player and substitute it into every
                            // any-site of the setup, instead of erroring.
                            return self.step_setup_any(&edge.clone());
                        }
                    }
                    // `bid any on <memory> of <owner>` / `bid >= M and <= N
                    // on ...` — the numeric-input prompt (2026-08-10): the
                    // interpreter intercepts the quantity and asks for a
                    // number, then substitutes a literal before dispatch.
                    if let front_end::ast::GameRule::Action {
                        action:
                            front_end::ast::ActionRule::BidMemoryAction {
                                memory,
                                quantity,
                                owner,
                            },
                    } = gr
                    {
                        if matches!(
                            quantity,
                            front_end::ast::Quantity::Quantifier {
                                quantifier: front_end::ast::Quantifier::Any
                            } | front_end::ast::Quantity::IntRange { .. }
                        ) {
                            // Clone out of the `self.ir` borrow before
                            // calling a `&mut self` method.
                            let edge_owned = edge.clone();
                            let memory_owned = memory.clone();
                            let quantity_owned = quantity.clone();
                            let owner_owned = owner.clone();
                            return self.step_bid_number(
                                &edge_owned,
                                &memory_owned,
                                &quantity_owned,
                                &owner_owned,
                            );
                        }
                    }
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::Action { rule: gr.clone() },
                        });
                    }
                    self.dispatch(edge.clone())
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
                            return self.dispatch(choice_edge.clone());
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
                                | InputKind::ChooseCards { .. }
                                | InputKind::Number { .. } => {
                                    return StepResult::Error(
                                        EngineError::UnexpectedInputForOptional,
                                    )
                                }
                            };
                            (sender)(TraceEntry::Step { from, to, event });
                        }
                        if let Some(opt_edge) = edges.get(input.idx()) {
                            return self.dispatch(opt_edge.clone());
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
                        return StepResult::Error(EngineError::ConditionEdgeCount {
                            state: self.current_state.raw(),
                            found: edges.len(),
                        });
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
                                expr: expr.clone(),
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
                        self.dispatch(e.clone())
                    } else {
                        StepResult::Error(EngineError::ConditionEdgeMissing)
                    }
                }
                Payload::EndCondition {
                    expr,
                    negated,
                    stage,
                } => {
                    if edges.len() != 2 {
                        return StepResult::Error(EngineError::EndConditionEdgeCount {
                            state: self.current_state.raw(),
                            found: edges.len(),
                        });
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
                    let mut should_exit = result == *negated;
                    // Auto-end (2026-08-10): if no players remain in the game
                    // or no players remain in this stage, the stage exits
                    // immediately (an empty winner set ends the game once the
                    // flow reaches the goal). Only applies when the game has
                    // players at all (zero-player fixtures keep pure dispatch
                    // semantics).
                    if !self.game_data.players.is_empty() {
                        let no_players_in_game = self.game_data.players.iter().all(|p| !p.in_game);
                        let no_players_in_stage = self
                            .game_data
                            .players
                            .iter()
                            .all(|p| !*p.in_stage.get(stage).unwrap_or(&false));
                        if no_players_in_game || no_players_in_stage {
                            should_exit = true;
                        }
                    }
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::EndCondition {
                                expr: expr.clone(),
                                stage: stage.clone(),
                                result,
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
                        self.dispatch(e.clone())
                    } else {
                        StepResult::Error(EngineError::EndConditionEdgeMissing)
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
                    self.dispatch(edge.clone())
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
                    self.dispatch(edge.clone())
                }
                Payload::Trigger => {
                    if let Some(ref sender) = self.trace_sender {
                        (sender)(TraceEntry::Step {
                            from,
                            to,
                            event: TraceEvent::Trigger,
                        });
                    }
                    self.dispatch(edge.clone())
                }
            }
        } else {
            StepResult::Error(EngineError::NoEdgesFound {
                state: self.current_state.raw(),
            })
        }
    }

    /// Advances `current_state` to `edge.to` and executes the edge's payload.
    /// Fallible since 2026-08: action-evaluation failures surface as
    /// `Err(EngineError)` instead of panicking.
    pub fn execute_edge(&mut self, edge: Edge<LoweredPayLoad>) -> Result<(), EngineError> {
        self.current_state = edge.to;
        crate::action::execute(edge.payload, &mut self.game_data)
    }

    /// `execute_edge` followed by `StepResult::Ok`/`StepResult::Error` mapping —
    /// the standard shape of the per-`Payload` dispatch arms.
    fn dispatch(&mut self, edge: Edge<LoweredPayLoad>) -> StepResult {
        match self.execute_edge(edge) {
            Ok(()) => StepResult::Ok,
            Err(e) => StepResult::Error(e),
        }
    }

    /// Pushes input to the input buffer.
    pub fn provide_input(&mut self, input: Input) {
        self.input_buffer.push(input);
    }

    /// `true` while the current player must not act: they are out of the
    /// game, or out of the current stage (or there is no current player).
    /// Only meaningful inside a stage — without a current stage nothing is
    /// skipped (setup and top-level rules always run).
    fn current_player_ineligible(&self) -> bool {
        let Some(stage) = self.game_data.get_current_stage() else {
            return false;
        };
        let Some(current) = self.game_data.get_current_player() else {
            return true;
        };
        !current.in_game || !*current.in_stage.get(&stage).unwrap_or(&false)
    }

    /// Prompts the current player for a number (`bid any` / `bid >= M and
    /// <= N on <memory> of <owner>`, 2026-08-10). The answer arrives as
    /// `InputKind::Number { value }`; the edge is then re-dispatched with a
    /// literal quantity. Out-of-range answers are discarded and re-prompted
    /// (the controller validates `Player`-sourced answers the same way).
    fn step_bid_number(
        &mut self,
        original: &Edge<LoweredPayLoad>,
        memory: &str,
        quantity: &front_end::ast::Quantity,
        owner: &front_end::ast::Owner,
    ) -> StepResult {
        let (min, max) = bid_bounds(quantity, &self.game_data);
        if let Some(input) = self.input_buffer.pop() {
            let value = match input.kind {
                InputKind::Number { value } => value,
                // Stale non-number input: discard and re-prompt (I-21-style).
                _ => return StepResult::NeedsInput(bid_prompt(memory, owner, min, max)),
            };
            if min.is_some_and(|m| value < m) || max.is_some_and(|m| value > m) {
                return StepResult::NeedsInput(bid_prompt(memory, owner, min, max));
            }
            let mut edge = original.clone();
            if let Payload::Action(front_end::ast::GameRule::Action { action }) = &mut edge.payload
            {
                *action = front_end::ast::ActionRule::BidMemoryAction {
                    memory: memory.to_string(),
                    quantity: front_end::ast::Quantity::Int {
                        int: front_end::ast::IntExpr::Literal { int: value },
                    },
                    owner: owner.clone(),
                };
            }
            if let Some(ref sender) = self.trace_sender {
                (sender)(TraceEntry::Step {
                    from: self.current_state.raw(),
                    to: edge.to.raw(),
                    event: TraceEvent::Action {
                        rule: GameRule::Action {
                            action: front_end::ast::ActionRule::BidMemoryAction {
                                memory: memory.to_string(),
                                quantity: front_end::ast::Quantity::Int {
                                    int: front_end::ast::IntExpr::Literal { int: value },
                                },
                                owner: owner.clone(),
                            },
                        },
                    },
                });
            }
            self.dispatch(edge)
        } else {
            StepResult::NeedsInput(bid_prompt(memory, owner, min, max))
        }
    }
}

fn bid_prompt(
    memory: &str,
    owner: &front_end::ast::Owner,
    min: Option<i32>,
    max: Option<i32>,
) -> InputType {
    let bounds = match (min, max) {
        (Some(m), Some(x)) => format!(" between {m} and {x}"),
        (Some(m), None) => format!(" of at least {m}"),
        (None, Some(x)) => format!(" of at most {x}"),
        (None, None) => String::new(),
    };
    InputType::Number {
        min,
        max,
        prompt: format!("Enter a number{bounds} to bid into {memory} of {owner:?}"),
    }
}

/// Advisory min/max bounds for a `bid` quantity (`any` → unbounded;
/// `>= M and <= N` → the range's bounds; `or`-joined ranges contribute
/// nothing). Bound expressions that cannot be evaluated are ignored — the
/// bounds are a convenience, not a security boundary.
fn bid_bounds(
    quantity: &front_end::ast::Quantity,
    game_data: &GameData,
) -> (Option<i32>, Option<i32>) {
    use front_end::ast::{IntCompare, IntRangeOperator};
    let mut min: Option<i32> = None;
    let mut max: Option<i32> = None;
    let mut consider = |cmp: &IntCompare, target: i32| match cmp {
        IntCompare::Ge | IntCompare::Gt => min = Some(target),
        IntCompare::Le | IntCompare::Lt => max = Some(target),
        IntCompare::Eq => {
            min = Some(target);
            max = Some(target);
        }
        IntCompare::Neq => {}
    };
    match quantity {
        front_end::ast::Quantity::Quantifier { .. } => (None, None),
        front_end::ast::Quantity::IntRange { int_range } => {
            let (start_cmp, start_expr) = &int_range.start;
            if let Ok(t) = crate::query::Evaluator::eval_int(start_expr, game_data) {
                consider(start_cmp, t);
            }
            for (op, cmp, expr) in &int_range.op_int {
                if matches!(op, IntRangeOperator::And) {
                    if let Ok(t) = crate::query::Evaluator::eval_int(expr, game_data) {
                        consider(cmp, t);
                    }
                }
            }
            (min, max)
        }
        front_end::ast::Quantity::Int { .. } => (None, None),
    }
}

/// Whether an edge's payload is subject to the ineligible-player skip
/// (everything except stage bookkeeping, setup rules, and cycle/end
/// actions).
fn payload_is_skippable(payload: &LoweredPayLoad) -> bool {
    match payload {
        Payload::EndCondition { .. } | Payload::StageRoundCounter(_) | Payload::EndStage(_) => {
            false
        }
        Payload::Action(gr) => match gr {
            front_end::ast::GameRule::SetUp { .. } => false,
            front_end::ast::GameRule::Action { action } => !matches!(
                action,
                front_end::ast::ActionRule::CycleAction { .. }
                    | front_end::ast::ActionRule::EndAction { .. }
            ),
            front_end::ast::GameRule::Scoring { .. } => true,
        },
        _ => true,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
