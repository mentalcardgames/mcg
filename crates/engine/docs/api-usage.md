---
type: agent_wiki_node
module: crates::engine
scope: [engine::controller, engine::interpreter, engine::query]
topics: [public-api, usage, extension-points, examples, integration]
associated_files:
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/interpreter/trace.rs
  - crates/engine/src/query/mod.rs
  - crates/engine/src/bin/cgdsl-play.rs
last_validated: 2026-07-04
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
optionally an event callback and/or a trace callback) → call
`crates::engine::controller::run_game`. The runnable example below mirrors
`crates/engine/src/bin/cgdsl-play.rs` but is written as an external consumer (`examples`-style)
and adds the event-sender and trace-sender hooks.

```rust
// In a downstream crate's example or test. Requires:
//   cgdsl-engine = { path = "../crates/engine" }
//   front_end    = { path = "../crates/front_end" }
use std::path::PathBuf;
use std::sync::Mutex;

use cgdsl_engine::{
    run_game, GameData, Input, InputSource, InputType, TraceEntry, DebugLevel, format_game_data,
};
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
    //        "<N>"  -> Choice { idx: N-1 }        (1-based choice index),
    //        "p <N>"-> ChoosePlayer { idx: N-1 }  (1-based candidate index),
    //        "c <csv>" -> ChooseCards { selected: [..] } (1-based, comma-separated).
    //        Blank/# lines are ignored.
    let input_source = InputSource::TestFile(PathBuf::from("path/to/inputs.txt"));

    //    (b) Interactive / programmatic: a closure. The engine hands you an
    //        InputType (Choice | Optional | ChoosePlayer | ChooseCards) and you
    //        return an Input. Must be Send + Sync.
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
            InputType::ChoosePlayer { candidates, prompt } => {
                println!("{}: {:?}", prompt, candidates);
                // ...return Input::ChoosePlayer { idx: 0..candidates.len()-1 }
                Input::ChoosePlayer { idx: 0 }
            }
            InputType::ChooseCards { display, min, max, prompt } => {
                println!("{}: {}..={} of {:?}", prompt, min, max, display);
                // ...return Input::ChooseCards { selected: vec![0] }
                Input::ChooseCards { selected: vec![0] }
            }
        }
    }
    let _interactive = InputSource::Player(Box::new(interactive_input));

    // 4. Optional event hook: invoked with &GameData after every loop iteration and
    //    once more at GameOver. Receives a shared reference — do NOT mutate through
    //    it. (Closure bound is `Fn(&GameData) + Send` — NOT Sync; see concurrency.md §2.)
    let event_sender: Option<Box<dyn Fn(&GameData) + Send>> = Some(Box::new({
        move |gd: &GameData| {
            // Render a frame, log a diff, or forward over a channel. For a snapshot
            // you must `clone()` — see observability.md §1.
            println!("{}", format_game_data(gd, DebugLevel::Medium));
        }
    }));

    // 5. Optional trace hook (post-Stage-5): invoked once per FSM *transition*
    //    (not per loop iteration) with a TraceEntry. This is the recommended
    //    structured-logging seam. Bound is `Fn(TraceEntry) + Send` (also NOT
    //    Sync). See observability.md §2.
    let trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>> = Some(Box::new({
        move |entry: TraceEntry| {
            // entry implements Display, so a one-line trace readout is trivial:
            println!("{}", entry);
        }
    }));

    // 6. Run to completion. Returns the terminal GameData (deep-cloned) or an
    //    error string. This call BLOCKS the current thread until GameOver/Error.
    //
    //    `run_game` is now a 5-arg call (added `trace_sender`). When
    //    MCG_TRACE_LOG is set, `run_game` composes the file logger with this
    //    closure so you do NOT need to duplicate the file logging yourself
    //    (see observability.md §3.1).
    match run_game(ir, game_data, input_source, event_sender, trace_sender) {
        Ok(final_state) => {
            println!("Game over. {} players still in game.",
                final_state.players.iter().filter(|p| p.in_game).count());
            cgdsl_engine::print_game_data(&final_state, cgdsl_engine::DebugLevel::High);
        }
        Err(e) => eprintln!("Game error: {e}"),
    }
}
```

A minimal replay test (no event hook, no trace hook, no `MCG_TRACE_LOG`) reduces to one line,
exactly as the in-tree test `crates/engine/src/controller/tests.rs:117` does:

```rust
let result = run_game(ir, GameData::new(), InputSource::TestFile(path), None, None);
assert!(result.is_ok());
```

> **Signature:** `pub fn run_game(ir, game_data, input_source, event_sender, trace_sender) -> Result<GameData, String>`
> at `crates/engine/src/controller/mod.rs:31-37`. The two `Option<Box<dyn Fn + Send>>` arguments
> may both be `None`, both be `Some`, or be mixed; `run_game` composes them with the optional
> `MCG_TRACE_LOG` file logger internally (see `controller/mod.rs:71-84` and
> [`observability.md`](./observability.md) §3.1).

---

## 2. Driving the Interpreter Manually (Advanced)

Hosts that need finer-grained control (e.g., to persist state between requests in a server) can
skip `crates::engine::controller::run_game` and drive an
`crates::engine::interpreter::Interpreter` directly. This is the contract
`crates::engine::controller::Controller` itself implements
(`crates/engine/src/controller/mod.rs:151-169`):

