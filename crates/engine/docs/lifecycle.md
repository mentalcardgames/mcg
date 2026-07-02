---
type: agent_wiki_node
module: crates::engine
scope: [engine::controller, engine::interpreter, engine::action, engine::game_data]
topics: [lifecycle, run-loop, initialization, sequencing, termination]
associated_files:
  - crates/engine/src/controller.rs
  - crates/engine/src/interpreter.rs
  - crates/engine/src/action.rs
  - crates/engine/src/game_data.rs
last_validated: 2026-07-02
---

# Lifecycle & Runtime Sequencing

This page describes *when* things happen: construction of the engine, the setup phase, the play
loop, and termination/shutdown. For the *shapes* of the values involved see
[`data-structures.md`](./data-structures.md); for rules that must not be broken during these phases
see [`invariants.md`](./invariants.md).

---

## The Run Loop (Orchestration)

`crates::engine::controller::Controller::run` (`crates/engine/src/controller.rs:62-79`) is the
single entry point and the entire main loop. It is a plain synchronous `loop { … }`:

```rust
// crates/engine/src/controller.rs:62-79  (paraphrased for clarity; see source for exact lines)
fn run(&mut self) -> Result<GameData, String> {
    loop {
        self.emit_event();                                   // optional event_sender callback
        match self.interpreter.step() {
            StepResult::Ok => continue,                      // advanced one edge; keep going
            StepResult::NeedsInput(input_type) => {           // stalled; ask InputSource
                let input = self.get_input(input_type)?;
                self.interpreter.provide_input(input);       // push onto LIFO buffer
            }
            StepResult::GameOver => {                         // terminal
                self.emit_event();                            //   one final event
                return Ok(self.interpreter.game_data.clone());//   deep-clone terminal state
            }
            StepResult::Error(e) => return Err(e),            // propagate error string
        }
    }
}
```

Key sequencing facts:

- `crates::engine::controller::Controller::emit_event` (`crates/engine/src/controller.rs:161-165`)
  is called at the **top of every iteration** *and* once more just before returning `GameOver`
  (`crates/engine/src/controller.rs:64,73`). See [`observability.md`](./observability.md).
- `crates::engine::interpreter::Interpreter::step` (`crates/engine/src/interpreter.rs:23`) performs
  **exactly one** FSM transition per call. It may synchronously call
  `crates::engine::query::Evaluator` (reads) and `crates::engine::action::execute` (writes).
- When `step()` returns `crates::engine::interpreter::StepResult::NeedsInput`, `run()` consults the
  `crates::engine::controller::InputSource` (`Player` closure or `TestFile`), obtains one
  `crates::engine::interpreter::Input`, and pushes it via
  `crates::engine::interpreter::Interpreter::provide_input`
  (`crates/engine/src/interpreter.rs:151-153`). The next `step()` pops it (LIFO — see I-7).
- `crates::engine::interpreter::Interpreter::execute_edge`
  (`crates/engine/src/interpreter.rs:145-148`) is the single mutator boundary: it sets
  `current_state = edge.to` and then calls
  `crates::engine::action::execute(edge.payload, &mut game_data)`.
- There is no scheduler, no green threads, no `Future`s; `run()` **blocks the calling thread** until
  `GameOver` or `Error`. See [`concurrency.md`](./concurrency.md).

---

## State Lifecycle

### 1. Construction

`crates::engine::game_data::GameData::new` (`crates/engine/src/game_data.rs:102-118`) produces an
empty store with `current_player = Some(0)`, empty `stage_stack`, empty `memories`.
`crates::engine::controller::run_game` (`crates/engine/src/controller.rs:24-46`) wraps it in an
`crates::engine::interpreter::Interpreter` (`current_state = ir.entry`, empty `input_buffer`) and a
`crates::engine::controller::Controller`.

Note: `GameData::new` sets `current_player = Some(0)` even though `turn_order` is empty — see
invariant I-2 in [`invariants.md`](./invariants.md).

### 2. Setup Phase

The IR's first edges carry
`front_end::ir::Payload::Action(front_end::ast::GameRule::SetUp { … })`. These run
`crates::engine::action::execute_setup_rule` (`crates/engine/src/action.rs:62-157`), which is the
**only** place `players`, `teams`, `turn_order`, `locations`, `cards`, `combos`, `precedences`,
`point_maps`, and `memories` are *created*. Order matters:

- `front_end::ast::SetUpRule::CreateLocation` resolves an owner by name
  (`crates/engine/src/action.rs:95-96`, `.expect("Failed to resolve owner to name")`), so
  `CreatePlayer`/`CreateTeams` must precede it.
- `front_end::ast::SetUpRule::CreateCardOnLocation`
  `.expect("Location not found")` (`crates/engine/src/action.rs:112`) requires the location to
  already exist.

### 3. Play Phase

Each `crates::engine::interpreter::Interpreter::step` either advances `current_state` via
`crates::engine::interpreter::Interpreter::execute_edge`
(`crates/engine/src/interpreter.rs:145-148`:
`current_state = edge.to; action::execute(edge.payload, &mut game_data)`), or yields
`NeedsInput`/`GameOver`/`Error`. Conditions and end-conditions are evaluated live against the
current `GameData` by `crates::engine::query::Evaluator`.

The `front_end::ir::Payload` of the current state's first outgoing edge determines the transition
kind dispatched inside `step()`:

- `Payload::Action(_)` → execute unconditionally, advance.
- `Payload::Choice` → if the input buffer has an `Input`, pick `edges[input.idx()]` and execute;
  otherwise return `NeedsInput(InputType::Choice { … })`.
- `Payload::Optional` → same as `Choice` but with `InputType::Optional(prompt)`;
  `Input::OptionalAccept` → edge 0, `Input::OptionalDecline` → edge 1.
- `Payload::Condition { expr, negated }` → evaluate `expr`, pick edge 0 or 1 (inverted vs.
  `EndCondition` — see I-3).
- `Payload::EndCondition { expr, negated, stage }` → evaluate, pick edge 0 (exit) or 1 (continue).
- `Payload::StageRoundCounter(stage)` → increment counter, advance (applied twice — see I-5).
- `Payload::EndStage(stage)` → `leave_stage`, advance (applied twice — see I-5).
- `Payload::Trigger` → advance only (no mutation; `action::execute` catch-all swallows it).

### 4. Termination

- On `GameOver`: `run()` emits a final event and returns
  `Ok(self.interpreter.game_data.clone())` (`crates/engine/src/controller.rs:72-75`) — a **full deep
  clone** of the terminal state. `GameOver` itself only fires when the current state has **no
  outgoing edges AND `current_state == ir.goal`** (I-4).
- On `Error(String)`: `run()` returns `Err(String)` (`crates/engine/src/controller.rs:76`). The
  engine does **not** roll back mutations already applied before the error — see
  [`error-handling.md`](./error-handling.md).

### 5. Shutdown

The `crates::engine::controller::Controller` and `crates::engine::interpreter::Interpreter` are
dropped at the end of `run_game`; no explicit teardown is required. The test-input `std::fs::File`
handle is dropped at the end of `Controller::read_test_file`'s loading block
(`crates/engine/src/controller.rs:117-129`). Drop order (`Controller` owns `Interpreter` owns
`GameData`) is standard Rust and no `Drop` impls exist in the crate — see
[`concurrency.md`](./concurrency.md) §"Resource Management".
