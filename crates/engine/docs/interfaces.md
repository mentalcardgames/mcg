---
type: agent_wiki_node
module: crates::engine
scope: [public API — all modules]
topics: [public-api, data-flow, ui-interface, observability, integration, controller-contract, threading, worked-examples]
associated_files:
  - crates/engine/src/lib.rs
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/interpreter/trace.rs
  - crates/engine/src/interpreter/types.rs
  - crates/engine/src/game_data.rs
last_validated: 2026-08-09
---

# Public Interfaces — the External Host Contract

> **Audience:** the project lead starting the greenfield external-UI project shortly. This is the
> **integration hub**: a reader who finishes this page should be able to write a working host
> (CLI, GUI, or network relay) from scratch. Every other wiki page is a spoke linked from here —
> this page inventories the public API surface and links out for field-level detail.
> It also absorbs the former `api-usage.md` (worked examples, §7) and `concurrency.md`
> (threading/`Send`/`Sync`, §6).

The engine's public API surface is the set of symbols re-exported at the crate root by
`crates/engine/src/lib.rs:9-13`. `Controller` itself is `pub(crate)` — an external host never
constructs it. Hosts integrate either by handing a closure to `run_game` (Mode A) or by owning an
`Interpreter` and driving `step()` themselves (Mode B). Both modes are documented in §4.

---

## §1. Public API Inventory

This section inventories every symbol re-exported by `crates/engine/src/lib.rs:9-13`:

```rust
// crates/engine/src/lib.rs:9-13
pub use controller::{run_game, InputSource};
pub use debug::{format_game_data, save_game_data, DebugLevel};
pub use game_data::{Card, Combo, GameData, Location, OwnerData, Player, PointMap, Precedence};
pub use interpreter::{Input, InputType, Interpreter, IrExt, StepResult, TraceEntry, TraceEvent};
pub use quantifier::{PendingKind, PendingQuant, QuantSite};
```

Grouped by concern below. Signatures are re-resolved verbatim from the post-refactor source;
field-level layouts live in [`data-structures.md`](./data-structures.md) §1–§3.

### §1.1 Construction & driving

**`cgdsl_engine::run_game`** — drives the FSM to completion (Mode A entry point).

```rust
// crates/engine/src/controller/mod.rs:31
pub fn run_game(
    ir: Ir<LoweredPayLoad>,
    game_data: GameData,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
) -> Result<GameData, String>
```

Intent: blocks the calling thread, drives the FSM to `GameOver`/`Error`, returns a deep clone of
the terminal state. The two `Option<Box<dyn Fn + Send>>` args compose with the optional
`MCG_TRACE_LOG` file logger internally — see [`observability.md`](./observability.md) §3.1. For the
run-loop sequencing see [`lifecycle.md`](./lifecycle.md); for thread-safety bounds see
�6 §2. Invariants: I-2 (empty-state sentinel), I-4 (GameOver
condition), I-15 (validation-loop spin), I-16 (synthetic-id seed). Error strings: see
[`error-handling.md`](./error-handling.md) §2.

**`cgdsl_engine::InputSource`** — the single seam for external I/O supplied to `run_game`.

```rust
// crates/engine/src/controller/mod.rs:22
pub enum InputSource {
    Player(Box<dyn Fn(InputType) -> Input + Send + Sync>),
    TestFile(PathBuf),
}
```

Intent: `Player` is the closure a UI/CLI host supplies (the primary integration point); `TestFile`
is the recorded-replay path for test-suite writers. The closure bound is `Send + Sync` (stronger
than the `Send`-only `event_sender`/`trace_sender` — see �6 §2).
The controller validates closure answers and re-prompts on violation (I-8, I-15); the `TestFile`
path errors on exhaustion rather than re-prompting — see §4.2.

**`cgdsl_engine::Interpreter`** — the running FSM state a host owns in Mode B.

```rust
// crates/engine/src/interpreter/mod.rs:26
pub struct Interpreter {
    pub ir: Ir<LoweredPayLoad>,
    pub game_data: GameData,
    pub input_buffer: Vec<Input>,
    pub current_state: StateID,
    pub trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
    pub pending_overlay: std::collections::HashMap<StateID, Vec<Edge<LoweredPayLoad>>>,
    pub next_synth: u32,
    pub pending_quant: Option<crate::quantifier::PendingQuant>,
}
```

Intent: all fields are `pub` so direct struct construction is supported, but `Interpreter::new`
(initializes `current_state = ir.entry`, `next_synth = u32::MAX - 1`, empty `pending_overlay` /
`pending_quant`) is preferred — omitting the quantifier bookkeeping fields misbehaves on the first
quantifier edge. Field-level commentary lives in [`data-structures.md`](./data-structures.md) §3.1;
quantifier overlay/synthetic-id invariants are I-16, I-17, I-18, I-19.

