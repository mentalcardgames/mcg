---
type: agent_wiki_node
module: crates::engine
scope: [engine::controller, engine::interpreter, engine::action, engine::game_data]
topics: [lifecycle, run-loop, initialization, sequencing, termination]
associated_files:
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/interpreter/quant_driver.rs
  - crates/engine/src/action.rs
  - crates/engine/src/game_data.rs
  - crates/engine/src/quantifier.rs
last_validated: 2026-08-11
---

# Lifecycle & Runtime Sequencing

This page describes *when* things happen: construction of the engine, the setup phase, the play
loop, and termination/shutdown. For the *shapes* of the values involved see
[`data-structures.md`](./data-structures.md); for rules that must not be broken during these phases
see [`invariants.md`](./invariants.md).

---

## The Run Loop (Orchestration)

`crates::engine::controller::Controller::run` (`crates/engine/src/controller/mod.rs:266-285`) is
the single entry point and the entire main loop. It is a plain synchronous `loop { … }`:

```rust
// crates/engine/src/controller/mod.rs:266-285  (paraphrased for clarity; see source for exact lines)
fn run(&mut self) -> Result<GameData, EngineError> {
    loop {
        self.emit_event();                                   // optional event_sender callback
        *self.step_count.lock().unwrap() += 1;               // shared with the trace file
        match self.interpreter.step() {
            StepResult::Ok => continue,                      // advanced one edge; keep going
            StepResult::NeedsInput(input_type) => {            // stalled; ask InputSource
                let input = self.get_input(input_type)?;
                self.interpreter.provide_input(input);       // push onto LIFO buffer
            }
            StepResult::GameOver => {                          // terminal
                self.emit_event();                             //   one final event
                return Ok(self.interpreter.game_data.clone()); //   deep-clone terminal state
            }
            StepResult::Error(e) => return Err(e),             // propagate EngineError
        }
    }
}
```

Key sequencing facts:

- `crates::engine::controller::Controller::emit_event`
  (`crates/engine/src/controller/mod.rs:427-431`) is called at the **top of every iteration**
  (`mod.rs:270`) *and* once more just before returning `GameOver` (`mod.rs:281`). See
  [`observability.md`](./observability.md) §1.
- A `step_count: Arc<Mutex<usize>>` is incremented at the top of every iteration
  (`crates/engine/src/controller/mod.rs:271`); the same `Arc` is shared with the composed
  trace-sender closure so the trace file's `[Step NNN]` line numbers match the loop iteration
  (see `observability.md` §3).
- `crates::engine::interpreter::Interpreter::step` (`crates/engine/src/interpreter/mod.rs:70`)
  performs **exactly one** FSM transition per call. It may synchronously call
  `crates::engine::query::Evaluator` (reads) and `crates::engine::action::execute` (writes).
- When `step()` returns `crates::engine::interpreter::StepResult::NeedsInput`, `run()` consults the
  `crates::engine::controller::InputSource` (`Player` closure or `TestFile`), obtains one
  `crates::engine::interpreter::Input`, and pushes it via
  `crates::engine::interpreter::Interpreter::provide_input`
  (`crates/engine/src/interpreter/mod.rs:369-371`). The next `step()` pops it (LIFO — see I-7).
  A `NeedsInput` can carry `InputType::Choice`, `InputType::Optional`,
  `InputType::ChoosePlayer`/`ChooseCards` (from quantifier sites), or
  `InputType::Number` (from `bid`/`deal` count prompts); the controller runs the same loop for
  all of them (see `get_input`'s validation at `controller/mod.rs:439-457` and the `TestFile`
  syntaxes at `controller/mod.rs:328-426`).
- `crates::engine::interpreter::Interpreter::execute_edge`
  (`crates/engine/src/interpreter/mod.rs:362-367`) is the single mutator boundary: it sets
  `current_state = edge.to` and then calls
  `crates::engine::action::execute(edge.payload, &mut game_data)`.
- When `MCG_TRACE_LOG` resolves to a writable path, `run_game` wraps `controller.run()` in
  `std::panic::catch_unwind(AssertUnwindSafe(...))` (`crates/engine/src/controller/mod.rs:198-220`)
  so a panic is logged to the trace file before being re-raised — see
  [`observability.md`](./observability.md) §3.2.
- There is no scheduler, no green threads, no `Future`s; `run()` **blocks the calling thread** until
  `GameOver` or `Error`. See [`interfaces.md`](./interfaces.md) §6.

---

## State Lifecycle

### 1. Construction

