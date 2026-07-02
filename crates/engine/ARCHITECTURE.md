# `crates/engine` — Architecture & Reference Manual

> Crate name (Cargo): `cgdsl-engine` · Binary: `cgdslsl-play` (`src/bin/cgdsl-play.rs`) · Edition 2021
> Scope: this document covers `crates/engine/**` exclusively. Types from the dependency crate
> `front_end` are referenced only where the engine's public contract depends on them.

---

## 1. Executive Architecture & Design Overview

### 1.1 High-Level Purpose

`crates/engine` is the **runtime execution kernel** for MCG's *Card Game DSL* (`cgdsl`). It does
not parse the DSL itself — that is the responsibility of the `front_end` crate. Instead, the engine
consumes a **lowered intermediate representation** (`Ir<LoweredPayLoad>`, a finite-state machine)
produced by `front_end` from `.cgdsl` source, and **drives that FSM to completion** against an
initially-empty `GameData`, mutating the game state in lock-step with each FSM transition.

Concretely, the engine solves three problems:

1. **State-machine execution** — given the current `StateID`, inspect the outgoing `Edge`s and
   advance exactly one transition per `step()` (`interpreter.rs`).
2. **State mutation** — translate each traversed edge's `Payload` into concrete writes on
   `GameData` (players, locations, cards, memories, stage counters, turn order) (`action.rs`).
3. **State query / expression evaluation** — evaluate the DSL's expression sub-language
   (`BoolExpr`, `IntExpr`, `CardSet`, `PlayerExpr`, …) over the live `GameData` to resolve
   conditions, end-conditions, card sets, quantities, and ownership (`query.rs`).

A fourth concern, **I/O orchestration**, is handled by `controller.rs`, which owns the main loop
and feeds external input (human or recorded) into the interpreter whenever a transition stalls on
`StepResult::NeedsInput`.

### 1.2 Architectural Pattern

The engine is a **finite-state-machine (FSM) interpreter layered over a data-oriented state store**,
combined with a **visitor/evaluator** over an external expression-tree. It is *not* ECS, *not*
actor-model, *not* async. Three cooperating patterns compose it:

| Layer | Pattern | Implementation |
|---|---|---|
| Orchestration | **Pull-driven event loop** with an external input source | `Controller::run` (`controller.rs:62`) |
| Transition | **Single-step FSM interpreter** dispatching on edge payload | `Interpreter::step` (`interpreter.rs:23`) |
| Write side | **Payload → mutation dispatch** (one arm per `Payload`/`GameRule` variant) | `action::execute` (`action.rs:45`) |
| Read side | **Recursive-descent evaluator** (stateless, associated functions) | `Evaluator` (`query.rs:177`) |
| State | **Flat, mutable aggregate** (`GameData`) with index-based references | `game_data.rs:24` |

The control flow is strictly synchronous and single-threaded: `run()` loops calling `step()`; each
`step()` may synchronously call into `Evaluator` (reads) and `action::execute` (writes); when
`step()` returns `NeedsInput`, `run()` consults the `InputSource` and pushes one `Input` back before
the next iteration. There is no scheduler, no green threads, no `Future`s.

### 1.3 Module Hierarchy & Dependency Flow

All six modules are declared public in `lib.rs:1-6` and re-exported at the crate root
(`lib.rs:8-11`). Internal dependency flow (arrow = "uses"):

```
                       front_end::ir  (Ir, Edge, StateID, Payload, LoweredPayLoad)
                       front_end::ast (GameRule, ActionRule, SetUpRule, MoveType, …)
                       front_end::validation::parse_document  (used only by the bin + tests)
                                │
                                ▼
  ┌─────────────┐    ┌──────────────────┐    ┌───────────────────┐
  │ controller  │──▶ │   interpreter    │──▶ │     action        │  (write side)
  │  (run_game, │    │  (step, edges,   │    │  (execute payload │
  │   InputSrc, │    │   StepResult)    │    │   → GameData mut) │
  │   events)   │    └────────┬─────────┘    └─────────┬─────────┘
  └─────┬───────┘             │                        │
        │                     ▼                        ▼
        │            ┌──────────────────┐      ┌───────────────────┐
        │            │     query        │◀────▶│    game_data      │  (state store)
        │            │  (Evaluator:     │      │  (GameData, Card, │
        │            │   eval_*)        │      │   Player, …)      │
        │            └──────────────────┘      └───────────────────┘
        ▼
 ┌─────────────┐
 │   debug     │  (format/print/save GameData; observability only)
 └─────────────┘
```

Public-facing vs. internal:

- **Public-facing API surface** (the contract external crates may rely on): `run_game`,
  `InputSource` (`controller.rs`); `Interpreter`, `Input`, `InputType`, `StepResult`
  (`interpreter.rs`); the `GameData` family of structs and `Card` type alias (`game_data.rs`);
  `Evaluator` and its `pub` methods (`query.rs`); `DebugLevel`, `format_game_data`,
  `print_game_data`, `save_game_data` (`debug.rs`).