**`Interpreter::new`** — the canonical constructor.

```rust
// crates/engine/src/interpreter/mod.rs:46
pub fn new(
    ir: Ir<LoweredPayLoad>,
    game_data: GameData,
    trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
) -> Self
```

Intent: 3-arg construction. The Mode B skeleton in §4.6 calls it with exactly these three.

**`Interpreter::step`** — one FSM transition (the Mode B driver).

```rust
// crates/engine/src/interpreter/mod.rs:64
pub fn step(&mut self) -> StepResult
```

Intent: dispatches one edge (after quantifier pre-dispatch); returns `Ok`/`NeedsInput`/`GameOver`/
`Error`. The controller's `run_game` calls this in a loop; a Mode B host calls it directly.
Loop sequencing: [`lifecycle.md`](./lifecycle.md) §3.

**`Interpreter::provide_input`** — pushes one `Input` onto the LIFO input buffer.

```rust
// crates/engine/src/interpreter/mod.rs:372
pub fn provide_input(&mut self, input: Input)
```

Intent: the only way a Mode B host answers a `NeedsInput`. Note I-7 — the buffer is a LIFO stack,
not a queue; the `validate_int_range` re-prompt path can re-push (see I-19).

**`Interpreter::execute_edge`** — direct edge dispatch (advanced).

```rust
// crates/engine/src/interpreter/mod.rs
pub fn execute_edge(&mut self, edge: Edge<LoweredPayLoad>) -> Result<(), String>
```

Intent: sets `current_state = edge.to` and calls
`crate::action::execute(edge.payload, &mut self.game_data)`. Fallible since
2026-08: action-evaluation failures (e.g. `cycle to next` with no eligible
*other* player) surface as `Err(String)` instead of panicking. Exposed for
quantifier-driver resume and tests; a UI host normally never calls this.

**`cgdsl_engine::IrExt`** — the single `pub` trait (consumed, not implemented, by hosts).

```rust
// crates/engine/src/interpreter/ir_ext.rs:3
pub trait IrExt {
    fn edge_labels(&self, state: StateID) -> Vec<String>;
}
```

Intent: already implemented for `Ir<LoweredPayLoad>`; downstream code calls `ir.edge_labels(state)`.
Extension is by composition, not implementation — see §4.5.

### §1.2 Input contract

Full verbatim enum bodies (the contract a UI host dispatches on):

```rust
// crates/engine/src/interpreter/types.rs:2
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
```

```rust
// crates/engine/src/interpreter/types.rs:58
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
```

```rust
// crates/engine/src/interpreter/types.rs:50
pub enum StepResult {
    Ok,
    NeedsInput(InputType),
    GameOver,
    Error(String),
}
```

Intent: `InputType` is what the engine *asks*; `Input` is what the host *answers*; `StepResult` is
the per-transition outcome a Mode B host matches on. `Input::ChooseCards.selected` are indices
INTO `InputType::ChooseCards.display` (not card ids) — the most error-prone part of this contract
(see pitfall §4.2). `InputType::ChooseCards.display` is a `Vec<Card>` (the cards themselves), a
deliberate convenience so a UI can render them without a lookup. Validation rules + the matching
pairs are enumerated in §4.2.

### §1.3 State

`cgdsl_engine::GameData` is the single mutable aggregate; `Card`/`Player`/`Location`/`OwnerData`/
`Combo`/`Precedence`/`PointMap` are its field types. Full field-level layouts live in
[`data-structures.md`](./data-structures.md) §1 (and the `MemoryValue` enum, `Team`, etc.). For
the index-indirection / turn-order / stage-stack conventions see invariants I-1, I-2, I-11, I-13
in [`invariants.md`](./invariants.md).

```rust
// crates/engine/src/game_data.rs:22
pub type Card = HashMap<String, String>;
// crates/engine/src/game_data.rs:25
pub struct GameData { /* see data-structures.md §1 */ }
// crates/engine/src/game_data.rs:54
pub struct OwnerData { pub locations: Vec<usize> /* later: memories */ }
// crates/engine/src/game_data.rs:60
pub struct Player { /* name, score, owner, in_game, in_stage */ }
// crates/engine/src/game_data.rs:69
pub struct Location { pub name: String, pub cards: Vec<usize> }
// crates/engine/src/game_data.rs:81
pub struct Combo { pub name: String, pub filter: FilterExpr }
// crates/engine/src/game_data.rs:88
pub struct Precedence { pub name: String, pub key: String, pub values: Vec<String> }
// crates/engine/src/game_data.rs:96
pub struct PointMap { pub name: String, pub map: HashMap<String, i32> }
```