`crates::engine::game_data::GameData::new` (`crates/engine/src/game_data.rs:118-130`) produces an
empty store with `current_player = Some(0)`, empty `stage_stack`, empty `memories`.
`crates::engine::controller::run_game` (`crates/engine/src/controller/mod.rs:96-250`) wraps it in
an `crates::engine::interpreter::Interpreter` via the canonical
`crates::engine::interpreter::Interpreter::new`
(`crates/engine/src/interpreter/mod.rs:46-62`) — which seeds `current_state = ir.entry`,
`input_buffer = Vec::new()`, `pending_overlay = HashMap::new()`, `next_synth = u32::MAX - 1`,
`pending_quant = None`, and stores the composed `trace_sender` — and a private
`crates::engine::controller::Controller` (`controller/mod.rs:253-261`).

Note: `GameData::new` sets `current_player = Some(0)` even though `turn_order` is empty — see
invariant I-2 in [`invariants.md`](./invariants.md).

### 2. Setup Phase

The IR's first edges carry
`front_end::ir::Payload::Action(front_end::ast::GameRule::SetUp { … })`. These run
`crates::engine::action::execute_setup_rule` (`crates/engine/src/action.rs:61-160`), which is the
**only** place `players`, `teams`, `turn_order`, `locations`, `cards`, `combos`, `precedences`,
`point_maps`, and `memories` are *created*. Order matters:

- `front_end::ast::SetUpRule::CreateLocation` resolves an owner via
  `Evaluator::resolve_owner_to_names` (plural); a resolution failure surfaces as a **recoverable
  error** (`Err("CreateLocation: failed to resolve owner …")`), so `CreatePlayer`/`CreateTeams`
  must precede it.
  `resolve_owner_to_names` now transparently routes
  `Owner::PlayerCollection { Aggregate { Quantifier::All } }` (the quantifier owner)
  through `Evaluator::resolve_player_collection` so a setup `CreateLocation` with `Owner =
  All` produces one location per in-game player — see invariant I-10 in
  [`invariants.md`](./invariants.md).
- `front_end::ast::SetUpRule::CreateCardOnLocation` requires the location to already exist;
  a missing location is a recoverable error (`Err("CreateCardOnLocation: location … not found")`).
- `front_end::ast::SetUpRule::{CreateTeams, CreateTurnorder, CreateTurnorderRandom,
  CreateLocation, CreateMemory, CreateMemoryWithMemoryType}` whose element collection is a
  `Quantifier::Any` are *rejected before dispatch* by the interpreter's setup-`Any` guard — see
  §3 below and invariant I-20.

> The setup-phase mutations `execute_setup_rule` makes are still applied through
> `Interpreter::execute_edge` → `action::execute`, *not* through a special-cased path. What
> `step()` adds *around* that dispatch for a `SetUp` payload is the setup-`Any` guard described
> below.

### 3. Play Phase

Each `crates::engine::interpreter::Interpreter::step` (`crates/engine/src/interpreter/mod.rs:70`)
either advances `current_state` via
`crates::engine::interpreter::Interpreter::execute_edge`
(`crates/engine/src/interpreter/mod.rs:362-367`:
`current_state = edge.to; action::execute(edge.payload, &mut game_data)`), or yields
`NeedsInput`/`GameOver`/`Error`. Conditions and end-conditions are evaluated live against the
current `GameData` by `crates::engine::query::Evaluator`.

`step()`'s body has a fixed pre-dispatch sequence, then the per-`Payload` dispatch:

```
(A) SYNTH_MEMORY_KEY cleanup      interpreter/mod.rs:71-85
(S) ineligible-player skip        interpreter/mod.rs (I-24)
(B) overlay dispatch                 interpreter/mod.rs:84-106
(C0) quantifier resume              interpreter/mod.rs:109-112  (take_quant_resume)
      - resolve real IR edges for current_state            interpreter/mod.rs:114-131
      - look up first edge                                  interpreter/mod.rs:130-...
(C1) quantifier preprocessor        interpreter/mod.rs:137-153  (scan_edge)
      - setup-Any guard (Payload::Action(GameRule::SetUp))  interpreter/mod.rs:160-163
      - per-Payload dispatch                                interpreter/mod.rs:155-360
```

The pre-dispatch arms, in order:

1. **(A) `SYNTH_MEMORY_KEY` cleanup** (`interpreter/mod.rs:71-85`). When `current_state` is a real
   IR state (i.e. *not* a synthetic overlay id) **and** the overlay has no entry for it **and**
   `game_data.memories` contains the synthetic slot
   `crate::quantifier::SYNTH_MEMORY_KEY` (`"__quantifier_overlay_cards"`), the slot is removed.
   This guarantees the synthetic card-set memory a quantifier edge wrote just before dispatch is
   gone by the time the FSM is back on a real state, so a user `.cgdsl` program that later
   `CreateMemory`s the same key by coincidence is unaffected (invariant I-18). See
   [`invariants.md`](./invariants.md) I-18.