- **Internal-only**: `Controller` (`controller.rs:49`) is `struct` (not `pub`); `action::execute`
  and all `execute_*` helpers are `pub fn` but are intended for the interpreter's use (no `pub use`
  re-export of `action` symbols at crate root beyond the module itself); `query.rs`'s private
  helpers (`eval_int_collection`, `eval_group`, `apply_filter`, `card_matches_filter`,
  `infer_location_from_cards`, …) are module-private.

The binary `cgdsl-play` (`src/bin/cgdsl-play.rs`) is a thin CLI driver that wires
`front_end::validation::parse_document` → `SGame::to_lowered_graph` → `run_game`; it is **not**
part of the library target.

---

## 2. Core Data Structures & State Management

### 2.1 The State Store: `GameData` and its Family

All runtime state lives in a single aggregate, `GameData` (`game_data.rs:24`). It derives only
`Clone` (no `Debug`, no `Serialize` — serialization is handled separately by `debug.rs`).

```rust
// game_data.rs:22
pub type Card = HashMap<String, String>;

// game_data.rs:24-39
#[derive(Clone)]
pub struct GameData {
    pub table: OwnerData,
    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    pub turn_order: Vec<usize>,
    pub locations: Vec<Location>,
    pub cards: Vec<Card>,
    pub combos: Vec<Combo>,
    pub precedences: Vec<Precedence>,
    pub point_maps: Vec<PointMap>,
    pub current_player: Option<usize>,
    pub stage_counters: HashMap<String, u32>,
    pub stage_stack: Vec<String>,
    pub memories: HashMap<String, MemoryValue>,
}
```

A `Card` is deliberately schemaless — a bag of string key/value attributes (e.g. `Rank → Ace`,
`Suite → Hearts`). Cards are stored **only** in `cards: Vec<Card>` and referenced elsewhere by
`usize` index (a "card id"). Locations hold card ids, not cards.

| Struct | Location | Fields | Role |
|---|---|---|---|
| `OwnerData` | `game_data.rs:54` | `locations: Vec<usize>` | Ownership of location indices; held by both `GameData::table` and each `Player`. |
| `Player` | `game_data.rs:60` | `name, score: i32, owner: OwnerData, in_game: bool, in_stage: HashMap<String,bool>` | Per-player state; `in_stage` tracks participation per named stage. |
| `Location` | `game_data.rs:69` | `name: String, cards: Vec<usize>` | A named pile; `cards` is an ordered list of card ids. |
| `Team` | `game_data.rs:75` | `name, players: Vec<usize>` | Named group of player indices. |
| `Combo` | `game_data.rs:81` | `name: String, filter: FilterExpr` | A named, reusable card filter (from `front_end::ast`). |
| `Precedence` | `game_data.rs:88` | `name, key: String, values: Vec<String>` | Ordered values on one key, low→high. Used by `Adjacent`/`Higher`/`Lower`/`ExtremaPrecedence`. |
| `PointMap` | `game_data.rs:96` | `name, map: HashMap<String,i32>` | Maps `"key:value"` → points. Used by `SumOfCardSet`, `ExtremaCardset`, `ExtremaPointMap`. |

```rust
// game_data.rs:42-51
#[derive(Clone)]
pub enum MemoryValue {
    Int(i32),
    String(String),
    CardSet(Vec<usize>),
    PlayerCollection(Vec<usize>),
    Team(String),
    IntCollection(Vec<i32>),
    StringCollection(Vec<String>),
    LocationCollection(Vec<usize>),
}
```

`MemoryValue` is the dynamically-typed storage for DSL "memory" variables. There is **no** separate
`TeamCollection` variant — a stored team collection is represented as `Team(String)` holding one
team name (see `query.rs:637-647`), and `MemoryType::TeamCollection` initializes to `MemoryValue::Int(0)`
(`game_data.rs:261`), a known mismatch documented in §2.4.

### 2.2 The IR the Engine Consumes (defined in `front_end::ir`)

The engine does not own its IR types; it parameterizes over `front_end::ir`:

```rust
// front_end/src/ir.rs:43
pub struct StateID(u32);            // Copy, Eq+Hash+Ord

// front_end/src/ir.rs:53-60
pub struct Edge<T: serde::Serialize> { pub to: StateID, pub payload: T, pub meta: Option<Vec<Meta>> }

// front_end/src/ir.rs:79-83
pub struct Ir<T: serde::Serialize> { pub states: HashMap<StateID, Vec<Edge<T>>>, pub entry: StateID, pub goal: StateID }
```

`Payload<Ctx>` is the sum of transition kinds (`front_end/src/ir.rs:252-268`):

