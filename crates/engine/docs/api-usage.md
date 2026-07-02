---
type: agent_wiki_node
module: crates::engine
scope: [engine::controller, engine::interpreter, engine::query]
topics: [public-api, usage, extension-points, examples, integration]
associated_files:
  - crates/engine/src/controller.rs
  - crates/engine/src/interpreter.rs
  - crates/engine/src/query.rs
  - crates/engine/src/bin/cgdsl-play.rs
last_validated: 2026-07-02
---

# Public API & End-to-End Usage Manual

This page is for **external consumers** of `cgdsl-engine`. It covers the golden path through
`crates::engine::controller::run_game`, the lower-level manual driving of
`crates::engine::interpreter::Interpreter`, and the extension seams the engine exposes. For the
shapes of the values passed here, see [`data-structures.md`](./data-structures.md); for what can go
wrong, see [`error-handling.md`](./error-handling.md).

---

## 1. The Golden Path

The canonical end-to-end flow is: parse `.cgdsl` → lower to
`front_end::ir::Ir<front_end::ir::LoweredPayLoad>` → construct an empty
`crates::engine::game_data::GameData` → supply an `crates::engine::controller::InputSource` (and
optionally an event callback) → call `crates::engine::controller::run_game`. The runnable example
below mirrors `crates/engine/src/bin/cgdsl-play.rs` but is written as an external consumer
(`examples`-style) and adds the event-sender hook.

```rust
// In a downstream crate's example or test. Requires:
//   cgdsl-engine = { path = "../crates/engine" }
//   front_end    = { path = "../crates/front_end" }
use std::path::PathBuf;
use std::sync::Mutex;

use cgdsl_engine::{run_game, GameData, Input, InputSource, InputType};
use front_end::validation::parse_document;

fn main() {
    // 1. Load + parse + lower the DSL source into the FSM the engine consumes.
    let source = std::fs::read_to_string("path/to/game.cgdsl")
        .expect("failed to read game file");
    let game = parse_document(&source).expect("failed to parse .cgdsl");
    let ir = game.to_lowered_graph(); // -> Ir<LoweredPayLoad>

    // 2. Fresh, empty game state. `current_player` starts at Some(0) (see I-2).
    let game_data = GameData::new();

    // 3. Choose an input source.
    //
    //    (a) Recorded/replayable: a text file with one line per input request.
    //        Lines: "y"/"yes" -> OptionalAccept, "n"/"no" -> OptionalDecline,
    //        "<N>" -> Choice { idx: N-1 } (1-based). Blank/# lines are ignored.
    let input_source = InputSource::TestFile(PathBuf::from("path/to/inputs.txt"));

    //    (b) Interactive / programmatic: a closure. The engine hands you an
    //        InputType (Choice { options, max_index } | Optional(prompt)) and
    //        you return an Input. Must be Send + Sync.
    #[allow(dead_code)]
    fn interactive_input(input_type: InputType) -> Input {
        match input_type {
            InputType::Choice { options, max_index } => {
                println!("Choose: {:?}", options);
                // ...read stdin, validate 0..=max_index, return Input::Choice { idx }
                Input::Choice { idx: 0 }
            }
            InputType::Optional(prompt) => {
                println!("{}", prompt);
                Input::OptionalAccept
            }
        }
    }
    let _interactive = InputSource::Player(Box::new(interactive_input));

    // 4. Optional event hook: invoked with &GameData after every step and once
    //    more at GameOver. Receives a shared reference — do NOT mutate through it.
    //    (Closure bound is `Fn(&GameData) + Send` — NOT Sync; see concurrency.md §2.)
    let snapshots: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let event_sender: Option<Box<dyn Fn(&GameData) + Send>> = Some(Box::new({
        let snapshots = snapshots.lock().unwrap(); // capture by move is awkward; shown illustratively
        move |gd: &GameData| {
            let s = cgdsl_engine::format_game_data(gd, cgdsl_engine::DebugLevel::Medium);
            // snapshots.push(s) would require moving the Mutex in; in real code
            // wrap in Arc<Mutex<_>> or send over a channel to another thread.
            println!("{}", s);
        }
    }));

    // 5. Run to completion. Returns the terminal GameData (deep-cloned) or an
    //    error string. This call BLOCKS the current thread until GameOver/Error.
    match run_game(ir, game_data, input_source, None) {
        Ok(final_state) => {
            println!("Game over. {} players still in game.",
                final_state.players.iter().filter(|p| p.in_game).count());
            cgdsl_engine::print_game_data(&final_state, cgdsl_engine::DebugLevel::High);
        }
        Err(e) => eprintln!("Game error: {e}"),
    }
}
```