2. **(S) Ineligible-player skip** (I-24). If the current player is out of the
   game or out of the current stage (`current_player_ineligible`; no current stage → never
   ineligible) and the first outgoing edge's payload is *skippable*
   (`payload_is_skippable` — everything except `EndCondition`/`StageRoundCounter`/`EndStage`
   bookkeeping, `SetUp` rules, and `CycleAction`/`EndAction`), the edge is advanced through
   **without executing** (a `TraceEvent::Skipped` is emitted) and `step()` returns `Ok`. The
   check runs before the overlay dispatch and quantifier preprocessor, so quantifier prompt
   sites are skipped wholesale. Bookkeeping still processes normally — the stage loops back,
   the end-condition is re-evaluated (with the auto-end checks below), and the next eligible
   player (moved into place by the executed cycle action) takes over.

3. **(B) Overlay dispatch** (`interpreter/mod.rs:84-106`). If `current_state` has a synthetic
   replacement edge in `pending_overlay`, dispatch the first one through the normal `Action` trace
   + `execute_edge` path, then return `StepResult::Ok`. This arm fires once per fan-out edge for a
    `DestPlayerAll` quantifier (and for the per-player edges of an `All`-of-`Any`), emitting one
    `TraceEvent::Action` (carrying the cloned `GameRule`) per synthetic transition.

4. **(C0) Quantifier resume** (`interpreter/mod.rs:109-112` → `quant_driver.rs:15-101`). If a
   quantifier prompt is in flight (`pending_quant`) and its `state` equals `current_state` and an
   input has arrived matching the prompt kind, `take_quant_resume` consumes both and dispatches the
   chosen answer (see `resume_dest_player_any` at `quant_driver.rs:359-380`,
   `resume_cards_any_or_range` at `quant_driver.rs:384-423`, `resume_dest_all_then_cards` at
   `quant_driver.rs:425-479`). If `pending_quant.state != current_state`, the resume is *skipped*
   and the pending request + buffered input are left untouched — see invariant I-19. If the
   buffered input does **not** match the pending quantifier kind (e.g. a `Choice` arrives while a
   `CardsAnyOrRange` prompt is in flight), the stale input is popped from the buffer (preventing an
   infinite prompt loop) and the pending quantifier is left intact to receive the next input.

5. **Real IR edge lookup** (`interpreter/mod.rs:114-131`). If `current_state` is not in
   `ir.states` → `Error`. If `edges.is_empty()` → `GameOver` if at `ir.goal`, else `Error`.

6. **(C1) Quantifier preprocessor** (`interpreter/mod.rs:137-153` → `quantifier::scan_edge`).
   Before per-`Payload` dispatch, `step()` calls `crate::quantifier::scan_edge(edge)` and, if it
   returns a non-`None` `QuantSite`, hands off to the dedicated quantifier arm:
   - `QuantSite::DestPlayerAll { pc }` → `step_dest_player_all`
     (`interpreter/quant_driver.rs:155-210`) — fans out to every resolved player, or fires a single
     `ChooseCards` prompt first if the edge also carries an `All`-of-`Any` (in which case the
     fan-out is deferred to the resume branch `resume_dest_all_then_cards`).
   - `QuantSite::DestPlayerAny { pc }` → `step_dest_player_any`
     (`interpreter/quant_driver.rs:212-237`) — issues a `ChoosePlayer` prompt; the resume is
     `resume_dest_player_any` (`quant_driver.rs:407-429`).
   - `QuantSite::SourcePlayerAny { pc }` → `step_source_player_any`
     (`interpreter/quant_driver.rs:239-264`) — like `DestPlayerAny`, but the chosen player
     becomes the move's *source* owner (e.g. `deal Hand where Rank is "X" of any …` — "ask any
     player"); the resume is `resume_source_player_any` (`quant_driver.rs:431-454`).
   - `QuantSite::SrcCardsAnyOrRange { qty, from }` → `step_src_cards_any_or_range`
     (`interpreter/quant_driver.rs:265-294`) — issues a `ChooseCards` prompt; the resume is
     `resume_cards_any_or_range` (`quant_driver.rs:456-495`).
   - `QuantSite::None` → fall through to the per-`Payload` dispatch below.

   **Chaining.** Every resume arm ends in `quantify_or_dispatch`
   (`quant_driver.rs`): the substituted edge is re-scanned, and a *remaining*
   site (e.g. the card-amount `any` on `deal any from Hand of any …`) chains
   to its own prompt — player choice first, then cards. `step_dest_player_all`
   additionally resolves a chained `SourcePlayerAny` *before* fanning out, so
   the per-player replacement edges always carry a concrete owner.