```rust
pub enum Payload<Ctx: AstContext> {
    Condition { expr: Ctx::Condition, negated: bool },
    EndCondition { expr: Ctx::EndCondition, negated: bool, stage: Ctx::Id },
    Action(Ctx::GameRule),
    StageRoundCounter(Ctx::Id),
    EndStage(Ctx::Id),
    Choice,
    Optional,
    Trigger,
}
```

The engine operates on the *lowered* specialization (`front_end/src/ir.rs:313-322`):

```rust
pub type LoweredPayLoad = Payload<LoweredCtx>;
// where LoweredCtx resolves: Condition→BoolExpr, EndCondition→EndCondition,
//   GameRule→GameRule, Id→String.
```

So at the engine boundary, `Ir<LoweredPayLoad>` is `HashMap<StateID, Vec<Edge<LoweredPayLoad>>>`
plus `entry`/`goal`. The `Edge.meta` field is **ignored** by the engine — it is read only by
`front_end::fsm_to_dot`.

### 2.3 The Execution Types

```rust
// interpreter.rs:15-20
pub struct Interpreter {
    pub ir: Ir<LoweredPayLoad>,
    pub game_data: GameData,
    pub input_buffer: Vec<Input>,
    pub current_state: StateID,
}

// interpreter.rs:156-161  (LIFO stack; see §2.4 invariant I-7)
#[derive(Clone, Debug, PartialEq)]
pub enum Input { Choice { idx: usize }, OptionalAccept, OptionalDecline }

// interpreter.rs:173-178
pub enum StepResult { Ok, NeedsInput(InputType), GameOver, Error(String) }

// interpreter.rs:180-187
#[derive(Clone)]
pub enum InputType { Choice { options: Vec<String>, max_index: usize }, Optional(String) }
```

`Input::idx()` (`interpreter.rs:164-170`) normalizes all three variants to a 0-based edge index:
`Choice{idx}` → `idx`, `OptionalAccept` → `0`, `OptionalDecline` → `1`.

```rust
// controller.rs:15-18
pub enum InputSource {
    Player(Box<dyn Fn(InputType) -> Input + Send + Sync>),
    TestFile(PathBuf),
}

// controller.rs:49-57  (internal — NOT re-exported)
struct Controller {
    interpreter: Interpreter,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    line_buffer: VecDeque<String>,
    file_loaded: bool,
    loaded_line_count: usize,
    input_sequence: usize,
}
```

`Evaluator` (`query.rs:177`) is a zero-sized `pub struct` used purely as a namespace for
associated functions (`eval_bool`, `eval_int`, `eval_string`, `eval_player`, `eval_team`,
`eval_cardset`, `eval_card_position`, `eval_end_condition`, `eval_compare`, `eval_int_compare`,
`resolve_players`, `resolve_player_collection`, `resolve_owner_to_name`, `resolve_quantity`,
`expand_types`, `check_attr_value_in_cardset`). It holds **no state**; every method takes
`&GameData` (or `&mut GameData` for none of them — all reads are immutable).

### 2.4 System Invariants (Critical for AI Agents)

These rules are derived directly from the code. Violating them will silently corrupt game state or
hang the run loop.

> **I-1 — Index indirection for the current player.**
> `GameData::current_player: Option<usize>` indexes **`turn_order`**, not `players`.
> `turn_order` holds indices into `players`. The helper `get_current_player`
> (`game_data.rs:178-183`) does `turn_order[current_player] → players[…]`. Code that treats
> `current_player` as a player index is wrong. `CycleAction` (`action.rs:206-220`) deliberately
> stores the *turn-order position* (`turn_order.iter().position(...)`), not the player index.

> **I-2 — `GameData::new()` initializes `current_player = Some(0)` with an empty `turn_order`.**
> (`game_data.rs:113`). Until `CreateTurnorder`/`CreatePlayer` populate `turn_order`, calling
> `get_current_player()` returns `None` (safe), but any direct indexing of `turn_order[0]` would
> panic. The `Some(0)` sentinel must not be assumed valid before setup.

> **I-3 — Condition vs. EndCondition edge indexing is INVERTED.**
> Both require **exactly 2 outgoing edges** or `step()` returns `Error` (`interpreter.rs:72-76`,
> `interpreter.rs:99-103`). But the chosen edge differs:
> - `Condition` (`interpreter.rs:81-86`): `should_take_else = result != negated`; `true` → `edges[1]`,
>   `false` → `edges[0]`.
> - `EndCondition` (`interpreter.rs:112-117`): `should_exit = result != negated`; `true` → `edges[0]`
>   (exit), `false` → `edges[1]` (continue).
>
> So edge **0 is the "true/exit" branch for EndCondition but the "false" branch for Condition**.
> Any change to the IR builder's edge ordering or to these match arms must be mirrored across both
> or games will branch backwards.

