---
type: agent_wiki_node
module: crates::engine
scope: [engine::game_data, engine::interpreter, engine::action, engine::query, engine::controller]
topics: [invariants, guardrails, state-boundaries, edge-cases, ordering]
associated_files:
  - crates/engine/src/game_data.rs
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/interpreter/quant_driver.rs
  - crates/engine/src/action.rs
  - crates/engine/src/query/mod.rs
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/quantifier.rs
last_validated: 2026-08-11
---

# System Invariants & Guardrails

> **Read this before modifying any engine code.** These rules are derived directly from the source.
> Violating them will silently corrupt game state or hang the run loop. Each invariant is numbered
> (I-1 … I-25) and cross-referenced from other pages (e.g. `I-5`, `I-8`, `I-18`) — preserve those
> IDs when editing.

For the panic conditions that enforce some of these, see [`error-handling.md`](./error-handling.md).

---

> **I-1 — Index indirection for the current player.**
> `crates::engine::game_data::GameData::current_player: Option<usize>` indexes **`turn_order`**,
> not `players`. `turn_order` holds indices into `players`. The helper
> `crates::engine::game_data::GameData::get_current_player`
> (`crates/engine/src/game_data.rs:228-246`) does `turn_order[current_player] → players[…]`. Code
> that treats `current_player` as a player index is wrong.
> `front_end::ast::ActionRule::CycleAction` (`crates/engine/src/action.rs:349-371`) deliberately
> stores the *turn-order position* (`turn_order.iter().position(...)`), not the player index.

> **I-2 — `GameData::new()` initializes `current_player = Some(0)` with an empty `turn_order`.**
> (`crates/engine/src/game_data.rs:113`). Until
> `front_end::ast::SetUpRule::CreateTurnorder`/`CreatePlayer` populate `turn_order`, calling
> `get_current_player()` returns `None` (safe), but any direct indexing of `turn_order[0]` would
> panic. The `Some(0)` sentinel must not be assumed valid before setup.

> **I-3 — Condition vs. EndCondition edge indexing is INVERTED.**
> Both require **exactly 2 outgoing edges** or `step()` returns `Error`
> (`crates/engine/src/interpreter/mod.rs:230-236` for `Condition`,
> `crates/engine/src/interpreter/mod.rs:271-277` for `EndCondition`). But the
> chosen edge differs:
> - `front_end::ir::Payload::Condition` (`crates/engine/src/interpreter/mod.rs:229-265`):
>   `should_take_else = result != negated`; `true` → `edges[0]` (this edge is correct — take it),
>   `false` → `edges[1]`.
> - `front_end::ir::Payload::EndCondition` (`crates/engine/src/interpreter/mod.rs:266-314`):
>   `should_exit = result == negated`; `true` → `edges[0]` (exit), `false` → `edges[1]` (continue).
>
> So edge **0 is the "true/exit" branch for `EndCondition` but the "false" branch for `Condition`**.
> (Both use opposite formulas due to opposite `negated` values on edge 0 in the IR:
> `Condition` edge 0 has `negated: false`, `EndCondition` edge 0 has `negated: true`.)
> Any change to the IR builder's edge ordering or to these match arms must be mirrored across both
> or games will branch backwards.

> **I-4 — `GameOver` requires *both* no outgoing edges AND `current_state == ir.goal`.**
> (`crates/engine/src/interpreter/mod.rs:120-128`). A dead-end state that is not the goal yields
> `crates::engine::interpreter::StepResult::Error(EngineError::NoOutgoingEdges)` —
> Display `"No outgoing edges from state … and not at goal state"` — not `GameOver`. An agent
> adding a terminal state must ensure it is registered as `front_end::ir::Ir::goal`.

> **I-5 — `StageRoundCounter` is applied exactly once per traversal.**
> The interpreter's `step()` is the single mutator for `StageRoundCounter` and
> `EndStage` payloads: it mutates `game_data` and then calls `execute_edge()`
> (`crates/engine/src/interpreter/mod.rs:315-331` for `StageRoundCounter`,
> `:332-345` for `EndStage`), which calls `action::execute()`.
> `action::execute()`'s match has **no** `StageRoundCounter`/`EndStage` arms — they
> fall through to the catch-all `_ => {}` (`crates/engine/src/action.rs:45-59`). Agents
> must NOT re-add those arms or the round counter increments twice and
> `leave_stage` is called twice.
> Note: the `EndStage` payload is currently never emitted by the front_end IR
> builder (stages exit via `EndAction` action edges or via the EndCondition-exit
> pop below); the `EndStage` interpreter arm is retained for completeness. A
> normal `EndCondition` exit (`should_exit == true`) pops `stage_stack` via
> `leave_stage(stage)` in the EndCondition arm.

