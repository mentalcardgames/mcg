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
    Quantifier {
        kind: String,
        detail: String,
    },
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
            TraceEvent::Quantifier { kind, detail } => {
                write!(f, "Quantifier:{} {}", kind, detail)
            }
        }
    }
}

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
                            let event = match input {
                                Input::OptionalAccept => TraceEvent::OptionalAccept,
                                Input::OptionalDecline => TraceEvent::OptionalDecline,
                                Input::Choice { .. }
                                | Input::ChoosePlayer { .. }
                                | Input::ChooseCards { .. } => {
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
                        edges.get(1)
                    } else {
                        edges.first()
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

    // =======================================================================
    // Quantifier preprocessor integration (see `crate::quantifier`).
    // =======================================================================

    /// If a quantifier prompt is in flight and its answer has arrived, consume
    /// both and return the resume `StepResult`. Returns `None` (leaving the
    /// pending request and input untouched) when there is nothing to resume.
    fn take_quant_resume(&mut self) -> Option<StepResult> {
        enum Resume {
            Player {
                idx: usize,
                candidates: Vec<String>,
                original: Edge<LoweredPayLoad>,
            },
            Cards {
                selected: Vec<usize>,
                candidate_ids: Vec<usize>,
                original: Edge<LoweredPayLoad>,
            },
            AllCards {
                selected: Vec<usize>,
                player_names: Vec<String>,
                candidate_ids: Vec<usize>,
                original: Edge<LoweredPayLoad>,
            },
        }
        // Peek (immutable) to decide; clone the needed data so the immutable
        // borrows of `pending_quant` / `input_buffer` end before we mutate.
        let resume: Option<Resume> = {
            let pq = self.pending_quant.as_ref()?;
            if pq.state != self.current_state {
                return None;
            }
            let input = self.input_buffer.last()?;
            Some(match (&pq.kind, input) {
                (
                    crate::quantifier::PendingKind::DestPlayerAny {
                        candidates,
                        original,
                    },
                    Input::ChoosePlayer { idx },
                ) => Resume::Player {
                    idx: *idx,
                    candidates: candidates.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::CardsAnyOrRange {
                        candidate_ids,
                        original,
                    },
                    Input::ChooseCards { selected },
                ) => Resume::Cards {
                    selected: selected.clone(),
                    candidate_ids: candidate_ids.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::DestAllThenCards {
                        player_names,
                        candidate_ids,
                        original,
                    },
                    Input::ChooseCards { selected },
                ) => Resume::AllCards {
                    selected: selected.clone(),
                    player_names: player_names.clone(),
                    candidate_ids: candidate_ids.clone(),
                    original: original.clone(),
                },
                _ => return None,
            })
        };
        self.input_buffer.pop();
        self.pending_quant = None;
        resume.map(|resume| match resume {
            Resume::Player {
                idx,
                candidates,
                original,
            } => self.resume_dest_player_any(idx, candidates, original),
            Resume::Cards {
                selected,
                candidate_ids,
                original,
            } => self.resume_cards_any_or_range(selected, candidate_ids, original),
            Resume::AllCards {
                selected,
                player_names,
                candidate_ids,
                original,
            } => self.resume_dest_all_then_cards(selected, player_names, candidate_ids, original),
        })
    }

    /// `DestPlayerAll` arm: fan out to every resolved player. If the edge also
    /// carries a card-amount quantifier (`All`-of-`Any`), fire a single
    /// `ChooseCards` prompt first and defer the fan-out to the resume branch.
    fn step_dest_player_all(
        &mut self,
        edge: &Edge<LoweredPayLoad>,
        pc: front_end::ast::PlayerCollection,
    ) -> StepResult {
        let names = self.resolve_player_names(&pc);
        if let Some((qty, from)) = crate::quantifier::card_site(edge) {
            let candidate_ids = match crate::query::Evaluator::eval_cardset(&from, &self.game_data)
            {
                Ok((_, ids)) => ids,
                Err(e) => return StepResult::Error(e),
            };
            let (display, min, max) = self.build_choose_cards(&qty, &candidate_ids);
            let n = names.len();
            self.pending_quant = Some(crate::quantifier::PendingQuant {
                state: self.current_state,
                kind: crate::quantifier::PendingKind::DestAllThenCards {
                    player_names: names,
                    candidate_ids,
                    original: edge.clone(),
                },
            });
            self.emit_quant_trace(
                "DestPlayerAll",
                format!("{} players, awaiting card choice", n),
            );
            return StepResult::NeedsInput(InputType::ChooseCards {
                display,
                min,
                max,
                prompt: "Choose cards to deal to all players".to_string(),
            });
        }
        let n = names.len();
        match crate::quantifier::build_dest_all_chain(edge, names, &mut self.next_synth) {
            Ok(chain) => {
                if chain.is_empty() {
                    self.current_state = edge.to;
                    return StepResult::Ok;
                }
                let first = chain[0].0;
                for (sid, e) in chain {
                    self.pending_overlay.insert(sid, vec![e]);
                }
                self.current_state = first;
                self.emit_quant_trace("DestPlayerAll", format!("{} players", n));
                StepResult::Ok
            }
            Err(e) => StepResult::Error(e),
        }
    }

    /// `DestPlayerAny` arm (initial): resolve candidates and issue a
    /// `ChoosePlayer` prompt. The resume is handled by [`take_quant_resume`].
    fn step_dest_player_any(
        &mut self,
        edge: &Edge<LoweredPayLoad>,
        pc: front_end::ast::PlayerCollection,
    ) -> StepResult {
        let candidates = self.resolve_player_names(&pc);
        let n = candidates.len();
        self.pending_quant = Some(crate::quantifier::PendingQuant {
            state: self.current_state,
            kind: crate::quantifier::PendingKind::DestPlayerAny {
                candidates: candidates.clone(),
                original: edge.clone(),
            },
        });
        self.emit_quant_trace("DestPlayerAny", format!("{} candidates", n));
        StepResult::NeedsInput(InputType::ChoosePlayer {
            candidates,
            prompt: "Choose a player".to_string(),
        })
    }

    /// `SrcCardsAnyOrRange` arm (initial): evaluate `from`, build the card
    /// display, and issue a `ChooseCards` prompt.
    fn step_src_cards_any_or_range(
        &mut self,
        edge: &Edge<LoweredPayLoad>,
        qty: front_end::ast::Quantity,
        from: front_end::ast::CardSet,
    ) -> StepResult {
        let candidate_ids = match crate::query::Evaluator::eval_cardset(&from, &self.game_data) {
            Ok((_, ids)) => ids,
            Err(e) => return StepResult::Error(e),
        };
        let n = candidate_ids.len();
        let (display, min, max) = self.build_choose_cards(&qty, &candidate_ids);
        self.pending_quant = Some(crate::quantifier::PendingQuant {
            state: self.current_state,
            kind: crate::quantifier::PendingKind::CardsAnyOrRange {
                candidate_ids,
                original: edge.clone(),
            },
        });
        self.emit_quant_trace("SrcCardsAnyOrRange", format!("{} candidates", n));
        StepResult::NeedsInput(InputType::ChooseCards {
            display,
            min,
            max,
            prompt: "Choose cards".to_string(),
        })
    }

    /// Resume a `DestPlayerAny`: substitute the chosen player into the original
    /// edge and dispatch the single replacement edge.
    fn resume_dest_player_any(
        &mut self,
        idx: usize,
        candidates: Vec<String>,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        let Some(name) = candidates.get(idx) else {
            return StepResult::Error(format!(
                "ChoosePlayer idx {} out of range ({})",
                idx,
                candidates.len()
            ));
        };
        let name = name.clone();
        let mut repl = crate::quantifier::substitute_dest_player(&original, name.clone());
        let s = crate::quantifier::alloc_synth(&mut self.next_synth);
        repl.to = original.to;
        self.emit_quant_trace("DestPlayerAny", format!("chose {}", name));
        self.pending_overlay.insert(s, vec![repl]);
        self.current_state = s;
        StepResult::Ok
    }

    /// Resume a `CardsAnyOrRange`: validate the selection against any
    /// `IntRange` (re-prompting on failure), write the chosen ids into the
    /// synthetic memory, and dispatch the replacement edge.
    fn resume_cards_any_or_range(
        &mut self,
        selected: Vec<usize>,
        candidate_ids: Vec<usize>,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        if selected.iter().any(|&i| i >= candidate_ids.len()) {
            return StepResult::Error("ChooseCards index out of range".to_string());
        }
        let chosen: Vec<usize> = selected.iter().map(|&i| candidate_ids[i]).collect();
        if let Some((qty, _)) = crate::quantifier::card_site(&original) {
            if let front_end::ast::Quantity::IntRange { int_range } = &qty {
                if let Err(e) = crate::quantifier::validate_int_range(
                    int_range,
                    chosen.len(),
                    candidate_ids.len(),
                ) {
                    let (display, min, max) = self.build_choose_cards(&qty, &candidate_ids);
                    self.pending_quant = Some(crate::quantifier::PendingQuant {
                        state: self.current_state,
                        kind: crate::quantifier::PendingKind::CardsAnyOrRange {
                            candidate_ids,
                            original,
                        },
                    });
                    return StepResult::NeedsInput(InputType::ChooseCards {
                        display,
                        min,
                        max,
                        prompt: format!("{}. Please choose again.", e),
                    });
                }
            }
        }
        self.game_data.memories.insert(
            crate::quantifier::SYNTH_MEMORY_KEY.to_string(),
            crate::game_data::MemoryValue::CardSet(chosen.clone()),
        );
        let mut repl = crate::quantifier::substitute_cardset_memory(&original, &chosen);
        let s = crate::quantifier::alloc_synth(&mut self.next_synth);
        repl.to = original.to;
        self.emit_quant_trace(
            "SrcCardsAnyOrRange",
            format!("chose {} cards", chosen.len()),
        );
        self.pending_overlay.insert(s, vec![repl]);
        self.current_state = s;
        StepResult::Ok
    }

    /// Resume an `All`-of-`Any`: write the chosen ids into the synthetic
    /// memory once, then build the per-player fan-out chain whose every edge
    /// reads that same memory.
    fn resume_dest_all_then_cards(
        &mut self,
        selected: Vec<usize>,
        player_names: Vec<String>,
        candidate_ids: Vec<usize>,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        if selected.iter().any(|&i| i >= candidate_ids.len()) {
            return StepResult::Error("ChooseCards index out of range".to_string());
        }
        let chosen: Vec<usize> = selected.iter().map(|&i| candidate_ids[i]).collect();
        if let Some((qty, _)) = crate::quantifier::card_site(&original) {
            if let front_end::ast::Quantity::IntRange { int_range } = &qty {
                if let Err(e) = crate::quantifier::validate_int_range(
                    int_range,
                    chosen.len(),
                    candidate_ids.len(),
                ) {
                    let (display, min, max) = self.build_choose_cards(&qty, &candidate_ids);
                    self.pending_quant = Some(crate::quantifier::PendingQuant {
                        state: self.current_state,
                        kind: crate::quantifier::PendingKind::DestAllThenCards {
                            player_names,
                            candidate_ids,
                            original,
                        },
                    });
                    return StepResult::NeedsInput(InputType::ChooseCards {
                        display,
                        min,
                        max,
                        prompt: format!("{}. Please choose again.", e),
                    });
                }
            }
        }
        self.game_data.memories.insert(
            crate::quantifier::SYNTH_MEMORY_KEY.to_string(),
            crate::game_data::MemoryValue::CardSet(chosen.clone()),
        );
        let n = player_names.len();
        let ncards = chosen.len();
        match crate::quantifier::build_dest_all_chain_with_memory(
            &original,
            player_names,
            &chosen,
            &mut self.next_synth,
        ) {
            Ok(chain) => {
                if chain.is_empty() {
                    self.current_state = original.to;
                    return StepResult::Ok;
                }
                let first = chain[0].0;
                for (sid, e) in chain {
                    self.pending_overlay.insert(sid, vec![e]);
                }
                self.current_state = first;
                self.emit_quant_trace(
                    "DestPlayerAll",
                    format!("{} players, shared {} cards", n, ncards),
                );
                StepResult::Ok
            }
            Err(e) => StepResult::Error(e),
        }
    }

    /// Resolve a dest-quantifier `PlayerCollection` to player names, dropping
    /// any index that no longer maps to a live player (defensive).
    fn resolve_player_names(&self, pc: &front_end::ast::PlayerCollection) -> Vec<String> {
        let idxs = crate::quantifier::resolve_player_candidates(pc, &self.game_data);
        idxs.iter()
            .filter_map(|&i| self.game_data.players.get(i).map(|p| p.name.clone()))
            .collect()
    }

    /// Build the `display`/`min`/`max` for a `ChooseCards` prompt from the
    /// candidate card ids and the quantity.
    fn build_choose_cards(
        &self,
        qty: &front_end::ast::Quantity,
        candidate_ids: &[usize],
    ) -> (Vec<crate::game_data::Card>, usize, usize) {
        let display: Vec<crate::game_data::Card> = candidate_ids
            .iter()
            .map(|&id| self.game_data.get_card(id).cloned().unwrap_or_default())
            .collect();
        // NOTE: display.len() == candidate_ids.len() (missing cards become empty
        // attribute maps) so a selected index maps directly to candidate_ids[i].
        let (min, max) = crate::quantifier::derive_min_max(qty, candidate_ids.len());
        (display, min, max)
    }

    /// Emit a `Quantifier` trace entry at the current state (no transition).
    fn emit_quant_trace(&self, kind: &str, detail: String) {
        if let Some(ref sender) = self.trace_sender {
            let here = self.current_state.raw();
            (sender)(TraceEntry::Step {
                from: here,
                to: here,
                event: TraceEvent::Quantifier {
                    kind: kind.to_string(),
                    detail,
                },
            });
        }
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
                let label = self
                    .states
                    .get(&edge.to)
                    .and_then(|target_edges| target_edges.first())
                    .map(|first_edge| payload_label(&first_edge.payload));
                label.unwrap_or_else(|| format!("Option {}", i + 1))
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

fn payload_label(payload: &LoweredPayLoad) -> String {
    match payload {
        Payload::Action(gr) => format!("{}", gr),
        Payload::Condition { expr, negated } => {
            if *negated {
                format!("unless {}", expr)
            } else {
                format!("if {}", expr)
            }
        }
        Payload::EndCondition {
            expr,
            negated,
            stage,
        } => {
            if *negated {
                format!("end stage {} unless {}", stage, expr)
            } else {
                format!("end stage {} when {}", stage, expr)
            }
        }
        Payload::StageRoundCounter(stage) => format!("round of {}", stage),
        Payload::EndStage(stage) => format!("end stage {}", stage),
        Payload::Choice => "choose".to_string(),
        Payload::Optional => "optional".to_string(),
        Payload::Trigger => "trigger".to_string(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Choice {
        idx: usize,
    },
    OptionalAccept,
    OptionalDecline,
    /// Chosen player index (0-based into `InputType::ChoosePlayer::candidates`).
    ChoosePlayer {
        idx: usize,
    },
    /// Chosen card indices (0-based into `InputType::ChooseCards::display`).
    ChooseCards {
        selected: Vec<usize>,
    },
}

impl Input {
    pub fn idx(&self) -> usize {
        match self {
            Input::Choice { idx } => *idx,
            Input::OptionalAccept => 0,
            Input::OptionalDecline => 1,
            // The new variants are not consumed by the Choice/Optional arms;
            // `idx()` is meaningless for them — callers use the dedicated
            // accessors below. Returning 0 keeps the match exhaustive.
            Input::ChoosePlayer { .. } => 0,
            Input::ChooseCards { .. } => 0,
        }
    }

    /// If this is a `ChoosePlayer` input, the chosen 0-based candidate index.
    pub fn player_idx(&self) -> Option<usize> {
        match self {
            Input::ChoosePlayer { idx } => Some(*idx),
            _ => None,
        }
    }

    /// If this is a `ChooseCards` input, the chosen 0-based `display` indices.
    pub fn card_selection(&self) -> Option<&[usize]> {
        match self {
            Input::ChooseCards { selected } => Some(selected),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum StepResult {
    Ok,
    NeedsInput(InputType),
    GameOver,
    Error(String),
}

#[derive(Clone, Debug)]
pub enum InputType {
    Choice {
        options: Vec<String>,
        max_index: usize,
    },
    Optional(String),
    /// Prompt the player to pick one player by index from `candidates`.
    ChoosePlayer {
        candidates: Vec<String>,
        prompt: String,
    },
    /// Prompt the player to pick a subset of `display` cards. `min`/`max`
    /// bound the number of selections (for `Quantifier::Any` this is
    /// `1..display.len()`; for an `IntRange` it reflects the range).
    ChooseCards {
        display: Vec<crate::game_data::Card>,
        min: usize,
        max: usize,
        prompt: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_data::GameData;
    use front_end::ast::*;
    use front_end::ir::{Edge, Ir, LoweredPayLoad, Payload, StateID};
    use std::collections::HashMap;

    fn state_id(n: u32) -> StateID {
        unsafe { std::mem::transmute(n) }
    }

    fn make_move_action(
        from_card_set: CardSet,
        status: Status,
        to_card_set: CardSet,
    ) -> LoweredPayLoad {
        Payload::Action(GameRule::Action {
            action: ActionRule::Move {
                move_type: MoveType::Classic {
                    classic: ClassicMove::MoveCardSet {
                        move_cs: MoveCardSet::Move {
                            from: from_card_set,
                            status,
                            to: to_card_set,
                        },
                    },
                },
            },
        })
    }

    fn make_card_set_top(location: &str) -> CardSet {
        CardSet::Group {
            group: Group::CardPosition {
                card_position: CardPosition::Query {
                    query: QueryCardPosition::Top {
                        location: location.to_string(),
                    },
                },
            },
        }
    }

    fn make_card_set_location(name: &str) -> CardSet {
        CardSet::Group {
            group: Group::Groupable {
                groupable: Groupable::Location {
                    name: name.to_string(),
                },
            },
        }
    }

    #[test]
    fn payload_label_action_renders_display() {
        let from = make_card_set_top("Hand");
        let to = make_card_set_location("Table");
        let payload = make_move_action(from, Status::FaceDown, to);
        let label = payload_label(&payload);
        assert!(
            label.contains("move"),
            "label should contain 'move': {}",
            label
        );
        assert!(
            label.contains("face down"),
            "label should contain 'face down': {}",
            label
        );
    }

    #[test]
    fn payload_label_condition_renders_if() {
        let payload = Payload::Condition {
            expr: BoolExpr::Aggregate {
                aggregate: AggregateBool::CardSetEmpty {
                    card_set: make_card_set_location("Hand"),
                },
            },
            negated: false,
        };
        let label = payload_label(&payload);
        assert!(
            label.contains("if "),
            "label should contain 'if ': {}",
            label
        );
    }

    #[test]
    fn payload_label_condition_renders_unless() {
        let payload = Payload::Condition {
            expr: BoolExpr::Aggregate {
                aggregate: AggregateBool::CardSetEmpty {
                    card_set: make_card_set_location("Hand"),
                },
            },
            negated: true,
        };
        let label = payload_label(&payload);
        assert!(
            label.contains("unless"),
            "label should contain 'unless': {}",
            label
        );
    }

    #[test]
    fn payload_label_choice_is_choose() {
        let payload = Payload::Choice;
        assert_eq!(payload_label(&payload), "choose");
    }

    #[test]
    fn payload_label_optional_is_optional() {
        let payload = Payload::Optional;
        assert_eq!(payload_label(&payload), "optional");
    }

    #[test]
    fn payload_label_trigger_is_trigger() {
        let payload = Payload::Trigger;
        assert_eq!(payload_label(&payload), "trigger");
    }

    #[test]
    fn edge_labels_uses_payload_label() {
        let mut ir = Ir::<LoweredPayLoad>::default();
        let s0 = ir.entry;
        let s1 = state_id(1);
        let s2 = state_id(2);
        let s3 = state_id(3);

        let move_down = make_move_action(
            make_card_set_top("Hand"),
            Status::FaceDown,
            make_card_set_location("Table"),
        );
        let move_up = make_move_action(
            make_card_set_top("Hand"),
            Status::FaceUp,
            make_card_set_location("Table"),
        );

        ir.states.insert(
            s0,
            vec![
                Edge {
                    to: s1,
                    payload: Payload::Choice,
                    meta: None,
                },
                Edge {
                    to: s2,
                    payload: Payload::Choice,
                    meta: None,
                },
            ],
        );
        ir.states.insert(
            s1,
            vec![Edge {
                to: s3,
                payload: move_down,
                meta: None,
            }],
        );
        ir.states.insert(
            s2,
            vec![Edge {
                to: s3,
                payload: move_up,
                meta: None,
            }],
        );
        ir.states.insert(s3, vec![]);

        let labels = ir.edge_labels(s0);
        assert_eq!(labels.len(), 2);
        assert!(
            labels[0].contains("move"),
            "label[0] should contain 'move': {}",
            labels[0]
        );
        assert!(
            labels[0].contains("face down"),
            "label[0] should contain 'face down': {}",
            labels[0]
        );
        assert!(
            labels[1].contains("move"),
            "label[1] should contain 'move': {}",
            labels[1]
        );
        assert!(
            labels[1].contains("face up"),
            "label[1] should contain 'face up': {}",
            labels[1]
        );
    }

    #[test]
    fn edge_labels_falls_back_when_target_empty() {
        let mut ir = Ir::<LoweredPayLoad>::default();
        let s0 = ir.entry;
        let s1 = state_id(1);

        ir.states.insert(
            s0,
            vec![Edge {
                to: s1,
                payload: Payload::Choice,
                meta: None,
            }],
        );
        ir.states.insert(s1, vec![]);

        let labels = ir.edge_labels(s0);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0], "Option 1");
    }

    #[test]
    fn step_choice_emits_rich_options_in_needs_input() {
        let mut ir = Ir::<LoweredPayLoad>::default();
        let s0 = ir.entry;
        let s1 = state_id(1);
        let s2 = state_id(2);
        let s3 = state_id(3);

        let move_down = make_move_action(
            make_card_set_top("Hand"),
            Status::FaceDown,
            make_card_set_location("Table"),
        );
        let move_up = make_move_action(
            make_card_set_top("Hand"),
            Status::FaceUp,
            make_card_set_location("Table"),
        );

        ir.states.insert(
            s0,
            vec![
                Edge {
                    to: s1,
                    payload: Payload::Choice,
                    meta: None,
                },
                Edge {
                    to: s2,
                    payload: Payload::Choice,
                    meta: None,
                },
            ],
        );
        ir.states.insert(
            s1,
            vec![Edge {
                to: s3,
                payload: move_down,
                meta: None,
            }],
        );
        ir.states.insert(
            s2,
            vec![Edge {
                to: s3,
                payload: move_up,
                meta: None,
            }],
        );
        ir.states.insert(s3, vec![]);

        let mut interpreter = Interpreter {
            ir,
            game_data: GameData::new(),
            input_buffer: Vec::new(),
            current_state: s0,
            trace_sender: None,
            pending_overlay: HashMap::new(),
            next_synth: u32::MAX - 1,
            pending_quant: None,
        };

        let result = interpreter.step();
        match result {
            StepResult::NeedsInput(InputType::Choice { options, max_index }) => {
                assert_eq!(max_index, 1);
                assert_eq!(options.len(), 2);
                assert!(
                    options[0].contains("move"),
                    "options[0] should contain 'move': {}",
                    options[0]
                );
                assert!(
                    options[0].contains("face down"),
                    "options[0] should contain 'face down': {}",
                    options[0]
                );
                assert!(
                    options[1].contains("move"),
                    "options[1] should contain 'move': {}",
                    options[1]
                );
                assert!(
                    options[1].contains("face up"),
                    "options[1] should contain 'face up': {}",
                    options[1]
                );
            }
            _ => panic!("expected NeedsInput(Choice), got {:?}", result),
        }
    }

    #[test]
    fn step_optional_prompt_contains_accept_action() {
        let mut ir = Ir::<LoweredPayLoad>::default();
        let s0 = ir.entry;
        let s1 = state_id(1);
        let s9 = state_id(9);
        let s3 = state_id(3);

        let deal_action = make_move_action(
            make_card_set_top("Stock"),
            Status::Private,
            make_card_set_location("Hand"),
        );

        ir.states.insert(
            s0,
            vec![
                Edge {
                    to: s1,
                    payload: Payload::Optional,
                    meta: None,
                },
                Edge {
                    to: s9,
                    payload: Payload::Optional,
                    meta: None,
                },
            ],
        );
        ir.states.insert(
            s1,
            vec![Edge {
                to: s3,
                payload: deal_action,
                meta: None,
            }],
        );
        ir.states.insert(s9, vec![]);
        ir.states.insert(s3, vec![]);

        let mut interpreter = Interpreter {
            ir,
            game_data: GameData::new(),
            input_buffer: Vec::new(),
            current_state: s0,
            trace_sender: None,
            pending_overlay: HashMap::new(),
            next_synth: u32::MAX - 1,
            pending_quant: None,
        };

        let result = interpreter.step();
        match result {
            StepResult::NeedsInput(InputType::Optional(prompt)) => {
                assert!(
                    prompt.contains("Do you want to:"),
                    "prompt should contain 'Do you want to:': {}",
                    prompt
                );
                assert!(
                    prompt.contains("move"),
                    "prompt should contain 'move': {}",
                    prompt
                );
            }
            _ => panic!("expected NeedsInput(Optional), got {:?}", result),
        }
    }

    #[test]
    fn step_optional_prompt_fallback_when_no_accept_edge() {
        let mut ir = Ir::<LoweredPayLoad>::default();
        let s0 = ir.entry;
        let s1 = state_id(1);
        let s9 = state_id(9);

        ir.states.insert(
            s0,
            vec![
                Edge {
                    to: s1,
                    payload: Payload::Optional,
                    meta: None,
                },
                Edge {
                    to: s9,
                    payload: Payload::Optional,
                    meta: None,
                },
            ],
        );
        ir.states.insert(s1, vec![]);
        ir.states.insert(s9, vec![]);

        let mut interpreter = Interpreter {
            ir,
            game_data: GameData::new(),
            input_buffer: Vec::new(),
            current_state: s0,
            trace_sender: None,
            pending_overlay: HashMap::new(),
            next_synth: u32::MAX - 1,
            pending_quant: None,
        };

        let result = interpreter.step();
        match result {
            StepResult::NeedsInput(InputType::Optional(prompt)) => {
                assert_eq!(prompt, "Do you want to take this optional action? (y/n)");
            }
            _ => panic!("expected NeedsInput(Optional), got {:?}", result),
        }
    }
}