> **I-4 — `GameOver` requires *both* no outgoing edges AND `current_state == ir.goal`.**
> (`interpreter.rs:29-34`). A dead-end state that is not the goal yields
> `StepResult::Error("No outgoing edges and not at goal state")`, not `GameOver`. An agent adding a
> terminal state must ensure it is registered as `ir.goal`.

> **I-5 — `StageRoundCounter` and `EndStage` payloads are applied TWICE per traversal.**
> `Interpreter::step` handles these payloads by mutating `game_data` and **then** calling
> `execute_edge(edge.clone())` (`interpreter.rs:125-134`), which calls `action::execute(edge.payload,
> &mut game_data)` (`interpreter.rs:145-148`). `action::execute` has **its own** arms for
> `StageRoundCounter` (`action.rs:52-54`) and `EndStage` (`action.rs:55-57`) that perform the same
> mutation again. Net effect: the stage round counter is incremented **twice** per traversal, and
> `leave_stage` is called **twice** (the second call on an already-popped stack is a no-op only if
> the stage was unique — otherwise it pops further stages). `Trigger` is unaffected because
> `action::execute`'s catch-all `_ => {}` (`action.rs:58`) swallows it. Agents must either preserve
> this double-application or refactor both call sites together.

> **I-6 — Cards have no reverse location index; location is found by linear scan.**
> `add_card` (`game_data.rs:154-157`) takes a `_location_id` parameter it **ignores** — it only
> appends to `cards` and returns the new id. The caller (`action.rs:116-119`) then pushes the id
> into `locations[loc_idx].cards`. To find which location holds a card, the engine scans all
> locations (`query.rs:964-968`, `query.rs:1020-1026`, `query.rs:1376-1388`,
> `action.rs:357-359`). `execute_cardset_move` scans **all locations per moved card**
> (`action.rs:357-359`) — O(cards × locations). Do not assume O(1) card location lookup.

> **I-7 — The input buffer is a LIFO stack, not a queue.**
> `provide_input` pushes (`interpreter.rs:151-153`), `step()` pops (`interpreter.rs:43,60`). In
> normal flow only one input is ever buffered at a time, but if an agent ever pushes multiple,
> they are consumed in reverse order.

> **I-8 — Out-of-range `Choice`/`Optional` input is a silent no-op that stalls the FSM.**
> `step()` does `if let Some(choice_edge) = edges.get(input.idx()) { self.execute_edge(...) }`
> then unconditionally returns `Ok` (`interpreter.rs:42-57`, `59-69`). If `input.idx()` is out of
> range, `execute_edge` is skipped, `current_state` is **not** advanced, and the next `step()`
> re-enters the same state — which, now that the buffer is empty, yields `NeedsInput` again. For
> `InputSource::TestFile` this **consumes the next recorded line** as a re-prompt; for
> `InputSource::Player` the controller's own validation loop (`controller.rs:86-96`, using
> `idx > max_index`) re-invokes the closure. There is no central validation: the interpreter trusts
> the index, the controller validates only the `Player` path.

> **I-9 — `set_memory` does not set; it increments an `Int` memory by 1.**
> `set_memory` (`game_data.rs:272-278`) ignores its `memory_type` argument entirely and, if the
> stored value is `MemoryValue::Int`, does `*v += 1`. It silently no-ops on non-`Int` memories.
> `reset_memory` (`game_data.rs:280-286`) only resets `Int` memories. This is the current behavior,
> not a spec'd design — agents implementing DSL `SetMemory` semantics must not assume general
> assignment here.

> **I-10 — `add_memory` initializes some `MemoryType`s to mismatched `MemoryValue`s.**
> (`game_data.rs:251-265`): `MemoryType::Player` → `MemoryValue::Int(0)` (not a player),
> `MemoryType::TeamCollection` → `MemoryValue::Int(0)` (not a collection). Reads of these
> (`query.rs:863-884`, `query.rs:637-647`) will therefore fail type checks until something writes a
> correctly-typed value. Agents adding new memory writes must respect the read-side expected
> `MemoryValue` variant.

> **I-11 — `leave_stage` pops the stage stack until (and including) the named stage.**
> (`game_data.rs:227-234`). This permits multi-stage jumps (an end-condition that exits several
> nested stages at once). If the named stage is not on the stack, the **entire stack is drained**.
> `get_current_stage()` (`game_data.rs:212-214`) returns `stage_stack.last()`.

> **I-12 — `enter_stage` is never called by the engine's action layer.**
> Stage entry (`game_data.rs:216-225`) is defined but no `ActionRule`/`SetUpRule` invokes it. Stage
> participation flags (`in_stage`) are only mutated by `set_player_stage_flag`
> (`game_data.rs:206-210`, called from `OutAction`, `action.rs:186-194`) and `enter_stage`. The
> `in_stage` map is relied upon by turn resolution (`resolve_turn`, `game_data.rs:236-249`) and by
> `RuntimePlayer::Next`/`OutOf::CurrentStage`. An agent that adds stage-entry logic must populate
> `in_stage` for every player or `resolve_turn` will find no eligible player and return `None`.

