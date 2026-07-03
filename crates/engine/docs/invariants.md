---
type: agent_wiki_node
module: crates::engine
scope: [engine::game_data, engine::interpreter, engine::action, engine::query, engine::controller]
topics: [invariants, guardrails, state-boundaries, edge-cases, ordering]
associated_files:
  - crates/engine/src/game_data.rs
  - crates/engine/src/interpreter.rs
  - crates/engine/src/action.rs
  - crates/engine/src/query.rs
  - crates/engine/src/controller.rs
last_validated: 2026-07-02
---

# System Invariants & Guardrails

> **Read this before modifying any engine code.** These rules are derived directly from the source.
> Violating them will silently corrupt game state or hang the run loop. Each invariant is numbered
> (I-1 … I-15) and cross-referenced from other pages (e.g. `I-5`, `I-8`) — preserve those IDs when
> editing.

For the panic conditions that enforce some of these, see [`error-handling.md`](./error-handling.md).

---

> **I-1 — Index indirection for the current player.**
> `crates::engine::game_data::GameData::current_player: Option<usize>` indexes **`turn_order`**,
> not `players`. `turn_order` holds indices into `players`. The helper
> `crates::engine::game_data::GameData::get_current_player`
> (`crates/engine/src/game_data.rs:178-183`) does `turn_order[current_player] → players[…]`. Code
> that treats `current_player` as a player index is wrong.
> `front_end::ast::ActionRule::CycleAction` (`crates/engine/src/action.rs:206-220`) deliberately
> stores the *turn-order position* (`turn_order.iter().position(...)`), not the player index.

> **I-2 — `GameData::new()` initializes `current_player = Some(0)` with an empty `turn_order`.**
> (`crates/engine/src/game_data.rs:113`). Until
> `front_end::ast::SetUpRule::CreateTurnorder`/`CreatePlayer` populate `turn_order`, calling
> `get_current_player()` returns `None` (safe), but any direct indexing of `turn_order[0]` would
> panic. The `Some(0)` sentinel must not be assumed valid before setup.

> **I-3 — Condition vs. EndCondition edge indexing is INVERTED.**
> Both require **exactly 2 outgoing edges** or `step()` returns `Error`
> (`crates/engine/src/interpreter.rs:72-76`, `crates/engine/src/interpreter.rs:99-103`). But the
> chosen edge differs:
> - `front_end::ir::Payload::Condition` (`crates/engine/src/interpreter.rs:81-86`):
>   `should_take_else = result != negated`; `true` → `edges[1]`, `false` → `edges[0]`.
> - `front_end::ir::Payload::EndCondition` (`crates/engine/src/interpreter.rs:112-117`):
>   `should_exit = result != negated`; `true` → `edges[0]` (exit), `false` → `edges[1]` (continue).
>
> So edge **0 is the "true/exit" branch for `EndCondition` but the "false" branch for `Condition`**.
> Any change to the IR builder's edge ordering or to these match arms must be mirrored across both
> or games will branch backwards.

> **I-4 — `GameOver` requires *both* no outgoing edges AND `current_state == ir.goal`.**
> (`crates/engine/src/interpreter.rs:29-34`). A dead-end state that is not the goal yields
> `crates::engine::interpreter::StepResult::Error("No outgoing edges and not at goal state")`, not
> `GameOver`. An agent adding a terminal state must ensure it is registered as
> `front_end::ir::Ir::goal`.

> **I-5 — `StageRoundCounter` is applied exactly once per traversal (was: twice).**
> The interpreter's `step()` is the single mutator for `StageRoundCounter` and
> `EndStage` payloads: it mutates `game_data` and then calls `execute_edge()`
> (`crates/engine/src/interpreter.rs`), which calls `action::execute()`.
> `action::execute()`'s match has **no** `StageRoundCounter`/`EndStage` arms — they
> fall through to the catch-all `_ => {}` (`crates/engine/src/action.rs`). Agents
> must NOT re-add those arms or the round counter increments twice and
> `leave_stage` is called twice.
> Note: the `EndStage` payload is currently never emitted by the front_end IR
> builder (stages exit via `EndAction` action edges or via the EndCondition-exit
> pop below); the `EndStage` interpreter arm is retained for completeness. A
> normal `EndCondition` exit (`should_exit == true`) pops `stage_stack` via
> `leave_stage(stage)` in the EndCondition arm.

> **I-6 — Cards have no reverse location index; location is found by linear scan.**
> `crates::engine::game_data::GameData::add_card` (`crates/engine/src/game_data.rs:154-157`) takes
> a `_location_id` parameter it **ignores** — it only appends to `cards` and returns the new id.
> The caller (`crates/engine/src/action.rs:116-119`) then pushes the id into
> `locations[loc_idx].cards`. To find which location holds a card, the engine scans all locations
> (`crates/engine/src/query.rs:964-968`, `crates/engine/src/query.rs:1020-1026`,
> `crates/engine/src/query.rs:1376-1388`, `crates/engine/src/action.rs:357-359`).
> `crates::engine::action::execute_cardset_move` scans **all locations per moved card**
> (`crates/engine/src/action.rs:357-359`) — O(cards × locations). Do not assume O(1) card location
> lookup.