> **I-6 — Cards have no reverse location index; location is found by linear scan.**
> `crates::engine::game_data::GameData::add_card` (`crates/engine/src/game_data.rs:156-159`) takes
> a `_location_id` parameter it **ignores** — it only appends to `cards` and returns the new id.
> The caller (`crates/engine/src/action.rs:119-121`) then pushes the id into
> `locations[loc_idx].cards`. To find which location holds a card, the engine scans all locations
> (`crates::engine::game_data::GameData::find_location_of_card`,
> `crates/engine/src/game_data.rs:169-173`, used by `query/cardset.rs:66,175`;
> `crates/engine/src/query/cardset.rs:595-613` `infer_location_from_cards`).
> `crates::engine::action::execute_cardset_move` scans **all locations per moved card**
> (`crates/engine/src/action.rs:386-388`) — O(cards × locations). Do not assume O(1) card location
> lookup.

> **I-7 — The input buffer is a LIFO stack, not a queue.**
> `crates::engine::interpreter::Interpreter::provide_input` pushes
> (`crates/engine/src/interpreter/mod.rs:372-374`), `step()` pops
> (`crates/engine/src/interpreter/mod.rs:175` for `Choice`, `:196` for `Optional`). In normal flow
> only one input is ever buffered at a time, but if an agent ever pushes multiple, they are
> consumed in reverse order. The quantifier driver also peeks (`last()`) before popping
> (`crates/engine/src/interpreter/quant_driver.rs:41,81`) so a misrouted answer is left in place
> when the pending kind does not match — see I-19.

> **I-8 — Out-of-range `Choice`/`Optional`/`ChoosePlayer`/`ChooseCards`/`Number` input is a
> silent no-op (interpreter) / re-prompt (controller) / fatal error (resume).**
> For `Choice`/`Optional`: `step()` does
> `if let Some(choice_edge) = edges.get(input.idx()) { self.execute_edge(...) }` then
> unconditionally returns `Ok` (`crates/engine/src/interpreter/mod.rs:173-194` for `Choice`,
> `:195-228` for `Optional`). If `input.idx()` is out of range, `execute_edge` is skipped,
> `current_state` is **not** advanced, and the next `step()` re-enters the same state — which,
> now that the buffer is empty, yields `NeedsInput` again. For
> `crates::engine::controller::InputSource::TestFile` this **consumes the next recorded line** as a
> re-prompt; for `InputSource::Player` the controller's own validation loop
> (`crates/engine/src/controller/mod.rs:439-457`, `validate_player_input`) re-invokes the closure.
> The controller's `validate_player_input` checks:
> - `Choice`: `idx <= max_index`;
> - `ChoosePlayer`: `idx < candidates.len()`;
> - `ChooseCards`: no `i >= display.len()`, `selected.len() >= min`, `selected.len() <= max`;
> - `Number`: `value` within `min..=max` when the bounds are present.
> A `Player`-sourced answer that fails is dropped and the closure re-invoked (the loop can spin
> forever — see I-15). The interpreter's *resume* path is stricter: a `ChoosePlayer` `idx` outside
> `candidates` returns `StepResult::Error(EngineError::ChoosePlayerIdxOutOfRange)`
> (`crates/engine/src/interpreter/quant_driver.rs:366-369`), and a `ChooseCards` index outside
> `candidate_ids` returns `StepResult::Error(EngineError::ChooseCardsIndexOutOfRange)`
> (`quant_driver.rs:538-539`). The `TestFile` path has no validation loop: it consumes one line per
> request and **errors on exhaustion** (`controller/mod.rs:351`,
> `EngineError::TestInputExhausted`).

> **I-9 — `set_memory` assigns the caller-provided `MemoryValue` verbatim.**
> `crates::engine::game_data::GameData::set_memory` (`crates/engine/src/game_data.rs:339-341`)
> inserts the `MemoryValue` it is given, overwriting any prior value. It is the write-side
> primitive used by `ActionRule::SetMemory` *after* the `MemoryType` expression has been
> evaluated by `action.rs` (eval failures surface as recoverable `Err`s). `reset_memory`
> (`crates/engine/src/game_data.rs:343-345`) resets every variant to its typed zero
> (`Int`→0, `String`→`""`, `Team`→`""`, collections→empty).