> **I-13 — `resolve_turn` / `next_player` find the next *eligible* player, wrapping the turn order.**
> (`game_data.rs:185-197`, `236-249`). Eligible = `in_game && in_stage[current_stage]`. If none is
> eligible, `current_player` becomes `None` and the game is effectively stuck (no `Error` is
> raised). `next_player` uses `.unwrap()` on the found position (`game_data.rs:192`) — safe only
> because `resolve_turn` returning `Some(idx)` guarantees the idx is in `turn_order`.

> **I-14 — `eval_cardset` returns `(location_idx, card_ids)`; the location is best-effort.**
> For `CardSet::Memory` with cards not found in any location, it returns `(0, card_ids)`
> (`query.rs:970`) — **location index 0 is a fallback sentinel**, not a real answer.
> `infer_location_from_cards` (`query.rs:1372-1389`) similarly falls back to `Ok(0)`. Consumers
> that index `locations[0]` after such a result may read an unrelated pile.

> **I-15 — `InputSource::Player`'s validation loop can spin forever.**
> `controller.rs:86-96` re-calls `callback(input_type)` with `continue` while the returned
> `Choice.idx` exceeds `max_index`. A buggy closure that always returns an out-of-range index will
> hang the run loop with no error. The `TestFile` path has no such loop (it consumes one line per
> request and errors on exhaustion).

### 2.5 State Lifecycle

1. **Construction.** `GameData::new()` (`game_data.rs:102-118`) produces an empty store with
   `current_player = Some(0)`, empty `stage_stack`, empty `memories`. `run_game`
   (`controller.rs:24-46`) wraps it in an `Interpreter` (`current_state = ir.entry`, empty
   `input_buffer`) and a `Controller`.
2. **Setup phase.** The IR's first edges carry `Payload::Action(GameRule::SetUp{…})`. These run
   `execute_setup_rule` (`action.rs:62-157`), which is the **only** place `players`, `teams`,
   `turn_order`, `locations`, `cards`, `combos`, `precedences`, `point_maps`, and `memories` are
   *created*. Order matters: `CreateLocation` resolves an owner by name (`action.rs:95-96`,
   `.expect`), so `CreatePlayer`/`CreateTeams` must precede it; `CreateCardOnLocation`
   `.expect("Location not found")` (`action.rs:112`) requires the location to exist.
3. **Play phase.** Each `step()` either advances `current_state` via `execute_edge`
   (`interpreter.rs:145-148`: `current_state = edge.to; action::execute(edge.payload, &mut game_data)`),
   or yields `NeedsInput`/`GameOver`/`Error`. Conditions and end-conditions are evaluated live
   against the current `GameData` by `Evaluator`.