A minimal replay test (no event hook, no I/O) reduces to one line, exactly as the in-tree test
`crates/engine/src/controller.rs:264` does:

```rust
let result = run_game(ir, GameData::new(), InputSource::TestFile(path), None);
assert!(result.is_ok());
```

---

## 2. Driving the Interpreter Manually (Advanced)

Hosts that need finer-grained control (e.g., to persist state between requests in a server) can
skip `crates::engine::controller::run_game` and drive an
`crates::engine::interpreter::Interpreter` directly. This is the contract
`crates::engine::controller::Controller` itself implements
(`crates/engine/src/controller.rs:62-78`):

```rust
use cgdsl_engine::{Interpreter, StepResult, Input, InputType, GameData};
// (Interpreter fields are pub, so this is a supported usage pattern.)

let mut interp = Interpreter {
    ir,                                   // Ir<LoweredPayLoad>
    game_data: GameData::new(),
    input_buffer: Vec::new(),
    current_state: ir.entry,              // start at the FSM entry
};

loop {
    match interp.step() {
        StepResult::Ok => {}                                       // advanced one edge
        StepResult::NeedsInput(it: InputType) => {                 // stalled; ask your UI…
            let input: Input = my_ui_resolve(it);                  // …then push and continue
            interp.provide_input(input);
        }
        StepResult::GameOver => break,                             // terminal
        StepResult::Error(msg) => { eprintln!("{msg}"); break; }  // see error-handling.md §2
    }
}
// interp.game_data is the final state (no extra clone is performed here).
```

---

## 3. Primary Traits & Extension Points

The engine deliberately exposes **no traits** for downstream implementation. Extension is by
composition, at three seams:

1. **`crates::engine::controller::InputSource::Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)`**
   (`crates/engine/src/controller.rs:16`). The single seam for front-ends (GUI, CLI, networked
   bot). The closure receives the full `crates::engine::interpreter::InputType` (choice options +
   `max_index`, or an optional prompt) and returns an `crates::engine::interpreter::Input`. It may
   block arbitrarily (e.g., on user input) — the engine waits. Validation is the closure's
   responsibility for non-`Choice` paths; for `Choice`, the controller re-invokes on
   `idx > max_index` (see I-8/I-15 in [`invariants.md`](./invariants.md)).

2. **`event_sender: Option<Box<dyn Fn(&GameData) + Send>>`**
   (`crates/engine/src/controller.rs:52`, `crates::engine::controller::run_game`'s 4th arg).
   Reactive observability: called with a snapshot of `GameData` after every successful
   `crates::engine::interpreter::Interpreter::step` and once more immediately before `GameOver`
   return (`crates/engine/src/controller.rs:64,73`). `Send` but not `Sync` — see
   [`concurrency.md`](./concurrency.md) §2. Typical use: render a GUI frame, log a diff, or forward
   over a channel. (See [`observability.md`](./observability.md).)

3. **The DSL itself.** Because the engine is a pure interpreter over `front_end`'s IR, the primary
   "extension" for new game mechanics is authoring `.cgdsl` (which `front_end` lowers into
   `front_end::ast::GameRule`/`front_end::ast::SetUpRule`/`front_end::ast::ActionRule` variants).
   The engine's `crates/engine/src/action.rs`/`crates/engine/src/query.rs` must already implement
   the corresponding variant — variants with `// TODO` (see
   [`error-handling.md`](./error-handling.md) §2 "Silent no-ops") are no-ops.

`crates::engine::query::Evaluator`'s `pub` methods (`eval_bool`, `eval_int`, `eval_string`,
`eval_player`, `eval_team`, `eval_cardset`, `eval_card_position`, `resolve_quantity`,
`expand_types`, …) are also available for hosts that want to query a
`crates::engine::game_data::GameData` outside of a running game (e.g., to render a derived
statistic). All take `&GameData` and return `Result<T, String>` (or `Vec<usize>` for resolvers).