> **I-10 — `add_memory` initialises typed memories correctly.**
> `crates/engine/src/game_data.rs` `add_memory(key, owner_name, memory_type,
> initial)` inserts the caller-provided `initial` `MemoryValue` verbatim, or a
> per-type default: `Int`→`Int(0)`, `String`→`""`, `Team`→`""`, collections →
> empty. `MemoryType::Player` → `MemoryValue::String(owner_name)` (player
> memories store *names*, matching `SetMemory`'s convention) and
> `MemoryType::TeamCollection` → `MemoryValue::TeamCollection(vec![])`.
> Setup-time evaluation of the declared expression happens in `action.rs`
> (`evaluate_memory_type`). Agents adding new memory writes must respect the
> read-side expected `MemoryValue` variant.

> **I-11 — `leave_stage` pops the stage stack until (and including) the named stage.**
> (`crates/engine/src/game_data.rs:252-259`). This permits multi-stage jumps (an end-condition that
> exits several nested stages at once). If the named stage is not on the stack, the **entire stack
> is drained**. `crates::engine::game_data::GameData::get_current_stage`
> (`crates/engine/src/game_data.rs:229-231`) returns `stage_stack.last()`.

> **I-12 — `enter_stage` is invoked by the interpreter via `ensure_stage_entered`.**
> Stage entry (`GameData::enter_stage`) is called from
> `GameData::ensure_stage_entered` (`crates/engine/src/game_data.rs:244-250`), which the
> interpreter calls on the first encounter of any stage-carrying payload
> (`EndCondition` at `interpreter/mod.rs:278`, `StageRoundCounter` at `:316`) for a stage not yet
> on `stage_stack`. It is idempotent (guarded by `stage_stack` membership).
> `ensure_stage_entered` marks **all** players in-stage for the entered stage
> (participants-by-default); `ActionRule::OutAction` (`end <stage>` / `out of`)
> removes specific players afterwards. `resolve_turn` and `RuntimePlayer::Next`
> rely on `in_stage[current_stage]`; without `ensure_stage_entered` they find no
> eligible player and `current_player` becomes `None`.

> **I-13 — `resolve_turn` / `next_player` find the next *eligible* player, wrapping.**
> (`crates/engine/src/game_data.rs` `next_eligible_player` / `previous_eligible_player`). Eligible =
> `in_game && in_stage[current_stage]`. The scan skips ineligible players; with no eligible *other*
> player the **current player itself** is returned when it is still eligible (the turn wraps onto
> itself instead of stranding the game), and `None` only when nobody — including the current player
> — is eligible.
> `crates::engine::game_data::GameData::next_player` uses `unwrap_or_else(|| panic!(...))` on the
> found position — safe only because `resolve_turn` returning `Some(idx)` guarantees the idx is in
> `turn_order`.
> **Note:** `cycle to next` with **no** eligible player (not even the current one) is a **no-op**
> (`Ok(())`) — it never errors. The stage's auto-end (I-24) terminates the stage from the
> loop-back. `end turn` behaves identically (it never strands `current_player = None` while the
> current player is still eligible).

> **I-14 — `eval_cardset` returns `(location_idx, card_ids)`; the location is best-effort.**
> `crates::engine::query::Evaluator::eval_cardset` returns `(usize, Vec<usize>)`. For
> `front_end::ast::CardSet::Memory` with cards not found in any location, it returns `(0, card_ids)`
> (`crates/engine/src/query/cardset.rs:70`) — **location index 0 is a fallback sentinel**, not a
> real answer. `crates::engine::query::Evaluator::infer_location_from_cards`
> (`crates/engine/src/query/cardset.rs:595-613`) similarly falls back to `Ok(0)`. Consumers that
> index `locations[0]` after such a result may read an unrelated pile.

> **I-15 — `InputSource::Player`'s validation loop can spin forever.**
> `crates/engine/src/controller/mod.rs:291-297` re-calls `callback(input_type)` with `continue`
> while `validate_player_input(&raw, &input_type)` (`controller/mod.rs:439-457`) returns `false`.
> The loop spins on **any** out-of-range answer (`Choice`/`ChoosePlayer`/`ChooseCards`/`Number` —
> see I-8 for the per-variant checks). A buggy closure that always returns an out-of-range
> answer will hang the run loop with no error. The `TestFile` path has no such loop (it consumes
> one line per request and errors on exhaustion — `controller/mod.rs:351`).

---

## Quantifier Subsystem Invariants (I-16 … I-20)

The Stage-5 quantifier preprocessor (`crates/engine/src/quantifier.rs` and
`crates/engine/src/interpreter/quant_driver.rs`) introduces five invariants. They govern
synthetic-state allocation, the overlay's key discipline, the synthetic memory slot, the
pending-resume state match, and the setup-`Any` guard.

> **I-16 — Synthetic `StateID`s are allocated from `u32::MAX - 1` decrementing (via `wrapping_sub`),
> so they never collide with the densely-from-0-allocated real IR ids.**
> `crates::engine::interpreter::Interpreter::new` seeds `next_synth = u32::MAX - 1`
> (`crates/engine/src/interpreter/mod.rs:59`). `crates::engine::quantifier::alloc_synth`
> (`crates/engine/src/quantifier.rs:134-138`) reads the current value, then advances the counter
> with `next_synth.wrapping_sub(1)`. `wrapping_sub` (not `-`) prevents an overflow panic on
> pathological reuse; the `u32` id space (2³²) is effectively unlimited for any realistic game.
> Real IR ids, allocated densely from 0 upward by the `front_end` IR builder, can therefore never
> be shadowed by a synthetic id.

> **I-17 — `pending_overlay` is keyed only by synthetic `StateID`s; real IR ids are never inserted.**
> The overlay (`crates::engine::interpreter::Interpreter::pending_overlay`,
> `crates/engine/src/interpreter/mod.rs:36`) is the bridge between the quantifier preprocessor and
> the unchanged `action::execute` path. Every `insert` into it passes a synthetic id freshly
> returned by `alloc_synth`:
> - `step_dest_player_all` inserts chain ids at `quant_driver.rs:148`;
> - `resume_dest_player_any` inserts the resume id at `quant_driver.rs:231`;
> - `resume_cards_any_or_range` inserts the resume id at `quant_driver.rs:272`;
> - `resume_dest_all_then_cards` inserts the fan-out ids at `quant_driver.rs:323`.
>
> The overlay therefore never shadows `ir.states` — `step()`'s overlay-dispatch arm
> (`interpreter/mod.rs:81-103`) only fires on a synthetic id, and the real-edge lookup
> (`interpreter/mod.rs:110`) is reached as soon as the FSM returns to a real IR state.

> **I-18 — `SYNTH_MEMORY_KEY` (`"__quantifier_overlay_cards"`) is stored under the
> owner-prefixed key `"Table___quantifier_overlay_cards"` in `game_data.memories` just before
> dispatching a replacement edge and is removed at the top of `step()` once the FSM returns to
> a real IR state.**
> The slot is written by `resume_cards_any_or_range` (`quant_driver.rs:264-267`) and
> `resume_dest_all_then_cards` (`quant_driver.rs:307-310`) immediately before substituting the
> chosen card ids into the replacement edge's `from`; both sites prefix the key with `"Table_"`
> (the memory-ownership model, see `developer-notes.md` §1.1). It is removed by `step()`'s
> cleanup block (`crates/engine/src/interpreter/mod.rs:69-79`), which removes
> `"Table_{SYNTH_MEMORY_KEY}"` when (a) `current_state` is a real IR state, (b) the overlay has
> no entry for it, and (c) `memories` still contains the slot. A user `.cgdsl` program that
> later `CreateMemory`s the same key would otherwise be corrupted; this invariant guarantees
> the slot's lifetime is bounded by the quantifier edge.

> **I-19 — `pending_quant.state` must equal `current_state` for the resume to fire.**
> `crates::engine::interpreter::Interpreter::take_quant_resume`
> (`crates/engine/src/interpreter/quant_driver.rs:15-101`) peeks at `pending_quant` and the
> `input_buffer`; if `pq.state != self.current_state` it returns `None` (line 39) **before**
> popping either — leaving the pending request and the input buffered. This means a quantifier
> prompt is only ever resumed at the state it was issued from. The FSM does not advance while a
> quantifier awaits input (the controller's `NeedsInput` branch holds `current_state` still), so
> in normal flow the equality always holds; the guard exists so a future caller cannot misroute an
> answer.

> **I-20 — Setup rules containing `Quantifier::Any` are resolved by a player prompt
> before dispatch.**
> `step()`'s setup-`Any` guard (`crates/engine/src/interpreter/mod.rs:155-161`) checks
> `crate::quantifier::setup_contains_any(setup)` for any `Payload::Action(GameRule::SetUp {
> setup })` edge. If any element collection is `Aggregate { Quantifier::Any }`, `step()` issues a
> `ChoosePlayer` prompt (`step_setup_any`) and, on resume, substitutes the chosen player into
> *every* any-site of the setup (`quantifier::substitute_setup_any`) *before* calling
> `execute_edge` — no `GameData` mutation occurs until the prompt is answered. `Quantifier::All`
> in setup is supported (it expands to all in-game players via `resolve_owner_to_names` →
> `Evaluator::resolve_player_collection`, see I-10's setup note in
> [`lifecycle.md`](./lifecycle.md) §2).

> **I-21 — Stale input on quantifier prompt mismatch is discarded.**
> `take_quant_resume` (`crates/engine/src/interpreter/quant_driver.rs:78`) pops mismatched input
> from the buffer when the pending quantifier kind does not match the input variant (e.g. a `Choice`
> arrives while a `CardsAnyOrRange` prompt is in flight). Without this, the stale input would never
> be consumed, `take_quant_resume` would return `None` every step, and `scan_edge` would re-trigger
> the same quantifier site — creating an infinite prompt loop. The pending quantifier itself is
> preserved so a future matching input can still resolve it.

> **I-22 — Positional card queries on empty locations resolve to the bare location.**
> `Evaluator::eval_group`'s `CardPosition` arm (`crates/engine/src/query/cardset.rs:264-280`)
> extracts the location name from positional queries (`Top`, `Bottom`, `At`) before evaluating
> the card position. If the location exists but the positional lookup fails (e.g. `top(Discard)` on
> an empty pile), it gracefully returns `(loc_idx, vec![])` instead of erroring. This is necessary
> for destination resolution in `execute_cardset_move` where only the location index matters, not
> individual card IDs. The location-existence check still fails if the location does not exist.

> **I-23 — Inputs are rejected if `player_id` does not match the current player.**
> `validate_player_input` (`crates/engine/src/controller/mod.rs`) checks
> `input.player_id == current_player_name` before any range validation. During active
> play, only the current player's inputs are accepted. Pre-setup
> (`current_player == None`), `current_player_name` is empty and all inputs pass. The
> check uses the player **name** (`Player::name: String`) for readability. The `Player`
> closure path re-prompts on rejection; the `TestFile` path parses `player_id` from an
> optional `Name:` prefix (defaulting to `"P1"`).

> **I-24 — Ineligible-player skip & stage auto-end.**
> A player who is out of the game or out of the current stage is never asked for input and
> none of their instructions execute:
> - `Interpreter::step()` checks (before the quantifier preprocessor and per-payload dispatch)
>   `current_player_ineligible()` (no current stage → never ineligible; else `!in_game` or
>   `!in_stage[current_stage]`). When ineligible and the first outgoing edge's payload is
>   *skippable* (`payload_is_skippable`: everything except `EndCondition`/`StageRoundCounter`/
>   `EndStage` bookkeeping, `SetUp` rules, and `CycleAction`/`EndAction`), the edge is advanced
>   through **without executing** and a `TraceEvent::Skipped { player, stage }` is emitted.
> - Bookkeeping payloads process normally, so the stage's loop-back re-evaluates the end
>   condition (with the new current player, moved by the cycle action) — "the stage skips all
>   instructions except cycle actions until the end of the stage".
> - **Auto-end:** the `EndCondition` arm also exits the stage when (a) **no player is in the
>   game** (`players` non-empty and all `!in_game` — the game then runs out to the goal with an
>   empty winner set) or (b) **no player is in this stage** (`in_stage[stage]` all `false`).
>   These checks are skipped for zero-player `GameData`s so hand-built dispatch tests keep pure
>   semantics.
> - The skip applies per edge while the condition holds; there is no persistent "skip mode"
>   flag — each step re-evaluates eligibility, so skipping naturally stops once a cycle lands
>   on an eligible player.
>
> **I-25 — The winner set is the set of in-game players.**
> At game end (`GameOver`), the winner set is every player still `in_game`, in declaration
> order (`GameData::winner_names()`); empty when nobody won. Explicit winner declarations
> (`winner is X`, `end game with winner X`) reduce to the same rule because they eliminate
> everyone else. The set is emitted as a `TraceEvent::GameOver { winners }` on the transition
> into `GameOver`, written into the trace-file footer, and printed by `cgdsl-play`.