4. **Termination.** On `GameOver`, `run()` emits a final event and returns
   `Ok(self.interpreter.game_data.clone())` (`controller.rs:72-75`) — a **full deep clone** of the
   terminal state. On `Error(String)`, it returns `Err(String)` (`controller.rs:76`). The
   `Controller` and `Interpreter` are then dropped; no explicit teardown is required (the test-file
   `File` handle is dropped at the end of `read_test_file`'s loading block).

---

## 3. Concurrency, Memory, & Thread Safety

### 3.1 Threading Model

The engine is **single-threaded and fully synchronous**. There is no `tokio`, `async`, `spawn`, or
`Arc`/`Mutex` in `crates/engine/src`. The main loop (`Controller::run`, `controller.rs:62-79`) is a
plain `loop { … }` that calls `self.interpreter.step()` directly. The only "concurrency-relevant"
construct is the `Send + Sync` bound on the `Player` input closure (`controller.rs:16`), which
exists so a *host* application can move an `InputSource` across threads — but the engine itself
never spawns threads.

### 3.2 `Send` / `Sync` Characteristics

None of the engine types explicitly implement (or derive) `Send`/`Sync`; their auto-trait status
follows from their fields:

| Type | Auto `Send`? | Auto `Sync`? | Rationale |
|---|---|---|---|
| `GameData` | yes | yes | All fields are `Vec`/`HashMap` of `String`/`usize`/`i32`/`bool`/`u32`; no `Rc`/`RefCell`/raw. |
| `Interpreter` | yes | yes | `Ir<LoweredPayLoad>` (serde types), `GameData`, `Vec<Input>`, `StateID(u32)`. |
| `Controller` | yes | **no** | `event_sender: Option<Box<dyn Fn(&GameData) + Send>>` — the bound is `Send` but **not `Sync`** (`controller.rs:52`). Two threads cannot share `&Controller` safely. |
| `InputSource` | yes | yes | `Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)` explicitly `Send + Sync`; `TestFile(PathBuf)` is `Send + Sync`. |
| `Evaluator` | yes | yes | Zero-sized; no interior mutability. |
| `StepResult` | yes | yes | Plain enum (`String` is `Send+Sync`). |
| `Input`, `InputType`, `DebugLevel` | yes | yes | Plain data. |

**Interior mutability:** none. The engine uses `&mut GameData` passed down the call stack
(`action::execute(payload, &mut game_data)`, `Evaluator` takes `&GameData`). There are no
`RefCell`/`Cell`/`Mutex`/`RwLock` anywhere in the crate. The only shared mutability pattern is the
`event_sender` callback, which receives `&GameData` (shared ref) — it must not attempt to mutate
through the provided reference; hosts that need to snapshot must `clone()`.

**Implication for hosts:** because `event_sender` is `Send` but not `Sync`, a host that wants to
emit events from multiple worker threads must wrap the engine in its own `Mutex<Controller>` or run
the engine on a single dedicated thread and communicate via channels.

### 3.3 Resource Management

- **Memory:** `GameData` is a flat aggregate of owned `Vec`/`HashMap`. The only large allocation
  point is the terminal `clone()` in `run()` (`controller.rs:74`) — for a game with many cards this
  is O(total state). There is no arena, slab, or recycling; card ids are never reused (only
  appended), so `cards.len()` grows monotonically.
- **File descriptors:** the test-input `File` (`controller.rs:118`) is opened lazily on the first
  `NeedsInput` and its `BufReader` is consumed and dropped within `read_test_file`'s loading block
  (`controller.rs:117-129`). No FD leaks across a run. `save_game_data` (`debug.rs:255-271`) opens
  a file in `append(true).create(true)` mode per call and drops it on return.
- **Network sockets:** none. The engine has no networking; per the workspace's P2P architecture
  intent, each player runs their own backend and this crate is transport-agnostic.
- **Drop order:** `Controller` owns `Interpreter` owns `GameData`; standard Rust drop order
  suffices. No `Drop` impls exist in the crate.

### 3.4 Unused Dependencies (Agent Note)

`Cargo.toml` declares `indexmap`, `dashmap`, `thiserror`, and `anyhow`, but **none are imported
anywhere in `crates/engine/src`** (verified: only `std::collections::HashMap` is used, in
`game_data.rs`, `query.rs`, and `action.rs`). Error handling is stringly-typed (`Result<_, String>`,
`StepResult::Error(String)`) — `thiserror`/`anyhow` are not exercised. Agents should not assume
these crates are available to engine code without re-adding a real import; conversely, removing
them from `Cargo.toml` is safe as of this writing.

---

## 4. Public API & End-to-End Usage Manual

### 4.1 The Golden Path

The canonical end-to-end flow is: parse `.cgdsl` → lower to `Ir<LoweredPayLoad>` → construct an
empty `GameData` → supply an `InputSource` (and optionally an event callback) → call `run_game`.
The runnable example below mirrors `src/bin/cgdsl-play.rs` but is written as an external consumer
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
    //    (Closure bound is `Fn(&GameData) + Send` — NOT Sync; see §3.2.)
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
`controller.rs:264` does:

```rust
let result = run_game(ir, GameData::new(), InputSource::TestFile(path), None);
assert!(result.is_ok());
```

### 4.2 Driving the Interpreter Manually (Advanced)

Hosts that need finer-grained control (e.g., to persist state between requests in a server) can
skip `run_game` and drive an `Interpreter` directly. This is the contract `Controller` itself
implements (`controller.rs:62-78`):

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
        StepResult::Error(msg) => { eprintln!("{msg}"); break; }  // see §5.2
    }
}
// interp.game_data is the final state (no extra clone is performed here).
```

### 4.3 Primary Traits & Extension Points

The engine deliberately exposes **no traits** for downstream implementation. Extension is by
composition, at three seams:

1. **`InputSource::Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)`** (`controller.rs:16`).
   The single seam for front-ends (GUI, CLI, networked bot). The closure receives the full
   `InputType` (choice options + `max_index`, or an optional prompt) and returns an `Input`. It may
   block arbitrarily (e.g., on user input) — the engine waits. Validation is the closure's
   responsibility for non-`Choice` paths; for `Choice`, the controller re-invokes on
   `idx > max_index` (see I-8/I-15).

2. **`event_sender: Option<Box<dyn Fn(&GameData) + Send>>`** (`controller.rs:52`, `run_game`'s 4th
   arg). Reactive observability: called with a snapshot of `GameData` after every successful `step`
   and once more immediately before `GameOver` return (`controller.rs:64,73`). `Send` but not
   `Sync` — see §3.2. Typical use: render a GUI frame, log a diff, or forward over a channel.

3. **The DSL itself.** Because the engine is a pure interpreter over `front_end`'s IR, the primary
   "extension" for new game mechanics is authoring `.cgdsl` (which `front_end` lowers into
   `GameRule`/`SetUpRule`/`ActionRule` variants). The engine's `action.rs`/`query.rs` must already
   implement the corresponding variant — variants with `// TODO` (see §5.3) are no-ops.