```rust
use cgdsl_engine::{Interpreter, StepResult, Input, InputType, GameData};
// `Interpreter::new` is the canonical constructor (seeds `current_state = ir.entry`,
// `next_synth = u32::MAX - 1`, empty `pending_overlay` / `pending_quant`). All fields
// remain `pub` so direct struct construction is *also* a supported pattern, but
// omitting the quantifier bookkeeping fields will misbehave on the first quantifier
// edge — prefer `new`. See data-structures.md §3.1.

let mut interp = Interpreter::new(
    ir,                  // Ir<LoweredPayLoad>
    GameData::new(),
    None,                // trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>
);

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

Note: manual driving does **not** get the `MCG_TRACE_LOG` file logging or the panic-capture
behavior — those live in `run_game` (`controller/mod.rs:38-134`). If you drive `Interpreter`
directly and want trace logging, pass a `trace_sender` to `Interpreter::new` and write the file
yourself, or call `run_game` instead.

---

## 3. Primary Traits & Extension Points

The engine deliberately exposes only **one** `pub` trait for downstream implementation:
`crates::engine::interpreter::IrExt` (`crates/engine/src/interpreter/ir_ext.rs:3-5`), and it is
already implemented for `Ir<LoweredPayLoad>` — downstream code consumes it, not implements it.
Extension is by composition, at four seams:

1. **`crates::engine::controller::InputSource::Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)`**
   (`crates/engine/src/controller/mod.rs:23`). The single seam for front-ends (GUI, CLI, networked
   bot). The closure receives the full `crates::engine::interpreter::InputType` and returns an
   `crates::engine::interpreter::Input`. Post-Stage-5 the `InputType` enum has **four** variants:
   - `Choice { options, max_index }` — pick an edge;
   - `Optional(prompt)` — accept/decline;
   - `ChoosePlayer { candidates, prompt }` — pick one player by 0-based index into `candidates`
     (issued by `DestPlayerAny` quantifier sites);
   - `ChooseCards { display, min, max, prompt }` — pick a subset of `display` cards with size in
     `[min, max]` (issued by `SrcCardsAnyOrRange` and `DestPlayerAll`-of-`Any` quantifier sites).
   The closure may block arbitrarily (e.g., on user input) — the engine waits. The controller
   validates the answer (`controller/mod.rs:302-319`) and re-invokes the closure on out-of-range
   answers — see I-8/I-15 in [`invariants.md`](./invariants.md).

2. **`event_sender: Option<Box<dyn Fn(&GameData) + Send>>`**
   (`crates/engine/src/controller/mod.rs:35`, `crates::engine::controller::run_game`'s 4th arg).
   Coarse-grained reactive observability: called with a snapshot of `GameData` after every loop
   iteration and once more immediately before `GameOver` return
   (`crates/engine/src/controller/mod.rs:153,163`). `Send` but not `Sync` — see
   [`concurrency.md`](./concurrency.md) §2. Typical use: render a GUI frame, log a diff, or forward
   over a channel. (See [`observability.md`](./observability.md) §1.)

3. **`trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>`** (post-Stage-5)
   (`crates/engine/src/controller/mod.rs:36`, `crates::engine::controller::run_game`'s 5th arg).
   Fine-grained reactive observability: called once per FSM *transition* (not per loop iteration)
   with a `crates::engine::interpreter::TraceEntry`. `Send` but not `Sync` (so `Interpreter` is
   now `Send` but not `Sync` — see [`concurrency.md`](./concurrency.md) §2). When `MCG_TRACE_LOG`
   is also set, `run_game` composes this closure with the file logger
   (`crates/engine/src/controller/mod.rs:71-84`) — hosts do NOT need to duplicate the file logging.
   (See [`observability.md`](./observability.md) §2–§3.)

4. **The DSL itself.** Because the engine is a pure interpreter over `front_end`'s IR, the primary
   "extension" for new game mechanics is authoring `.cgdsl` (which `front_end` lowers into
   `front_end::ast::GameRule`/`front_end::ast::SetUpRule`/`front_end::ast::ActionRule` variants).
   The engine's `crates/engine/src/action.rs`/`crates/engine/src/query/` must already implement
   the corresponding variant — variants with `// TODO` (see
   [`error-handling.md`](./error-handling.md) §2 "Silent no-ops") are no-ops. Quantifier-bearing
   edges (`Quantifier::All`/`Any` over a dest `PlayerCollection`, or `Any`/`IntRange` `Quantity`)
   are intercepted by `crates/engine/src/quantifier.rs` and rewritten into concrete replacement
   edges — see [`lifecycle.md`](./lifecycle.md) §3 "Play Phase" pre-dispatch arms.

`crates::engine::query::Evaluator`'s `pub` methods (`eval_bool`, `eval_int`, `eval_string`,
`eval_player`, `eval_team`, `eval_cardset`, `eval_card_position`, `resolve_quantity`,
`expand_types`, `resolve_owner_to_name`/`resolve_owner_to_names`, …) are also available for hosts
that want to query a `crates::engine::game_data::GameData` outside of a running game (e.g., to
render a derived statistic). All take `&GameData` and return `Result<T, String>` (or `Vec<usize>`
for resolvers).
