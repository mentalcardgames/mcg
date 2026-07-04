use front_end::ir::{Edge, LoweredPayLoad};

use super::Interpreter;
use crate::interpreter::trace::{TraceEntry, TraceEvent};
use crate::interpreter::types::{Input, InputType, StepResult};

impl Interpreter {
    // =======================================================================
    // Quantifier preprocessor integration (see `crate::quantifier`).
    // =======================================================================

    /// If a quantifier prompt is in flight and its answer has arrived, consume
    /// both and return the resume `StepResult`. Returns `None` (leaving the
    /// pending request and input untouched) when there is nothing to resume.
    pub(super) fn take_quant_resume(&mut self) -> Option<StepResult> {
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
    pub(super) fn step_dest_player_all(
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
    pub(super) fn step_dest_player_any(
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
    pub(super) fn step_src_cards_any_or_range(
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
        let qty_match = crate::quantifier::card_site(&original).map(|(q, _)| q.clone());
        let chosen = match self.validate_choose_cards(&selected, &candidate_ids, qty_match.as_ref())
        {
            CardValidation::Ok(chosen) => chosen,
            CardValidation::RePrompt(it) => {
                self.pending_quant = Some(crate::quantifier::PendingQuant {
                    state: self.current_state,
                    kind: crate::quantifier::PendingKind::CardsAnyOrRange {
                        candidate_ids,
                        original,
                    },
                });
                return StepResult::NeedsInput(it);
            }
            CardValidation::Fatal(sr) => return sr,
        };
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
        let qty_match = crate::quantifier::card_site(&original).map(|(q, _)| q.clone());
        let chosen = match self.validate_choose_cards(&selected, &candidate_ids, qty_match.as_ref())
        {
            CardValidation::Ok(chosen) => chosen,
            CardValidation::RePrompt(it) => {
                self.pending_quant = Some(crate::quantifier::PendingQuant {
                    state: self.current_state,
                    kind: crate::quantifier::PendingKind::DestAllThenCards {
                        player_names,
                        candidate_ids,
                        original,
                    },
                });
                return StepResult::NeedsInput(it);
            }
            CardValidation::Fatal(sr) => return sr,
        };
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

    /// Validates a `ChooseCards` answer. The caller is responsible for
    /// restoring `pending_quant` on the `RePrompt` branch (the two callers
    /// store *different* `PendingKind` variants, so the restoration is not
    /// unified here). See Stage 6 / sub-task B1.
    fn validate_choose_cards(
        &self,
        selected: &[usize],
        candidate_ids: &[usize],
        qty: Option<&front_end::ast::Quantity>,
    ) -> CardValidation {
        if selected.iter().any(|&i| i >= candidate_ids.len()) {
            return CardValidation::Fatal(StepResult::Error(
                "ChooseCards index out of range".to_string(),
            ));
        }
        let chosen: Vec<usize> = selected.iter().map(|&i| candidate_ids[i]).collect();
        if let Some(qty) = qty {
            if let front_end::ast::Quantity::IntRange { int_range } = qty {
                if let Err(e) = crate::quantifier::validate_int_range(
                    int_range,
                    chosen.len(),
                    candidate_ids.len(),
                ) {
                    let (display, min, max) = self.build_choose_cards(qty, candidate_ids);
                    return CardValidation::RePrompt(InputType::ChooseCards {
                        display,
                        min,
                        max,
                        prompt: format!("{}. Please choose again.", e),
                    });
                }
            }
        }
        CardValidation::Ok(chosen)
    }
}

/// Result of validating a `ChooseCards` answer; see [`Interpreter::validate_choose_cards`].
enum CardValidation {
    /// Selection is valid; the caller proceeds with the chosen ids.
    Ok(Vec<usize>),
    /// Selection is invalid; the caller restores `pending_quant` and returns
    /// the re-prompt as a `StepResult::NeedsInput`.
    RePrompt(InputType),
    /// A fatal error (e.g. index out of range); the caller returns this verbatim.
    Fatal(StepResult),
}