`Evaluator`'s `pub` methods (`eval_bool`, `eval_int`, `eval_string`, `eval_player`, `eval_team`,
`eval_cardset`, `eval_card_position`, `resolve_quantity`, `expand_types`, …) are also available
for hosts that want to query a `GameData` outside of a running game (e.g., to render a derived
statistic). All take `&GameData` and return `Result<T, String>` (or `Vec<usize>` for resolvers).

---

## 5. Error Handling, Panic Conditions, & Diagnostics

### 5.1 Error Types

The engine is **stringly-typed throughout** — there is no custom `Error` enum despite `thiserror`
being declared in `Cargo.toml` (see §3.4). Three error channels exist:

| Channel | Type | Origin |
|---|---|---|
| Run failure | `Result<GameData, String>` from `run_game` (`controller.rs:29`) | Propagated from `StepResult::Error` or `get_input`. |
| Step failure | `StepResult::Error(String)` (`interpreter.rs:177`) | Missing state, bad edge counts, evaluator errors (below). |
| Eval failure | `Result<_, String>` from every `Evaluator` method | Division by zero, missing memory/location/precedence/pointmap/combo, type-mismatched memory, out-of-range index, "no current player/stage", etc. |

Representative `Evaluator` error strings (verbatim from `query.rs`): `"Division by zero"`
(`query.rs:387`), `"No current stage"` (`query.rs:508`), `"Memory {key} not found"`, `"Memory
value is not an Int"` (and `String`/`Team`/`CardSet`/… variants), `"Location {name} not found"`,
`"PointMap {name} not found"`, `"Precedence {name} not found"`, `"Combo {name} not found"`,
`"No card at index {idx} in location {loc}"`, `"No card at top of location {loc}"`,
`"No next player available"`, `"No competitor found"`, `"Owner of card position not found"`,
`"No card found for extrema"`, `"resolve_owner_to_name: PlayerCollection cannot resolve to a single
name"`, `"Card position not found in any location"`.

Controller-level errors: `"Failed to open test file: {e}"`, `"Failed to read test file: {e}"`
(`controller.rs:118,121`), `"Test input file exhausted"` (`controller.rs:134`), `"Invalid test
input #{n}: expected number, 'y', or 'n', got '{line}'"` (`controller.rs:140-145`), `"Invalid test
input #{n}: choice indices start at 1, got 0"` (`controller.rs:146-151`).

Interpreter-level errors: `"Current state not found in IR"` (`interpreter.rs:26`), `"No outgoing
edges and not at goal state"` (`interpreter.rs:33`), `"Condition state must have exactly 2 edges"`
(`interpreter.rs:73`), `"EndCondition state must have exactly 2 edges"` (`interpreter.rs:100`),
`"Failed to get condition edge"` / `"Failed to get end condition edge"` (`interpreter.rs:91,122`),
`"No edges found"` (`interpreter.rs:141`).

### 5.2 Recoverable vs. Unrecoverable Paths

**Recoverable** (surfaced as `Err(String)` / `StepResult::Error`): all `Evaluator` `Result`
returns; condition/end-condition edge-count violations; missing current state in the IR; dead-end
non-goal states; test-file open/parse/exhaustion errors. These terminate `run_game` with `Err` and
leave `GameData` in whatever partially-mutated state it reached (the engine does **not** roll back
applied mutations on error — `execute_edge` has already written before a later evaluator call can
fail).

**Unrecoverable** (process-aborting `panic!` / `.expect()` / `.unwrap()` / `todo!`). These are
invariants the code assumes the IR/data will satisfy; an abnormal IR or DSL input can trigger them:

