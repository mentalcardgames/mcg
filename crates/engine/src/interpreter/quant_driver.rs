use front_end::ir::{Edge, LoweredPayLoad};

use super::Interpreter;
use crate::error::EngineError;
use crate::interpreter::trace::{TraceEntry, TraceEvent};
use crate::interpreter::types::{InputKind, InputType, StepResult};
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
            SourcePlayer {
                idx: usize,
                candidates: Vec<String>,
                original: Edge<LoweredPayLoad>,
            },
            SetupAny {
                idx: usize,
                candidates: Vec<String>,
                original: Edge<LoweredPayLoad>,
            },
            Cards {
                selected: Vec<usize>,
                candidate_ids: Vec<usize>,
                original: Edge<LoweredPayLoad>,
            },
            ExactCards {
                selected: Vec<usize>,
                candidate_ids: Vec<usize>,
                expected: usize,
                original: Edge<LoweredPayLoad>,
            },
            DealCount {
                value: i32,
                min: Option<i32>,
                max: Option<i32>,
                prompt: String,
                original: Edge<LoweredPayLoad>,
            },
            Combo {
                selected: Vec<usize>,
                candidate_ids: Vec<usize>,
                filter: front_end::ast::FilterExpr,
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
            Some(match (&pq.kind, &input.kind) {
                (
                    crate::quantifier::PendingKind::DestPlayerAny {
                        candidates,
                        original,
                    },
                    InputKind::ChoosePlayer { idx },
                ) => Resume::Player {
                    idx: *idx,
                    candidates: candidates.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::SourcePlayerAny {
                        candidates,
                        original,
                    },
                    InputKind::ChoosePlayer { idx },
                ) => Resume::SourcePlayer {
                    idx: *idx,
                    candidates: candidates.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::SetupAny {
                        candidates,
                        original,
                    },
                    InputKind::ChoosePlayer { idx },
                ) => Resume::SetupAny {
                    idx: *idx,
                    candidates: candidates.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::CardsAnyOrRange {
                        candidate_ids,
                        original,
                    },
                    InputKind::ChooseCards { selected },
                ) => Resume::Cards {
                    selected: selected.clone(),
                    candidate_ids: candidate_ids.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::CardsExactN {
                        candidate_ids,
                        expected,
                        original,
                    },
                    InputKind::ChooseCards { selected },
                ) => Resume::ExactCards {
                    selected: selected.clone(),
                    candidate_ids: candidate_ids.clone(),
                    expected: *expected,
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::DealCount {
                        min,
                        max,
                        prompt,
                        original,
                    },
                    InputKind::Number { value },
                ) => Resume::DealCount {
                    value: *value,
                    min: *min,
                    max: *max,
                    prompt: prompt.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::Combo {
                        candidate_ids,
                        filter,
                        original,
                    },
                    InputKind::ChooseCards { selected },
                ) => Resume::Combo {
                    selected: selected.clone(),
                    candidate_ids: candidate_ids.clone(),
                    filter: filter.clone(),
                    original: original.clone(),
                },
                (
                    crate::quantifier::PendingKind::DestAllThenCards {
                        player_names,
                        candidate_ids,
                        original,
                    },
                    InputKind::ChooseCards { selected },
                ) => Resume::AllCards {
                    selected: selected.clone(),
                    player_names: player_names.clone(),
                    candidate_ids: candidate_ids.clone(),
                    original: original.clone(),
                },
                _ => {
                    self.input_buffer.pop();
                    return None;
                }
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
            Resume::SourcePlayer {
                idx,
                candidates,
                original,
            } => self.resume_source_player_any(idx, candidates, original),
            Resume::SetupAny {
                idx,
                candidates,
                original,
            } => self.resume_setup_any(idx, candidates, original),
            Resume::Cards {
                selected,
                candidate_ids,
                original,
            } => self.resume_cards_any_or_range(selected, candidate_ids, original),
            Resume::ExactCards {
                selected,
                candidate_ids,
                expected,
                original,
            } => self.resume_cards_exact_n(selected, candidate_ids, expected, original),
            Resume::DealCount {
                value,
                min,
                max,
                prompt,
                original,
            } => self.resume_deal_count(value, min, max, prompt, original),
            Resume::Combo {
                selected,
                candidate_ids,
                filter,
                original,
            } => self.resume_combo_source(selected, candidate_ids, filter, original),
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
        // Chained source-player site: resolve it *first* (one prompt), then
        // let the resume re-enter this arm with the substituted edge — the
        // per-player fan-out edges must carry a concrete source owner.
        if let Some(source_pc) = crate::quantifier::edge_source_any(edge) {
            return self.step_source_player_any(edge, source_pc);
        }
        let names = match self.resolve_player_names(&pc) {
            Ok(n) => n,
            Err(e) => return StepResult::Error(e),
        };
        if let Some((qty, from)) = crate::quantifier::src_card_choice_site(edge) {
            match &qty {
                // `move N from X to <all>`: pick exactly N cards first, then
                // the resume re-scans and fans out (2026-08-10).
                front_end::ast::Quantity::Int { .. } => {
                    return self.step_src_cards_exact_n(edge, qty, from);
                }
                _ => {
                    if crate::quantifier::is_deal_move(edge) {
                        // `deal any/range from X to <all>`: choose the COUNT
                        // first, then the resume re-scans and fans out.
                        return self.step_deal_count(edge, qty, from);
                    }
                    let candidate_ids =
                        match crate::query::Evaluator::eval_cardset(&from, &self.game_data) {
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
            }
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
        let candidates = match self.resolve_player_names(&pc) {
            Ok(n) => n,
            Err(e) => return StepResult::Error(e),
        };
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

    /// `SourcePlayerAny` arm (initial): like [`step_dest_player_any`], but the
    /// chosen player becomes the move's *source* owner (e.g.
    /// `deal Hand where Rank is "Ace" of any …` — "ask any player").
    pub(super) fn step_source_player_any(
        &mut self,
        edge: &Edge<LoweredPayLoad>,
        pc: front_end::ast::PlayerCollection,
    ) -> StepResult {
        let candidates = match self.resolve_player_names(&pc) {
            Ok(n) => n,
            Err(e) => return StepResult::Error(e),
        };
        let n = candidates.len();
        self.pending_quant = Some(crate::quantifier::PendingQuant {
            state: self.current_state,
            kind: crate::quantifier::PendingKind::SourcePlayerAny {
                candidates: candidates.clone(),
                original: edge.clone(),
            },
        });
        self.emit_quant_trace("SourcePlayerAny", format!("{} candidates", n));
        StepResult::NeedsInput(InputType::ChoosePlayer {
            candidates,
            prompt: "Choose a player".to_string(),
        })
    }

    /// `SetupAny` arm (initial): a setup rule contains `Quantifier::Any`
    /// (e.g. `location Hand on any`) — prompt for one player, then substitute
    /// it into every any-site of the setup before dispatch (I-20, relaxed).
    pub(super) fn step_setup_any(&mut self, edge: &Edge<LoweredPayLoad>) -> StepResult {
        let pc = front_end::ast::PlayerCollection::Aggregate {
            aggregate: front_end::ast::AggregatePlayerCollection::Quantifier {
                quantifier: front_end::ast::Quantifier::Any,
            },
        };
        let candidates = match self.resolve_player_names(&pc) {
            Ok(n) => n,
            Err(e) => return StepResult::Error(e),
        };
        let n = candidates.len();
        self.pending_quant = Some(crate::quantifier::PendingQuant {
            state: self.current_state,
            kind: crate::quantifier::PendingKind::SetupAny {
                candidates: candidates.clone(),
                original: edge.clone(),
            },
        });
        self.emit_quant_trace("SetupAny", format!("{} candidates", n));
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

    /// `ComboSource` arm (initial): prompt the player to choose cards from
    /// the whole source pile; the resume validates the choice against the
    /// combo's filter.
    pub(super) fn step_combo_source(
        &mut self,
        edge: &Edge<LoweredPayLoad>,
        combo: String,
        from: front_end::ast::CardSet,
    ) -> StepResult {
        // The pile the combo filters (groupable + owner, no combo).
        let pile = crate::quantifier::combo_pile_cardset(&from);
        let candidate_ids = match crate::query::Evaluator::eval_cardset(&pile, &self.game_data) {
            Ok((_, ids)) => ids,
            Err(e) => return StepResult::Error(e),
        };
        let n = candidate_ids.len();
        if n == 0 {
            // Nothing to lay down — dispatch the original edge (an empty
            // combo source is a no-op move).
            return self.dispatch(edge.clone());
        }
        let filter = match self
            .game_data
            .combos
            .iter()
            .find(|c| c.name == combo)
            .map(|c| c.filter.clone())
        {
            Some(f) => f,
            None => {
                return StepResult::Error(EngineError::ComboNotFound { name: combo });
            }
        };
        let (display, _, max) = self.build_choose_cards(
            &front_end::ast::Quantity::Quantifier {
                quantifier: front_end::ast::Quantifier::Any,
            },
            &candidate_ids,
        );
        // min = 0: an empty selection is a valid no-op ("cancel / no book"),
        // so a prompt that over-fires (D-16 read-side) can always be
        // dismissed instead of trapping the player in re-prompts.
        self.pending_quant = Some(crate::quantifier::PendingQuant {
            state: self.current_state,
            kind: crate::quantifier::PendingKind::Combo {
                candidate_ids,
                filter,
                original: edge.clone(),
            },
        });
        self.emit_quant_trace("ComboSource", format!("{} candidates", n));
        StepResult::NeedsInput(InputType::ChooseCards {
            display,
            min: 0,
            max,
            prompt: format!("Choose cards that form a '{}' (0 to skip)", combo),
        })
    }

    /// `DealCount` arm (initial): `deal any` / `deal >= M and <= N from X`
    /// prompt for **how many** cards to deal (2026-08-10 verb semantics —
    /// `deal` = automatic from the top, the player only chooses the count).
    /// A degenerate range (`>= 2 and <= 2`, or `any` over a 1-card pile)
    /// short-circuits to the automatic top-N without prompting.
    pub(super) fn step_deal_count(
        &mut self,
        edge: &Edge<LoweredPayLoad>,
        qty: front_end::ast::Quantity,
        from: front_end::ast::CardSet,
    ) -> StepResult {
        use front_end::ast::{Quantifier, Quantity};
        let (min, max) = match &qty {
            Quantity::Quantifier {
                quantifier: Quantifier::Any,
            } => {
                let len = match crate::query::Evaluator::eval_cardset(&from, &self.game_data) {
                    Ok((_, ids)) => ids.len(),
                    Err(e) => return StepResult::Error(e),
                };
                if len == 0 {
                    // Empty pile: nothing to deal — no-op.
                    return self.dispatch(edge.clone());
                }
                (Some(1), Some(len as i32))
            }
            Quantity::IntRange { .. } => super::bid_bounds(&qty, &self.game_data),
            _ => (None, None),
        };
        if let (Some(m), Some(x)) = (min, max) {
            if m == x {
                // Degenerate range: the count is fixed — deal it automatically.
                let repl = crate::quantifier::substitute_quantity(edge, m);
                self.emit_quant_trace("DealCount", format!("fixed count {}", m));
                return self.quantify_or_dispatch(repl);
            }
        }
        let prompt = format!("How many cards to deal from {}?", from);
        self.pending_quant = Some(crate::quantifier::PendingQuant {
            state: self.current_state,
            kind: crate::quantifier::PendingKind::DealCount {
                min,
                max,
                prompt: prompt.clone(),
                original: edge.clone(),
            },
        });
        self.emit_quant_trace(
            "DealCount",
            format!("awaiting count in {:?}..{:?}", min, max),
        );
        StepResult::NeedsInput(InputType::Number { min, max, prompt })
    }

    /// `SrcCardsExactN` arm (initial): `move N from <non-positional>` prompts
    /// the player to pick **exactly N** cards (2026-08-10 verb semantics —
    /// `move`/`exchange` = the player chooses; `min=max=min(N, available)`).
    pub(super) fn step_src_cards_exact_n(
        &mut self,
        edge: &Edge<LoweredPayLoad>,
        qty: front_end::ast::Quantity,
        from: front_end::ast::CardSet,
    ) -> StepResult {
        use front_end::ast::Quantity;
        let Quantity::Int { int } = &qty else {
            // Defensive: the site gate guarantees an Int quantity.
            return self.dispatch(edge.clone());
        };
        let n = match crate::query::Evaluator::eval_int(int, &self.game_data) {
            Ok(v) => v,
            Err(e) => return StepResult::Error(e),
        };
        if n <= 0 {
            // `move 0 ...` is a no-op — no prompt.
            return self.dispatch(edge.clone());
        }
        let candidate_ids = match crate::query::Evaluator::eval_cardset(&from, &self.game_data) {
            Ok((_, ids)) => ids,
            Err(e) => return StepResult::Error(e),
        };
        let len = candidate_ids.len();
        if len == 0 {
            // Empty source: nothing to pick — no-op.
            return self.dispatch(edge.clone());
        }
        let expected = (n as usize).min(len);
        let display: Vec<crate::game_data::Card> = candidate_ids
            .iter()
            .map(|&id| self.game_data.get_card(id).cloned().unwrap_or_default())
            .collect();
        self.pending_quant = Some(crate::quantifier::PendingQuant {
            state: self.current_state,
            kind: crate::quantifier::PendingKind::CardsExactN {
                candidate_ids,
                expected,
                original: edge.clone(),
            },
        });
        self.emit_quant_trace(
            "SrcCardsExactN",
            format!("{} candidates, awaiting {}", len, expected),
        );
        StepResult::NeedsInput(InputType::ChooseCards {
            display,
            min: expected,
            max: expected,
            prompt: format!("Choose exactly {} card(s)", expected),
        })
    }

    /// Resume a `DealCount`: validate the count against the prompt bounds
    /// (re-prompting on violation), substitute the literal quantity, and
    /// re-scan (chaining — e.g. a dest fan-out follows).
    fn resume_deal_count(
        &mut self,
        value: i32,
        min: Option<i32>,
        max: Option<i32>,
        prompt: String,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        if min.is_some_and(|m| value < m) || max.is_some_and(|m| value > m) {
            self.pending_quant = Some(crate::quantifier::PendingQuant {
                state: self.current_state,
                kind: crate::quantifier::PendingKind::DealCount {
                    min,
                    max,
                    prompt: prompt.clone(),
                    original,
                },
            });
            return StepResult::NeedsInput(InputType::Number { min, max, prompt });
        }
        let repl = crate::quantifier::substitute_quantity(&original, value);
        self.emit_quant_trace("DealCount", format!("chose {}", value));
        self.quantify_or_dispatch(repl)
    }

    /// Resume a `SrcCardsExactN`: validate the count (re-prompting on
    /// mismatch), write the chosen ids into the synthetic memory, and
    /// re-scan the replacement edge (chaining — e.g. a dest fan-out follows).
    fn resume_cards_exact_n(
        &mut self,
        selected: Vec<usize>,
        candidate_ids: Vec<usize>,
        expected: usize,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        if selected.iter().any(|&i| i >= candidate_ids.len()) {
            return StepResult::Error(EngineError::ChooseCardsIndexOutOfRange);
        }
        let chosen: Vec<usize> = selected.iter().map(|&i| candidate_ids[i]).collect();
        if chosen.len() != expected {
            let display: Vec<crate::game_data::Card> = candidate_ids
                .iter()
                .map(|&id| self.game_data.get_card(id).cloned().unwrap_or_default())
                .collect();
            self.pending_quant = Some(crate::quantifier::PendingQuant {
                state: self.current_state,
                kind: crate::quantifier::PendingKind::CardsExactN {
                    candidate_ids,
                    expected,
                    original,
                },
            });
            return StepResult::NeedsInput(InputType::ChooseCards {
                display,
                min: expected,
                max: expected,
                prompt: format!("Choose exactly {} card(s). Please choose again.", expected),
            });
        }
        self.game_data.memories.insert(
            format!("Table_{}", crate::quantifier::SYNTH_MEMORY_KEY),
            crate::game_data::MemoryValue::CardSet(chosen.clone()),
        );
        let repl = crate::quantifier::substitute_cardset_memory(&original, &chosen);
        self.emit_quant_trace("SrcCardsExactN", format!("chose {}", chosen.len()));
        self.quantify_or_dispatch(repl)
    }

    /// Resume a `ComboSource`: validate the chosen set against the combo's
    /// filter (re-prompting on mismatch), then move the chosen cards via the
    /// synthetic-memory replacement edge.
    fn resume_combo_source(
        &mut self,
        selected: Vec<usize>,
        candidate_ids: Vec<usize>,
        filter: front_end::ast::FilterExpr,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        let mut chosen: Vec<usize> = selected.iter().map(|&i| candidate_ids[i]).collect();
        chosen.dedup(); // defensive: duplicate indices must not duplicate cards
        match crate::query::Evaluator::filter_card_ids(&filter, &chosen, &self.game_data) {
            Ok(matched) if matched.len() == chosen.len() => {
                self.game_data.memories.insert(
                    format!("Table_{}", crate::quantifier::SYNTH_MEMORY_KEY),
                    crate::game_data::MemoryValue::CardSet(chosen.clone()),
                );
                let repl = crate::quantifier::substitute_cardset_memory(&original, &chosen);
                self.emit_quant_trace("ComboSource", format!("laid down {}", chosen.len()));
                self.dispatch_concrete(repl)
            }
            Ok(_) => {
                let (display, _, max) = self.build_choose_cards(
                    &front_end::ast::Quantity::Quantifier {
                        quantifier: front_end::ast::Quantifier::Any,
                    },
                    &candidate_ids,
                );
                self.pending_quant = Some(crate::quantifier::PendingQuant {
                    state: self.current_state,
                    kind: crate::quantifier::PendingKind::Combo {
                        candidate_ids,
                        filter,
                        original,
                    },
                });
                StepResult::NeedsInput(InputType::ChooseCards {
                    display,
                    min: 0,
                    max,
                    prompt: "Selection does not match the combo (0 to skip). Please choose again."
                        .to_string(),
                })
            }
            Err(e) => StepResult::Error(e),
        }
    }

    /// Resume a `DestPlayerAny`: substitute the chosen player into the original
    /// edge, then re-scan — a remaining quantifier site (e.g. a card-amount
    /// `any` on `deal any from Stock to Hand of any`) chains to its own
    /// prompt; otherwise the concrete edge is dispatched.
    fn resume_dest_player_any(
        &mut self,
        idx: usize,
        candidates: Vec<String>,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        let Some(name) = candidates.get(idx) else {
            return StepResult::Error(EngineError::ChoosePlayerIdxOutOfRange {
                idx,
                len: candidates.len(),
            });
        };
        let name = name.clone();
        let repl = crate::quantifier::substitute_dest_player(&original, name.clone());
        self.emit_quant_trace("DestPlayerAny", format!("chose {}", name));
        self.quantify_or_dispatch(repl)
    }

    /// Resume a `SourcePlayerAny`: substitute the chosen player into the
    /// original edge's `from` owner, then re-scan (chaining) or dispatch.
    fn resume_source_player_any(
        &mut self,
        idx: usize,
        candidates: Vec<String>,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        let Some(name) = candidates.get(idx) else {
            return StepResult::Error(EngineError::ChoosePlayerIdxOutOfRange {
                idx,
                len: candidates.len(),
            });
        };
        let name = name.clone();
        let repl = crate::quantifier::substitute_source_player(&original, name.clone());
        self.emit_quant_trace("SourcePlayerAny", format!("chose {}", name));
        self.quantify_or_dispatch(repl)
    }

    /// Resume a `SetupAny`: substitute the chosen player into every any-site
    /// of the setup rule and dispatch the concrete edge.
    fn resume_setup_any(
        &mut self,
        idx: usize,
        candidates: Vec<String>,
        original: Edge<LoweredPayLoad>,
    ) -> StepResult {
        let Some(name) = candidates.get(idx) else {
            return StepResult::Error(EngineError::ChoosePlayerIdxOutOfRange {
                idx,
                len: candidates.len(),
            });
        };
        let name = name.clone();
        let repl = crate::quantifier::substitute_setup_any(&original, name.clone());
        self.emit_quant_trace("SetupAny", format!("chose {}", name));
        self.dispatch_concrete(repl)
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
            format!("Table_{}", crate::quantifier::SYNTH_MEMORY_KEY),
            crate::game_data::MemoryValue::CardSet(chosen.clone()),
        );
        let repl = crate::quantifier::substitute_cardset_memory(&original, &chosen);
        self.emit_quant_trace(
            "SrcCardsAnyOrRange",
            format!("chose {} cards", chosen.len()),
        );
        self.dispatch_concrete(repl)
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
            format!("Table_{}", crate::quantifier::SYNTH_MEMORY_KEY),
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

    /// Dispatch a concrete (quantifier-free) edge through the synthetic-overlay
    /// path: allocate a synth id, insert the edge, advance `current_state`.
    fn dispatch_concrete(&mut self, edge: Edge<LoweredPayLoad>) -> StepResult {
        let s = crate::quantifier::alloc_synth(&mut self.next_synth);
        self.pending_overlay.insert(s, vec![edge]);
        self.current_state = s;
        StepResult::Ok
    }

    /// If `edge` still carries a quantifier site, hand it to the matching step
    /// arm (sequential **chaining** — e.g. `deal any from Hand of any …`
    /// prompts for the source player, then for the cards); otherwise dispatch
    /// the concrete edge. Every resume arm ends here.
    fn quantify_or_dispatch(&mut self, edge: Edge<LoweredPayLoad>) -> StepResult {
        match crate::quantifier::scan_edge(&edge) {
            crate::quantifier::QuantSite::None => self.dispatch_concrete(edge),
            crate::quantifier::QuantSite::DestPlayerAll { pc } => {
                self.step_dest_player_all(&edge, pc)
            }
            crate::quantifier::QuantSite::DestPlayerAny { pc } => {
                self.step_dest_player_any(&edge, pc)
            }
            crate::quantifier::QuantSite::SourcePlayerAny { pc } => {
                self.step_source_player_any(&edge, pc)
            }
            crate::quantifier::QuantSite::SrcCardsAnyOrRange { qty, from } => {
                self.step_src_cards_any_or_range(&edge, qty, from)
            }
            crate::quantifier::QuantSite::SrcCardsExactN { qty, from } => {
                self.step_src_cards_exact_n(&edge, qty, from)
            }
            crate::quantifier::QuantSite::DealCount { qty, from } => {
                self.step_deal_count(&edge, qty, from)
            }
            crate::quantifier::QuantSite::ComboSource { combo, from } => {
                self.step_combo_source(&edge, combo, from)
            }
        }
    }

    /// Resolve a dest-quantifier `PlayerCollection` to player names, dropping
    /// any index that no longer maps to a live player (defensive).
    fn resolve_player_names(
        &self,
        pc: &front_end::ast::PlayerCollection,
    ) -> Result<Vec<String>, EngineError> {
        let idxs = crate::query::Evaluator::resolve_player_collection(pc, &self.game_data)?;
        Ok(idxs
            .iter()
            .filter_map(|&i| self.game_data.players.get(i).map(|p| p.name.clone()))
            .collect())
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
                EngineError::ChooseCardsIndexOutOfRange,
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

#[cfg(test)]
#[path = "quant_driver_tests.rs"]
mod tests;