> **I-7 — The input buffer is a LIFO stack, not a queue.**
> `crates::engine::interpreter::Interpreter::provide_input` pushes
> (`crates/engine/src/interpreter.rs:151-153`), `step()` pops
> (`crates/engine/src/interpreter.rs:43,60`). In normal flow only one input is ever buffered at a
> time, but if an agent ever pushes multiple, they are consumed in reverse order.

> **I-8 — Out-of-range `Choice`/`Optional` input is a silent no-op that stalls the FSM.**
> `step()` does `if let Some(choice_edge) = edges.get(input.idx()) { self.execute_edge(...) }` then
> unconditionally returns `Ok` (`crates/engine/src/interpreter.rs:42-57`, `59-69`). If
> `input.idx()` is out of range, `execute_edge` is skipped, `current_state` is **not** advanced,
> and the next `step()` re-enters the same state — which, now that the buffer is empty, yields
> `NeedsInput` again. For `crates::engine::controller::InputSource::TestFile` this **consumes the
> next recorded line** as a re-prompt; for `InputSource::Player` the controller's own validation
> loop (`crates/engine/src/controller.rs:86-96`, using `idx > max_index`) re-invokes the closure.
> There is no central validation: the interpreter trusts the index, the controller validates only
> the `Player` path.

> **I-9 — `set_memory` does not set; it increments an `Int` memory by 1.**
> `crates::engine::game_data::GameData::set_memory` (`crates/engine/src/game_data.rs:272-278`)
> ignores its `memory_type` argument entirely and, if the stored value is
> `crates::engine::game_data::MemoryValue::Int`, does `*v += 1`. It silently no-ops on non-`Int`
> memories. `crates::engine::game_data::GameData::reset_memory`
> (`crates/engine/src/game_data.rs:280-286`) only resets `Int` memories. This is the current
> behavior, not a spec'd design — agents implementing DSL `SetMemory` semantics must not assume
> general assignment here.

> **I-10 — `add_memory` initializes some `MemoryType`s to mismatched `MemoryValue`s.**
> (`crates/engine/src/game_data.rs:251-265`):
> `front_end::ast::MemoryType::Player` → `MemoryValue::Int(0)` (not a player),
> `front_end::ast::MemoryType::TeamCollection` → `MemoryValue::Int(0)` (not a collection). Reads of
> these (`crates/engine/src/query.rs:863-884`, `crates/engine/src/query.rs:637-647`) will therefore
> fail type checks until something writes a correctly-typed value. Agents adding new memory writes
> must respect the read-side expected `MemoryValue` variant.

> **I-11 — `leave_stage` pops the stage stack until (and including) the named stage.**
> (`crates/engine/src/game_data.rs:227-234`). This permits multi-stage jumps (an end-condition that
> exits several nested stages at once). If the named stage is not on the stack, the **entire stack
> is drained**. `crates::engine::game_data::GameData::get_current_stage`
> (`crates/engine/src/game_data.rs:212-214`) returns `stage_stack.last()`.

> **I-12 — `enter_stage` is invoked by the interpreter via `ensure_stage_entered`.**
> Stage entry (`GameData::enter_stage`) is called from
> `GameData::ensure_stage_entered` (`crates/engine/src/game_data.rs`), which the
> interpreter calls on the first encounter of any stage-carrying payload
> (`EndCondition`, `StageRoundCounter`, `EndStage`) for a stage not yet on
> `stage_stack`. It is idempotent (guarded by `stage_stack` membership).
> `ensure_stage_entered` marks **all** players in-stage for the entered stage
> (participants-by-default); `ActionRule::OutAction` (`end <stage>` / `out of`)
> removes specific players afterwards. `resolve_turn` and `RuntimePlayer::Next`
> rely on `in_stage[current_stage]`; without `ensure_stage_entered` they find no
> eligible player and `current_player` becomes `None`.

> **I-13 — `resolve_turn` / `next_player` find the next *eligible* player, wrapping the turn order.**
> (`crates/engine/src/game_data.rs:185-197`, `236-249`). Eligible = `in_game && in_stage[current_stage]`.
> If none is eligible, `current_player` becomes `None` and the game is effectively stuck (no
> `Error` is raised). `crates::engine::game_data::GameData::next_player` uses `.unwrap()` on the
> found position (`crates/engine/src/game_data.rs:192`) — safe only because `resolve_turn` returning
> `Some(idx)` guarantees the idx is in `turn_order`.

> **I-14 — `eval_cardset` returns `(location_idx, card_ids)`; the location is best-effort.**
> `crates::engine::query::Evaluator::eval_cardset` returns `(usize, Vec<usize>)`. For
> `front_end::ast::CardSet::Memory` with cards not found in any location, it returns `(0, card_ids)`
> (`crates/engine/src/query.rs:970`) — **location index 0 is a fallback sentinel**, not a real
> answer. `crates::engine::query::Evaluator::infer_location_from_cards`
> (`crates/engine/src/query.rs:1372-1389`) similarly falls back to `Ok(0)`. Consumers that index
> `locations[0]` after such a result may read an unrelated pile.

> **I-15 — `InputSource::Player`'s validation loop can spin forever.**
> `crates/engine/src/controller.rs:86-96` re-calls `callback(input_type)` with `continue` while the
> returned `Choice.idx` exceeds `max_index`. A buggy closure that always returns an out-of-range
> index will hang the run loop with no error. The `TestFile` path has no such loop (it consumes one
> line per request and errors on exhaustion).