`GameData::new()` is the empty-state constructor (sets `current_player = Some(0)` sentinel; see I-2).

### §1.4 Read-side queries — `Evaluator`

`Evaluator` is **not re-exported at the crate root**; import it as `cgdsl_engine::query::Evaluator`
(`crates/engine/src/query/mod.rs:173`, a zero-sized struct used as a namespace for stateless
read-only associated functions). It is included here because the plan groups re-exported symbols
by concern, but hosts wanting on-demand derived stats outside a running game reach for these
methods (cross-reference: full method list in [`data-structures.md`](./data-structures.md) §3.5).

All `eval_*`/`resolve_*` methods take `&GameData` and return `Result<_, String>` (or `Vec<usize>`
for the resolvers). The `pub` fns (verbatim from source):

```text
// crates/engine/src/query/bool.rs:6,30,100,154,165
eval_bool, eval_aggregate, eval_compare, eval_int_compare, eval_end_condition
// crates/engine/src/query/int.rs:9,285
eval_int, resolve_quantity
// crates/engine/src/query/string.rs:8
eval_string
// crates/engine/src/query/player.rs:9,175,207,227,286,301
eval_player, eval_team, resolve_players, resolve_player_collection, resolve_owner_to_name, resolve_owner_to_names
// crates/engine/src/query/cardset.rs:9,538
eval_cardset, eval_card_position
```

Error strings (division by zero, missing memory/location/precedence/pointmap/combo, type-mismatched
memory, out-of-range index, "no current player/stage", …) are catalogued in
[`error-handling.md`](./error-handling.md) §1.

### §1.5 Trace / observability

Full verbatim enum bodies:

```rust
// crates/engine/src/interpreter/trace.rs:2
pub enum TraceEntry {
    Step {
        from: u32,
        to: u32,
        event: TraceEvent,
    },
}
```

```rust
// crates/engine/src/interpreter/trace.rs:10-52
pub enum TraceEvent {
    Action { subtype: String, detail: String, raw_detail: String },
    Choice { chosen_idx: usize, options: Vec<String> },
    OptionalAccept,
    OptionalDecline,
    Condition { expr: String, raw_expr: String, result: bool, negated: bool, took_else: bool },
    EndCondition { expr: String, raw_expr: String, result: bool, stage: String, exited: bool },
    StageRoundCounter { stage: String, new_count: u32 },
    EndStage { stage: String },
    Trigger,
    Quantifier { kind: String, detail: String },
}
```