6. **Setup-`Any` guard** (`interpreter/mod.rs:160-163`). For a `Payload::Action` whose `GameRule`
   is `SetUp { setup }`, `step()` checks `crate::quantifier::setup_contains_any(setup)`. If any
   element collection of the setup uses `Quantifier::Any`, it issues a
   `ChoosePlayer` prompt (`step_setup_any`); on resume the chosen player is
   substituted into every any-site of the setup (`substitute_setup_any`)
   *before* calling
   `execute_edge` — no `GameData` mutation occurs until the prompt is answered
   (invariant I-20). `Quantifier::All` in setup is
   supported and expands to all in-game players.

After the pre-dispatch arms, the per-`Payload` dispatch
(`interpreter/mod.rs:155-360`) of the **first** outgoing edge's payload:

- `Payload::Action(_)` → run the setup-`Any` guard (only for `SetUp`), then execute unconditionally
  and advance; emit `TraceEvent::Action { rule }` (the cloned `GameRule`).
- `Payload::Choice` → if the input buffer has an `Input`, pick `edges[input.idx()]` and execute;
  otherwise return `NeedsInput(InputType::Choice { … })`. Emits `TraceEvent::Choice`.
- `Payload::Optional` → same as `Choice` but with `InputType::Optional(prompt)`;
  `Input::OptionalAccept` → edge 0, `Input::OptionalDecline` → edge 1. Emits
  `TraceEvent::OptionalAccept`/`OptionalDecline`.
- `Payload::Condition { expr, negated }` → evaluate `expr`, pick edge 0 or 1 (inverted vs.
  `EndCondition` — see I-3). Emits `TraceEvent::Condition`.
- `Payload::EndCondition { expr, negated, stage }` → `ensure_stage_entered(stage)` (enters on first
  encounter), evaluate `expr`, pick edge 0 (exit) or 1 (continue); on exit, `leave_stage(stage)`
  pops the stage stack. (Edge indexing is inverted vs. `Condition` — see I-3.)
  **Auto-end (I-24):** the stage also exits when no player is in the game or no
  player is in this stage — the game then runs out to the goal with an empty winner set.
  Emits `TraceEvent::EndCondition`.
- `Payload::StageRoundCounter(stage)` → `ensure_stage_entered(stage)` (idempotent), increment
  counter, advance. Applied **once** (interpreter is the single mutator — see I-5).
  Emits `TraceEvent::StageRoundCounter`.
- `Payload::EndStage(stage)` → `leave_stage`, advance. Applied once. (Currently never emitted by
  the front_end IR builder; retained.) Emits `TraceEvent::EndStage`.
- `Payload::Trigger` → advance only (no mutation; `action::execute` catch-all swallows it).
  Emits `TraceEvent::Trigger`.

For each trace mention, see `TraceEvent` variant definitions in
[`observability.md`](./observability.md) §2.2.

### 4. Termination

- On `GameOver`: `run()` emits a final event and returns
  `Ok(self.interpreter.game_data.clone())` (`crates/engine/src/controller/mod.rs:281-282`) — a
  **full deep clone** of the terminal state. `GameOver` itself only fires when the current state
  has **no outgoing edges AND `current_state == ir.goal`** (I-4). The transition into `GameOver`
  emits a `TraceEvent::GameOver { winners }` (the winner set — every player still in game;
  empty when nobody won). If a trace log is open, `run_game_with` writes the
  `=== GameOver after N steps — winners: <names> ===` footer.
- On `Error(EngineError)`: `run()` returns `Err(EngineError)`
  (`crates/engine/src/controller/mod.rs:284`). The engine does **not** roll back mutations already
  applied before the error — see [`error-handling.md`](./error-handling.md). If a trace log is
  open, `run_game_with` writes `=== Error: <e> (after N steps) ===`.
- On **panic** during `run()`: `run_game`/`run_game_with`'s `catch_unwind` wrapper
  (`controller/mod.rs`) extracts the panic message and either (a) with
  `RunOptions::capture_panics(true)`: logs `=== Panic: <msg> (after N steps) ===` (if a trace log
  is open) and returns `Err(EngineError::InternalPanic { message })` — the process does not abort;
  or (b) with the legacy default: logs the panic (if a trace log is open) and `resume_unwind`s —
  the panic surfaces to the caller, just logged. Without a trace log and without `capture_panics`,
  panics propagate untouched.

### 5. Shutdown

The `crates::engine::controller::Controller` and `crates::engine::interpreter::Interpreter` are
dropped at the end of `run_game`; no explicit teardown is required. The test-input `std::fs::File`
handle is dropped at the end of `Controller::read_test_file`'s loading block
(`crates/engine/src/controller/mod.rs:330-343`). Drop order (`Controller` owns `Interpreter` owns
`GameData`) is standard Rust and no `Drop` impls exist in the crate — see
[`interfaces.md`](./interfaces.md) §6.3. If a trace log was opened, `TraceLogger` and its
underlying `BufWriter<File>` are dropped when `run_game` returns, flushing any remaining buffered
lines.