| Site | Condition | Failure mode |
|---|---|---|
| `action.rs:96` | `CreateLocation` owner name not found | `.expect("Failed to resolve owner to name")` |
| `action.rs:112` | `CreateCardOnLocation` location name not found | `.expect("Location not found")` |
| `action.rs:208` | `CycleAction` player expr fails to eval | `.expect("Failed to eval player")` |
| `action.rs:213` | `CycleAction` resolved player not in `players` | `.expect("Player not found")` |
| `action.rs:218` | `CycleAction` player not in `turn_order` | `.expect("Player not in turn order")` |
| `action.rs:332,342` | `execute_cardset_move` source/dest `eval_cardset` fails | `.expect("Failed to eval cardset")` / `"Failed to eval dest"` |
| `action.rs:347` | destination location index strictly greater than `locations.len()` | `panic!("Could not resolve a destination for move action")` |
| `action.rs:362` | destination index **equal to** `locations.len()` (slips past the `>` check at `action.rs:345`) | Rust index-out-of-bounds panic at `game_data.locations[dest_loc_idx].cards.push(…)` — note the guard at `:345` uses `>` not `>=`, a latent off-by-one; an agent fixing move validation must change it to `>=` |
| `game_data.rs:134` | `add_location` owner (non-Table) not in `players` | `.expect("Owner not found")` |
| `game_data.rs:192` | `next_player` found idx missing from `turn_order` | `.unwrap()` (see I-13 — safe given `resolve_turn`'s contract) |
| `query.rs:1590,1609` | `resolve_players`/`resolve_player_collection` player eval fails or name missing | `.expect("Failed to eval player")` / `.expect("Player not found")` |
| `query.rs:542,635,706,1618` | `IntCollection`/`TeamCollection`/`StringCollection` `AggregateMemory`, or `PlayerCollection::Aggregate` | `todo!(…)` — panics if a DSL program reaches these arms |

**Silent no-ops** (neither error nor panic — agents must know these exist and do nothing):

- `FlipAction` (`action.rs:161-164`) — payload fields ignored entirely.
- `ShuffleAction` when `eval_cardset` fails (`action.rs:175-178`) — prints `eprintln!("ShuffleAction
  failed: {e}")` and continues; the pile is left unshuffled.
- `BidAction`, `BidMemoryAction`, `DemandAction`, `DemandMemoryAction`, `EndAction::GameWithWinner`,
  all `ScoringRule` variants, `CreateTokenOnLocation`, `MoveType::Place` (`action.rs:121,221-251,
  239-241,243-251,259-277,320`) — `// TODO` no-ops.
- `Payload::Trigger` traversal: `step` advances the state (`execute_edge`) but `action::execute`'s
  catch-all `_ => {}` (`action.rs:58`) performs no mutation.
- `PlayerCollection::AggregateMemory`, `PlayerCollection::Memory` (`query.rs:1650-1654`) — return
  `vec![]` silently.
- Out-of-range `Choice`/`Optional` input (I-8) — silent stall, no error.

### 5.3 Observability

The engine has **no `tracing`/`log` integration** (verified: zero imports of either in
`crates/engine/src`). Telemetry is provided by two mechanisms:

1. **Reactive event callback** (`event_sender`, §4.3). Invoked by `Controller::emit_event`
   (`controller.rs:161-165`) at the top of every loop iteration *and* once more just before
   returning `GameOver` (`controller.rs:64,73`). The callback receives `&GameData`, so a host can
   render or snapshot after every single transition without polling. This is the recommended
   observability seam for production hosts.

2. **`debug.rs` — structured human-readable dumps.** Three verbosity levels selectable via
   `DebugLevel` (`debug.rs:6-30`):

   ```rust
   pub enum DebugLevel { Low, Medium, High }
   impl DebugLevel {
       pub fn from_marker(s: &str) -> Option<Self>;   // parses "<!--LOW-->" etc., case-insensitive
       pub fn marker(&self) -> &'static str;          // inverse
   }
   pub fn format_game_data(data: &GameData, level: DebugLevel) -> String;
   pub fn print_game_data(data: &GameData, level: DebugLevel);
   pub fn save_game_data(data: &GameData, path: &Path) -> io::Result<()>;
   ```

   | Level | Includes |
   |---|---|
   | `Low` (`debug.rs:40-71`) | Player names, current player, current stage, turn-order indices, card count per location. |
   | `Medium` (`debug.rs:73-157`) | All of Low + per-player scores, teams, all memories, per-location card names (first 5 if >5). |
   | `High` (`debug.rs:159-249`) | Full dump: every player with `in_stage` map, teams with raw indices, all locations with raw card-id vecs, every card's full attribute map, stage stack, all stage counters, all memories (typed), combos, precedences, point maps. |

   `save_game_data` (`debug.rs:255-271`) **appends** to `path` (creating it if absent) and
   **persists the `DebugLevel`** as an HTML comment marker on the file's first line: on subsequent
   calls it reads the existing first line via `DebugLevel::from_marker` and reuses that level
   (defaulting to `Medium` if absent/unparseable). This round-trip (`marker` ↔ `from_marker`,
   tested at `debug.rs:278-315`) lets a log file self-describe its verbosity.

3. **Ad-hoc stderr** — the only direct stdlib logging in the crate is
   `eprintln!("ShuffleAction failed: {e}")` (`action.rs:176`). There are no `dbg!`/`println!` calls
   in the library target (the `println!`s in `src/bin/cgdsl-play.rs` are CLI output, not engine
   telemetry).

**Agent guidance for adding telemetry:** prefer extending `debug.rs`'s level-gated formatter or
emitting from the `event_sender` callback rather than sprinkling `println!`/`eprintln!` through
`action.rs`/`query.rs`/`interpreter.rs`. If structured logging is desired, `tracing` would need to
be added to `Cargo.toml` (it is not currently a dependency) and spans established around
`step()`/`execute()`/`eval_*`; do not assume spans already exist.