`detail`/`expr` are DSL-level text (via the AST `Display` impls);
`raw_detail`/`raw_expr` are the `Debug` forms. `TraceEvent::pretty()` /
`TraceEvent::raw()` select between them (the TUI's `r` toggle uses this).

Intent: `TraceEntry::Step { from, to, event }` is emitted once per FSM transition via the
`trace_sender` callback; the `Quantifier` variant surfaces intermediate sub-steps ("chose player
X") that don't transition state. `TraceEntry` implements `Display` (one-line trace readout). Deep
dive: [`observability.md`](./observability.md) §2.

### §1.6 Diagnostics — `debug`

```rust
// crates/engine/src/debug/mod.rs:13
pub enum DebugLevel { Low, Medium, High }

// crates/engine/src/debug/mod.rs:38
pub fn format_game_data(data: &GameData, level: DebugLevel) -> String
// crates/engine/src/debug/save.rs:8
pub fn save_game_data(data: &GameData, path: &Path) -> io::Result<()>
```

Intent: formatted human-readable dumps (`Low`/`Medium`/`High` verbosity); `save_game_data` appends
to `path` (auto-selecting the level from a `<!--LEVEL-->` first-line marker). Deep dive:
[`observability.md`](./observability.md) §4.

### §1.7 Quantifier preprocessor (advanced host use only)

For understanding synthetic `StateID`s / pending prompts emitted by the quantifier subsystem. A UI
host does **not** normally construct these; they appear in `Interpreter::pending_quant` and
`pending_overlay` between `step()` calls when a quantifier edge awaits input.

```rust
// crates/engine/src/quantifier.rs:46
pub enum QuantSite {
    None,
    DestPlayerAll { pc: PlayerCollection },
    DestPlayerAny { pc: PlayerCollection },
    SrcCardsAnyOrRange { qty: Quantity, from: CardSet },
    ComboSource { combo: String, from: CardSet },  // lay-down: prompt + validate
}

// crates/engine/src/quantifier.rs:69
pub struct PendingQuant {
    pub state: StateID,
    pub kind: PendingKind,
}

// crates/engine/src/quantifier.rs:78
pub enum PendingKind {
    DestPlayerAny { candidates: Vec<String>, original: Edge<LoweredPayLoad> },
    CardsAnyOrRange { candidate_ids: Vec<usize>, original: Edge<LoweredPayLoad> },
    DestAllThenCards { player_names: Vec<String>, candidate_ids: Vec<usize>, original: Edge<LoweredPayLoad> },
    Combo { candidate_ids: Vec<usize>, filter: FilterExpr, original: Edge<LoweredPayLoad> },
}
```

Intent: `QuantSite` classifies a quantifier-bearing edge (used by
`crates/engine/src/quantifier.rs`'s `scan_edge`); `PendingQuant` is the state carried across a
`NeedsInput` round-trip. Invariants: I-16, I-17, I-19, I-20. Deep dive:
[`lifecycle.md`](./lifecycle.md) §3 "Play Phase" pre-dispatch arms.

---

## §2. Data Flow IN

What a host supplies to the engine. Pipeline (parse → lower → run):

```
 .cgdsl source
      │ front_end::validation::parse_document(&source)   (externally owned; the engine never constructs it)
      ▼
 front_end::ast::SGame
      │ SGame::to_lowered_graph()
      ▼
 front_end::ir::Ir<LoweredPayLoad>   ──┐
                                       │      GameData::new()   ──┐   (current_player = Some(0) sentinel; I-2)
                                       │                            │
                                       ▼                            ▼
                            run_game(ir, game_data, input_source, event_sender, trace_sender)
                                       │        or Mode B: Interpreter::new(ir, game_data, trace_sender)
                                       ▼
                                drives FSM to completion
```

The five inputs a host supplies:

1. **The lowered FSM** — `Ir<LoweredPayLoad>` from
   `front_end::validation::parse_document(&source)` then `SGame::to_lowered_graph()`. Externally
   owned; the engine never constructs it (see [`README.md`](./README.md) "High-Level Purpose").
2. **Empty state** — `GameData::new()` (`current_player = Some(0)` sentinel; see I-2).
3. **Input source** — `InputSource::Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)` (the
   closure seam; primary integration point for UI hosts) or `InputSource::TestFile(PathBuf)`
   (recorded replay).
4. **Optional event callback** — `Option<Box<dyn Fn(&GameData) + Send>>`. Rate: once per loop
   iteration; payload: `&GameData` snapshot (clone it to retain — see [`observability.md`](./observability.md) §1).
5. **Optional trace callback** — `Option<Box<dyn Fn(TraceEntry) + Send>>`. Rate: once per FSM
   transition; payload: `TraceEntry`.

### `InputSource::TestFile` line-format protocol

For test-suite writers. The on-disk replay format (`crates/engine/src/controller/mod.rs:200-275`):

| Line | Maps to |
|---|---|
| `y`, `yes` | `Input { player_id: "P1", kind: InputKind::OptionalAccept }` |
| `n`, `no` | `Input { player_id: "P1", kind: InputKind::OptionalDecline }` |
| `<N>` | `Input { player_id: "P1", kind: InputKind::Choice { idx: N-1 } }` (1-based) |
| `p <N>` | `Input { player_id: "P1", kind: InputKind::ChoosePlayer { idx: N-1 } }` (1-based candidate index) |
| `c <csv>` | `Input { player_id: "P1", kind: InputKind::ChooseCards { selected: <0-based> } }` — input is 1-based, internally converted; comma-separated |
| `Name:y` / `Name:<N>` / etc. | Same as above, with `player_id: "Name"` (defaults to `"P1"` when no prefix) |

Blank lines and `#…` comment lines are ignored (`controller/mod.rs:211-216`). On exhaustion the
path returns the error `"Test input file exhausted (input #N)"` (`controller/mod.rs:223`) rather
than re-prompting. Other parse errors are catalogued in [`error-handling.md`](./error-handling.md) §1.

---

## §3. Data Flow OUT

What a host receives:

- **`Ok(GameData)`** — a deep clone of the terminal state. The clone is O(total state); see
  �6 §3 and the Mode B note in §6.
- **`Err(String)`** — engine-level error; the engine does **not** roll back mutations already
  applied (see [`error-handling.md`](./error-handling.md) §2).
- **During run: per-iteration `&GameData` snapshots** — via `event_sender` (see §5).
- **During run: per-step `TraceEntry` events** — via `trace_sender` (see §5).
- **On-demand: `Evaluator::eval_*` queries** — for rendering derived stats outside a running game
  (e.g. resolve a player by name, evaluate an int expression for a HUD gauge). See §1.4 and
  [`data-structures.md`](./data-structures.md) §3.5.

---

## §4. The Controller / UI Interface (PRIMARY section for the upcoming UI project)

This is the longest and most important section. It defines the turn contract a UI host implements.

### §4.1 Two integration modes

- **Mode A — high-level `run_game`.** Hand the closure; the engine blocks until `GameOver`.
  Simplest; the engine manages state. Suited for CLI/scripts/replay tools. The downside: no
  mid-loop control; the closure is the only seam.
- **Mode B — low-level `Interpreter::step` loop (recommended for the UI project).** The host owns
  the `Interpreter`, calls `step()` repeatedly, matches `StepResult`, renders on each event, and
  supplies `Input` via `provide_input`. This is the mode for any non-trivial GUI — the upcoming
  project should use Mode B.

**Mode A skeleton:**

```rust
use cgdsl_engine::{run_game, GameData, Input, InputSource, InputType};
use front_end::validation::parse_document;

let game = parse_document(&source).expect("parse");
let ir = game.to_lowered_graph();
let input_source = InputSource::Player(Box::new(move |it: InputType| -> Input {
    // resolve_input dispatcher — see §4.2 for each InputType arm's validation rule
    my_ui_resolve(it)
}));
let _final = run_game(ir, GameData::new(), input_source, None, None);
```

**Mode B skeleton (the recommended path for the UI project):**

```rust
use cgdsl_engine::{Interpreter, StepResult, Input, InputType, GameData};
use front_end::validation::parse_document;

let game = parse_document(&source).expect("parse");
let ir = game.to_lowered_graph();
let mut interp = Interpreter::new(
    ir,
    GameData::new(),
    None, // trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>
);
loop {
    match interp.step() {
        StepResult::Ok => ui.render(&interp.game_data),
        StepResult::NeedsInput(input_type) => {
            let input = ui.resolve_input(input_type);   // blocks on user gesture; see §4.2 rules
            interp.provide_input(input);
        }
        StepResult::GameOver => { ui.show_final(&interp.game_data); break; }
        StepResult::Error(e) => { ui.show_error(&e); break; }
    }
}
// interp.game_data is the final state — no extra clone (compare Mode A's terminal clone).
```

### §4.2 The request/response turn contract

For each `StepResult::NeedsInput(InputType::X)` variant, the matching `Input::Y` the host must
return via `provide_input`, and the validation the controller enforces on `Player`-sourced answers
(`crates/engine/src/controller/mod.rs:302-319`). The controller re-prompts the closure on violation
(I-8, I-15 — spin risk for buggy closures). The `TestFile` path errors on exhaustion (`§2`) rather
than re-prompting.

| `InputType` request | Matching `Input` answer | Validation rule |
|---|---|---|
| `Choice { options, max_index }` | `Input { player_id, kind: InputKind::Choice { idx } }` | `idx <= max_index` |
| `Optional(prompt)` | `Input { player_id, kind: InputKind::OptionalAccept }` \| `InputKind::OptionalDecline` | none — either variant is accepted |
| `ChoosePlayer { candidates, prompt }` | `Input { player_id, kind: InputKind::ChoosePlayer { idx } }` | `idx < candidates.len()` |
| `ChooseCards { display, min, max, prompt }` | `Input::ChooseCards { selected }` | every `i` in `selected` is `< display.len()` AND `selected.len() >= min` AND `<= max` |

> **Pitfall (most error-prone).** `Input::ChooseCards.selected` are indices **INTO `display`**, not
> card ids. A UI that renders `display` as a list should map the user's selected row → the row's
> index, not the card's `cards`-vec id. `InputType::ChooseCards.display` is a `Vec<Card>` (the cards
> themselves, not ids) precisely so the UI can render them directly without a lookup.

> **LIFO buffer caveat (I-7).** `provide_input` *pushes* onto a LIFO stack, not a queue. The
> `validate_int_range` re-prompt path can re-push (see I-19); a host should answer exactly one
> `Input` per `NeedsInput` and not pipeline ahead.

### §4.3 Closing the loop: how the UI knows when to stop

- **`StepResult::GameOver`** — terminal; `interp.game_data` is the final state (no clone in Mode B).
- **`StepResult::Error(String)`** — surface to the UI; the engine may be in a partially-mutated
  state (no rollback — see [`error-handling.md`](./error-handling.md) §2). The host should not keep
  stepping after `Error`.

### §4.4 Choosing between `event_sender` and `trace_sender` for UI rendering

`event_sender` only exists in Mode A (it's a `run_game` arg). In Mode B the host reads
`interp.game_data` directly after every `step()`, so there is no `event_sender` seam — but the host
can still pass a `trace_sender` to `Interpreter::new` to receive per-transition `TraceEntry`s.

| Seam | Rate | Payload | Use for |
|---|---|---|---|
| `event_sender` (Mode A) | once per loop iteration (per `step()` call) | `&GameData` (full state snapshot) | Re-rendering the whole UI screen each step; cheap to snapshot via `clone()`. |
| `trace_sender` (Mode A + B) | once per FSM transition (same as `event_sender` in normal flow, but adds finer detail at quantifier sub-steps that don't transition state) | `TraceEntry` | Displaying a step log / move history / debug overlay; the `Quantifier` variant lets a UI show "chose player X" intermediate sub-steps. |

**Recommendation for the upcoming UI project:** use BOTH — `event_sender`/direct `game_data` for
the game view, `trace_sender` for a debug/status panel.

### §4.5 What the engine does NOT expose (the boundaries)

- **No traits for the UI to implement.** `IrExt` is `pub` but already implemented for the IR type;
  extension is by composition, not inheritance.
- **No state-pause/resume hook** beyond holding the `Interpreter` between `step()` calls in Mode B.
- **No rollback.** Mutation is in-place; an `Error` leaves a partial state.
- **No multi-thread sharing of a `Controller`.** `Send` but not `Sync` — see
  �6 §2.
- **No streaming of intermediate results** beyond the two callbacks (`event_sender` /
  `trace_sender`).
- **No way to inject custom mutations** — the only writes the engine performs are via `Payload`
  dispatch. A UI that wants custom effects must run its own `GameData` mutation outside the engine
  (in Mode B it owns the `GameData` and could mutate between `step()` calls, but this is
  **unsupported** and may break invariants — see I-1, I-7, I-18).

### §4.6 Worked "attach a UI" skeleton (Mode B)

```rust
// Sketch only — the UI host owns the Interpreter.
let mut interp = Interpreter::new(ir, GameData::new(), Some(trace_closure));
loop {
    match interp.step() {
        StepResult::Ok => ui.render(&interp.game_data),
        StepResult::NeedsInput(input_type) => {
            let input = ui.resolve_input(input_type);   // blocks on user gesture
            interp.provide_input(input);
        }
        StepResult::GameOver => { ui.show_final(&interp.game_data); break; }
        StepResult::Error(e) => { ui.show_error(&e); break; }
    }
}
```

`ui.resolve_input` must enforce the validation rule for each `InputType` arm — see §4.2's table.
Mode B is the recommended path for the upcoming UI project (see §4.1).

---

## §5. Observability & Trace Hooks for External Modules

- **`event_sender`** — rate: per iteration; payload: `&GameData`; use: screen re-render. Mode A
  only (a `run_game` arg). Cross-ref [`observability.md`](./observability.md) §1.
- **`trace_sender`** — rate: per FSM transition; payload: `TraceEntry`. Available in both modes
  (Mode A: `run_game` arg; Mode B: `Interpreter::new`'s 3rd arg). Shape: `TraceEntry::Step
  { from: u32, to: u32, event: TraceEvent }` where `TraceEvent` is one of `Action`,
  `Choice`, `OptionalAccept`, `OptionalDecline`, `Condition`, `EndCondition`, `StageRoundCounter`,
  `EndStage`, `Trigger`, `Quantifier` (full variant bodies in §1.5 above; deep dive in
  [`observability.md`](./observability.md) §2).
- **`MCG_TRACE_LOG` env var** — file-based trace. The engine composes file logging with the
  caller's `trace_sender` so hosts don't need to duplicate the file output. Log-file format
  (header / `[Step NNN] <Display>` / footer / panic line) is documented in
  [`observability.md`](./observability.md) §3.
- **`debug.rs`** — `DebugLevel::{Low, Medium, High}` formatted dumps; `format_game_data` /
  `save_game_data`. Cross-ref [`observability.md`](./observability.md) §4.
- **Structured/`tracing` integration:** not present; hosts that need it must add it themselves
  (cross-ref [`error-handling.md`](./error-handling.md) §1 / �6 §4).

---

## §6. Lifecycle, Threading & Resource Contract

The engine is **single-threaded and fully synchronous**; `run_game` blocks the calling thread
([`lifecycle.md`](./lifecycle.md) §1). `catch_unwind` captures panics into the trace log then
re-panics ([`observability.md`](./observability.md) §3.2) — UI hosts running Mode B own the
panic-handling themselves, no `catch_unwind` wraps `step()` directly.

### §6.1 Threading model

No `tokio`, no `async`, no `spawn` in the production path. The only `std::sync` usage sits in the
trace-logging plumbing:

- `TraceLogger` (`controller/trace_logger.rs:10`) stores `Arc<Mutex<BufWriter<File>>>` so the
  composed trace-sender closure (handed to `Interpreter`) can write back into the same writer.
- `run_game` allocates `Arc<Mutex<usize>>` as a step counter shared between the run loop and the
  composed trace sender (`controller/mod.rs:67,71-84`).
- When a trace log is open, `run_game` wraps `controller.run()` in
  `catch_unwind(AssertUnwindSafe(..))`, logs the panic, then `resume_unwind`s (no thread spawned).

The only concurrency-relevant public contract is the `Send + Sync` bound on the `Player` input
closure — so a host may move an `InputSource` across threads; the engine itself never spawns
threads.

### §6.2 `Send` / `Sync` characteristics

None of the engine types implement (or derive) `Send`/`Sync` explicitly; their auto-trait status
follows from their fields:

| Type | Auto `Send`? | Auto `Sync`? | Rationale |
|---|---|---|---|
| `GameData` | yes | yes | `Vec`/`HashMap` of `String`/`usize`/`i32`/`bool`/`u32`; no `Rc`/`RefCell`/raw. |
| `Interpreter` | yes | **no** | Carries `trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>` — a `Box<dyn Fn + Send>` is `Send` but not `Sync`. |
| `Controller` (private) | yes | **no** | `event_sender: Box<dyn Fn(&GameData) + Send>` is `Send` not `Sync`. |
| `InputSource` | yes | yes | `Player` closure explicitly `Send + Sync`; `TestFile(PathBuf)` likewise. |
| `Evaluator` | yes | yes | Zero-sized; no interior mutability. |
| `StepResult`, `Input`, `InputType`, `DebugLevel`, `TraceEntry`, `TraceEvent` | yes | yes | Plain data. |

**Interior mutability:** none in the production state machine (no `RefCell`/`Cell`/`RwLock`); the
two `Mutex` uses are observation-only. Callbacks receive `&GameData` / `TraceEntry` by value —
hosts that need a snapshot must `clone()`.

**Implication for hosts:** because `event_sender` is `Send` but not `Sync`, a host emitting events
from multiple worker threads must wrap the engine in its own `Mutex<Controller>` or run it on one
dedicated thread and communicate via channels.

### §6.3 Resource management

- **Memory:** `GameData` is a flat aggregate of owned `Vec`/`HashMap`; the only large allocation
  is the terminal `clone()` in `run_game` (O(total state); Mode B avoids it by reading
  `interp.game_data` in place). Card ids are never reused; `cards.len()` grows monotonically. The
  quantifier overlay is bounded by `FANOUT_CAP = 64` and never leaks past overlay-dispatch
  completion.
- **File descriptors:** the test-input file is opened lazily, consumed, and dropped within
  `read_test_file`; `save_game_data` opens per call; `TraceLogger::open` opens once per
  `run_game` and drops on return. No FD leaks.
- **Network sockets:** none — the engine is transport-agnostic (each player runs their own
  backend per the workspace P2P intent).
- **Drop order:** `Controller` owns `Interpreter` owns `GameData`; standard Rust drop order; no
  `Drop` impls in the crate.

### §6.4 Dependencies inventory

`crates/engine/Cargo.toml`:

- **Library target:** `front_end` (IR/AST/lowering); `serde_json` (`alloc_synth`'s `StateID`
  construction — `serde` is not a direct dependency); `rand` (`ShuffleAction`,
  `CreateTurnorderRandom`).
- **`engine-tui` binary:** `ratatui` + `crossterm` (terminal UI), `crossbeam-channel` (input
  loop).
- **`cgdsl-play` binary:** no extra dependencies (auto-discovered; only `engine-tui` has an
  explicit `[[bin]]` entry).
- Error handling is stringly-typed (`Result<_, String>`); no unused dependencies remain.

---

## §7. Worked Examples

> The examples from the former `api-usage.md` live here. For the extension seams
> (`InputSource::Player`, `event_sender`, `trace_sender`) and the `Evaluator` read-side methods,
> see §1, §4.2, and §1.4 above.

### §7.1 The golden path — `run_game`

The canonical end-to-end flow is: parse `.cgdsl` → lower to
`front_end::ir::Ir<front_end::ir::LoweredPayLoad>` → construct an empty `GameData` → supply an
`InputSource` (and optionally an event and/or trace callback) → call `run_game`. The runnable
example below mirrors `crates/engine/src/bin/cgdsl-play.rs` but is written as an external
consumer and adds the event-sender and trace-sender hooks:

```rust
// In a downstream crate's example or test. Requires:
//   cgdsl-engine = { path = "../crates/engine" }
//   front_end    = { path = "../crates/front_end" }
use std::path::PathBuf;
use std::sync::Mutex;

use cgdsl_engine::{
    run_game, GameData, Input, InputKind, InputSource, InputType, TraceEntry, DebugLevel, format_game_data,
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
    //        return an Input. Must be Send + Sync (see §6.2).
    #[allow(dead_code)]
    fn interactive_input(input_type: InputType) -> Input {
        match input_type {
            InputType::Choice { options, max_index } => {
                println!("Choose: {:?}", options);
                // ...read stdin, validate 0..=max_index, return Input { player_id, kind: InputKind::Choice { idx } }
                Input { player_id: "P1".into(), kind: InputKind::Choice { idx: 0 } }
            }
            InputType::Optional(prompt) => {
                println!("{}", prompt);
                Input { player_id: "P1".into(), kind: InputKind::OptionalAccept }
            }
            InputType::ChoosePlayer { candidates, prompt } => {
                println!("{}: {:?}", prompt, candidates);
                // ...return Input { player_id, kind: InputKind::ChoosePlayer { idx } }
                Input { player_id: "P1".into(), kind: InputKind::ChoosePlayer { idx: 0 } }
            }
            InputType::ChooseCards { display, min, max, prompt } => {
                println!("{}: {}..={} of {:?}", prompt, min, max, display);
                // ...return Input { player_id, kind: InputKind::ChooseCards { selected: vec![0] } }
                Input { player_id: "P1".into(), kind: InputKind::ChooseCards { selected: vec![0] } }
            }
        }
    }
    let _interactive = InputSource::Player(Box::new(interactive_input));

    // 4. Optional event hook: invoked with &GameData after every loop iteration and
    //    once more at GameOver. Receives a shared reference — do NOT mutate through
    //    it. (Closure bound is `Fn(&GameData) + Send` — NOT Sync; see §6.2.)
    let event_sender: Option<Box<dyn Fn(&GameData) + Send>> = Some(Box::new({
        move |gd: &GameData| {
            // Render a frame, log a diff, or forward over a channel. For a snapshot
            // you must `clone()` — see observability.md §1.
            println!("{}", format_game_data(gd, DebugLevel::Medium));
        }
    }));

    // 5. Optional trace hook: invoked once per FSM *transition* with a TraceEntry
    //    (not per loop iteration). Bound is `Fn(TraceEntry) + Send` (also NOT Sync,
    //    see §6.2). See observability.md §2.
    let trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>> = Some(Box::new({
        move |entry: TraceEntry| {
            // entry implements Display, so a one-line trace readout is trivial:
            println!("{}", entry);
        }
    }));

    // 6. Run to completion. Returns the terminal GameData (deep-cloned) or an
    //    error string. This call BLOCKS the current thread until GameOver/Error.
    //    When MCG_TRACE_LOG is set, `run_game` composes the file logger with the
    //    caller's trace_sender, so hosts do NOT need to duplicate file logging
    //    (see observability.md §3.1).
    match run_game(ir, game_data, input_source, event_sender, trace_sender) {
        Ok(final_state) => {
            println!("Game over. {} players still in game.",
                final_state.players.iter().filter(|p| p.in_game).count());
            println!("{}", cgdsl_engine::format_game_data(&final_state, cgdsl_engine::DebugLevel::High));
        }
        Err(e) => eprintln!("Game error: {e}"),
    }
}
```

A minimal replay test (no event hook, no trace hook, no `MCG_TRACE_LOG`) reduces to one line,
exactly as the in-tree test `crates/engine/src/controller/tests.rs` does:

```rust
let result = run_game(ir, GameData::new(), InputSource::TestFile(path), None, None);
assert!(result.is_ok());
```

> **Signature:** `pub fn run_game(ir, game_data, input_source, event_sender, trace_sender) -> Result<GameData, String>`.
> The two `Option<Box<dyn Fn + Send>>` arguments may be mixed freely; `run_game` composes them
> with the optional `MCG_TRACE_LOG` file logger internally (see [`observability.md`](./observability.md) §3.1).

### §7.2 Driving the interpreter manually (Mode B)

Hosts that need finer-grained control (e.g., to persist state between requests in a server) can
skip `run_game` and drive an `Interpreter` directly — this is the contract
`Controller::run` itself implements:

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
behavior — those live in `run_game`. If you drive `Interpreter` directly and want trace logging,
pass a `trace_sender` to `Interpreter::new` and write the file yourself, or call `run_game`.