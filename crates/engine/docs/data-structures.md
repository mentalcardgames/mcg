---
type: agent_wiki_node
module: crates::engine
scope: [engine::game_data, engine::interpreter, engine::controller, front_end::ir]
topics: [data-model, state-store, types, ir, memory]
associated_files:
  - crates/engine/src/game_data.rs
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/interpreter/types.rs
  - crates/engine/src/interpreter/trace.rs
  - crates/engine/src/interpreter/ir_ext.rs
  - crates/engine/src/controller/mod.rs
  - crates/front_end/src/ir.rs
last_validated: 2026-08-09
---

# Data Structures & State Model

Every runtime value in `crates::engine` lives in or is referenced from the flat aggregate
`crates::engine::game_data::GameData`. This page documents (1) the `GameData` family, (2) the
`front_end::ir` types the engine consumes, and (3) the execution-time types (`Interpreter`,
`Controller`, `Input`, `StepResult`, `InputType`, plus the post-Stage-5 trace subsystem
`TraceEntry`/`TraceEvent` and the `IrExt` trait). For *how* they are sequenced at runtime, see
[`lifecycle.md`](./lifecycle.md); for *what must not be violated*, see
[`invariants.md`](./invariants.md).

---

## 1. The State Store: `GameData` and its Family

All runtime state lives in a single aggregate, `crates::engine::game_data::GameData`
(`crates/engine/src/game_data.rs:24`). It derives only `Clone` (no `Debug`, no `Serialize` —
serialization is handled separately by `crates/engine/src/debug/mod.rs`).

```rust
// crates/engine/src/game_data.rs:22
pub type Card = HashMap<String, String>;

// crates/engine/src/game_data.rs:30-35  (added 2026-08, reserved for card encryption)
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum CardStatus { FaceUp, FaceDown, Private }

// crates/engine/src/game_data.rs:37-56
#[derive(Clone)]
pub struct GameData {
    pub table: OwnerData,
    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    pub turn_order: Vec<usize>,
    pub locations: Vec<Location>,
    pub cards: Vec<Card>,
    pub card_statuses: Vec<CardStatus>,
    pub combos: Vec<Combo>,
    pub precedences: Vec<Precedence>,
    pub point_maps: Vec<PointMap>,
    pub current_player: Option<usize>,
    pub stage_counters: HashMap<String, u32>,
    pub stage_stack: Vec<String>,
    pub memories: HashMap<String, MemoryValue>,
}
```

A `crates::engine::game_data::Card` is deliberately schemaless — a bag of string key/value
attributes (e.g. `Rank → Ace`, `Suite → Hearts`). Cards are stored **only** in
`crates::engine::game_data::GameData::cards: Vec<Card>` and referenced elsewhere by `usize` index
(a "card id"). Locations hold card ids, not cards.

`crates::engine::game_data::CardStatus` (`game_data.rs:30-35`) is a per-card visibility slot,
stored **parallel to `cards`** in `GameData::card_statuses` (same indexing). It was added 2026-08
and is currently **unused by the engine**: every card is created `FaceUp`, and `card_status` /
`set_card_status` (`game_data.rs:182-192`) are the only accessors. It is reserved for the card
encryption work — `FlipAction` should become (de)encrypting a card's face (see
`engine-vs-design.md` §1b).

| Struct | Location | Fields | Role |
|---|---|---|---|
| `crates::engine::game_data::OwnerData` | `crates/engine/src/game_data.rs:67-71` | `locations: Vec<usize>` | Ownership of location indices; held by both `GameData::table` and each `Player`. |
| `crates::engine::game_data::Player` | `crates/engine/src/game_data.rs:73-80` | `name, score: i32, owner: OwnerData, in_game: bool, in_stage: HashMap<String,bool>` | Per-player state; `in_stage` tracks participation per named stage. |
| `crates::engine::game_data::Location` | `crates/engine/src/game_data.rs:82-86` | `name: String, cards: Vec<usize>` | A named pile; `cards` is an ordered list of card ids. |
| `crates::engine::game_data::Team` | `crates/engine/src/game_data.rs:88-92` | `name, players: Vec<usize>` | Named group of player indices. |
| `crates::engine::game_data::Combo` | `crates/engine/src/game_data.rs:94-98` | `name: String, filter: front_end::ast::FilterExpr` | A named, reusable card filter (from `front_end::ast`). |
| `crates::engine::game_data::Precedence` | `crates/engine/src/game_data.rs:101-107` | `name, key: String, values: Vec<String>` | Ordered values on one key, low→high. Used by `Adjacent`/`Higher`/`Lower`/`ExtremaPrecedence`. |
| `crates::engine::game_data::PointMap` | `crates/engine/src/game_data.rs:109-114` | `name, map: HashMap<String,i32>` | Maps `"key:value"` → points. Used by `SumOfCardSet`, `ExtremaCardset`, `ExtremaPointMap`. |

```rust
// crates/engine/src/game_data.rs:55-65
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

`crates::engine::game_data::MemoryValue` is the dynamically-typed storage for DSL "memory"
variables. There is **no** separate `TeamCollection` variant — a stored team collection is
represented as `MemoryValue::Team(String)` holding one team name (the read sites are
`crates/engine/src/query/int.rs:277` and `crates/engine/src/query/player.rs:199`), and
`front_end::ast::MemoryType::TeamCollection` initializes to `MemoryValue::Int(0)`
(`crates/engine/src/game_data.rs:304-319`, inside `GameData::add_memory`'s match), a known
mismatch documented as invariant I-10 in [`invariants.md`](./invariants.md). The
`MemoryValue::CardSet` variant is also used by the quantifier subsystem to carry player-chosen
card ids — see the `SYNTH_MEMORY_KEY` discussion in [`observability.md`](./observability.md) and
invariant I-18 in [`invariants.md`](./invariants.md).

---

## 2. The IR the Engine Consumes (defined in `front_end::ir`)

The engine does not own its IR types; it parameterizes over `front_end::ir`:

```rust
// crates/front_end/src/ir.rs:42-43
pub struct StateID(u32);            // Copy, Eq+Hash+Ord, Serialize/Deserialize

// crates/front_end/src/ir.rs:51-60
pub struct Edge<T: serde::Serialize> { pub to: StateID, pub payload: T, pub meta: Option<Vec<Meta>> }

// crates/front_end/src/ir.rs:77-83
pub struct Ir<T: serde::Serialize> { pub states: HashMap<StateID, Vec<Edge<T>>>, pub entry: StateID, pub goal: StateID }
```

`front_end::ir::Payload<Ctx>` is the sum of transition kinds
(`crates/front_end/src/ir.rs:248-268`):

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

The engine operates on the *lowered* specialization (`crates/front_end/src/ir.rs:313`):

```rust
pub type LoweredPayLoad = Payload<LoweredCtx>;
// where LoweredCtx resolves: Condition→BoolExpr, EndCondition→EndCondition,
//   GameRule→GameRule, Id→String.
```

So at the engine boundary,
`front_end::ir::Ir<front_end::ir::LoweredPayLoad>` is
`HashMap<front_end::ir::StateID, Vec<front_end::ir::Edge<front_end::ir::LoweredPayLoad>>>`
plus `entry`/`goal`. The `front_end::ir::Edge::meta` field is **ignored** by the engine — it is read
only by `front_end::fsm_to_dot`.

> **Engine-supplied `Ir` extension trait** (post-Stage-5): the engine defines
> `crates::engine::interpreter::IrExt` (`crates/engine/src/interpreter/ir_ext.rs:3-26`), a `pub`
> trait `impl`'d for `Ir<LoweredPayLoad>`:
> ```rust
> pub trait IrExt {
>     fn edge_labels(&self, state: StateID) -> Vec<String>;
> }
> ```
> USED by `Interpreter::step`'s `Payload::Choice` arm (`interpreter/mod.rs:174`) to derive the
> human-readable label of each outgoing edge from the next state's first edge's payload. `IrExt` is
> re-exported at the crate root (`crates/engine/src/lib.rs:12`).

---

## 3. The Execution Types

### 3.1 `Interpreter` — the running FSM state

```rust
// crates/engine/src/interpreter/mod.rs:26-43
pub struct Interpreter {
    pub ir: Ir<LoweredPayLoad>,
    pub game_data: GameData,
    pub input_buffer: Vec<Input>,
    pub current_state: StateID,
    pub trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
    /// Ephemeral overlay of synthetic replacement edges, keyed only by synthetic
    /// `StateID`s allocated from `next_synth`. Real IR ids are never keys here.
    pub pending_overlay: HashMap<StateID, Vec<Edge<LoweredPayLoad>>>,
    /// Counter for synthetic `StateID` allocation. Seeded at `u32::MAX - 1`.
    pub next_synth: u32,
    /// In-flight quantifier awaiting a player-input round-trip, if any.
    pub pending_quant: Option<crate::quantifier::PendingQuant>,
}
```

Post-Stage-5 the `Interpreter` grew from 4 fields (`ir`, `game_data`, `input_buffer`,
`current_state`) to 8 by adding the trace-sender seam and the quantifier bookkeeping. A canonical
constructor now exists:

```rust
// crates/engine/src/interpreter/mod.rs:45-62
impl Interpreter {
    pub fn new(
        ir: Ir<LoweredPayLoad>,
        game_data: GameData,
        trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
    ) -> Self { /* seeds current_state = ir.entry, next_synth = u32::MAX - 1, etc. */ }
}
```

All fields remain `pub` (the struct is constructible by hand and this is a supported pattern), but
`Interpreter::new` is the canonical entry point: it is the only place that initialises the
quantifier bookkeeping correctly (`pending_overlay = HashMap::new()`, `next_synth = u32::MAX - 1`,
`pending_quant = None`, `input_buffer = Vec::new()`, `current_state = ir.entry`). Direct
construction that omits these inits will misbehave on the first quantifier edge. See invariant I-16
in [`invariants.md`](./invariants.md) for the `next_synth` seeding rationale.

### 3.2 `Input`, `InputKind`, `StepResult`, `InputType` — the I/O contract

```rust
// crates/engine/src/interpreter/types.rs:1-16  (input from host → interpreter)
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub player_id: String,    // "P1", "P2" — who submitted this
    pub kind: InputKind,      // what they chose
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
    Choice { idx: usize },
    OptionalAccept,
    OptionalDecline,
    ChoosePlayer { idx: usize },
    ChooseCards { selected: Vec<usize> },
}

// Accessors delegate: Input::idx() → self.kind.idx(), etc.
// crates/engine/src/interpreter/types.rs:49-55  (step outcome)
pub enum StepResult { Ok, NeedsInput(InputType), GameOver, Error(String) }

// crates/engine/src/interpreter/types.rs:57-78  (prompt the host must answer)
#[derive(Clone, Debug)]
pub enum InputType {
    Choice { options: Vec<String>, max_index: usize },
    Optional(String),
    ChoosePlayer { candidates: Vec<String>, prompt: String },       // post-Stage-5
    ChooseCards { display: Vec<Card>, min: usize, max: usize, prompt: String }, // post-Stage-5
}
```

`Input` now carries three accessor methods that delegate to `InputKind`
(`crates/engine/src/interpreter/types.rs:22-35`):

- `pub fn idx(&self) -> usize` — delegates to `self.kind.idx()`: `Choice{idx}→idx`, `OptionalAccept→0`, `OptionalDecline→1`; returns `0` for other variants.
- `pub fn player_idx(&self) -> Option<usize>` — `Some(idx)` if this is a `ChoosePlayer`, else `None`.
- `pub fn card_selection(&self) -> Option<&[usize]>` — `Some(&selected)` if this is a `ChooseCards`, else `None`.

`InputKind` methods live at `crates/engine/src/interpreter/types.rs:38-69`; `Input` delegates at `types.rs:22-35`.

### 3.3 `TraceEntry` / `TraceEvent` — the per-step trace seam (post-Stage-5)

```rust
// crates/engine/src/interpreter/trace.rs:1-8
pub enum TraceEntry {
    Step { from: u32, to: u32, event: TraceEvent },
}

// crates/engine/src/interpreter/trace.rs:14-67
pub enum TraceEvent {
    Action { rule: GameRule },
    Choice { chosen_idx: usize, options: Vec<String> },
    OptionalAccept,
    OptionalDecline,
    Condition { expr: BoolExpr, result: bool, negated: bool, took_else: bool },
    EndCondition { expr: EndCondition, result: bool, stage: String, exited: bool },
    StageRoundCounter { stage: String, new_count: u32 },
    EndStage { stage: String },
    Trigger,
    Quantifier { kind: String, detail: String },     // post-Stage-5
}
```

`from`/`to` are **raw** `StateID` integers (via `StateID::raw()`). Both enums derive `Clone +
Debug`; both also implement `std::fmt::Display` (`trace.rs:48-102`) producing human-readable lines
suitable for `mcg-trace.log` (e.g. `[12->13] Action:Move deal 26 from Deck private to P1Pile`). The
`Quantifier` variant is emitted by the quantifier driver at synthetic-state allocation/deallocation
(`interpreter/quant_driver.rs:363-375`). See [`observability.md`](./observability.md) for how
`run_game` threads the sender into the `Interpreter`.

### 3.4 `InputSource` and the internal `Controller`

```rust
// crates/engine/src/controller/mod.rs:22-25
pub enum InputSource {
    Player(Box<dyn Fn(InputType) -> Input + Send + Sync>),
    TestFile(PathBuf),
}

// crates/engine/src/controller/mod.rs:137-146  (internal — NOT re-exported)
struct Controller {
    interpreter: Interpreter,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    line_buffer: VecDeque<String>,
    file_loaded: bool,
    loaded_line_count: usize,
    input_sequence: usize,
    step_count: Arc<std::sync::Mutex<usize>>,           // post-Stage-5
}
```

`Controller` is the (still-private) run-loop owner. Post-Stage-5 it gained a `step_count:
Arc<Mutex<usize>>` field shared with the composed trace-sender closure (see §6.1 of
[`interfaces.md`](./interfaces.md) and [`observability.md`](./observability.md)).

### 3.5 `Evaluator` — the read-side namespace

`crates::engine::query::Evaluator` (`crates/engine/src/query/mod.rs:176`) is a zero-sized `pub
struct` used purely as a namespace for associated functions (`eval_bool`, `eval_int`,
`eval_string`, `eval_player`, `eval_team`, `eval_cardset`, `eval_card_position`,
`eval_end_condition`, `eval_compare`, `eval_int_compare`, `resolve_players`,
`resolve_player_collection`, `resolve_multi_owner_names`, `resolve_owner_to_name`,
`resolve_owner_to_names`, `resolve_quantity`, `expand_types`, `check_attr_value_in_cardset`). It
holds **no state**; every method takes `&GameData` (all reads are immutable). Post-Stage-5 the
query module was split into submodules (`bool.rs`, `cardset.rs`, `int.rs`, `player.rs`,
`string.rs`) all hanging methods off the shared `Evaluator` struct.

**Fallibility (2026-08):** `resolve_players` and `resolve_player_collection` return
`Result<Vec<usize>, String>` — player-expressions that cannot be evaluated (e.g. `next` with no
eligible player) or that reference unknown players yield `Err` instead of panicking.
`resolve_quantity` takes `&GameData` and evaluates runtime int expressions against the live
state. `resolve_owner_to_names` (plural) routes `Owner::PlayerCollection` through
`Evaluator::resolve_player_collection` so it supports the `Aggregate { Quantifier::All }`
owner that the setup path produces. The former `todo!()`/silent-empty collection-memory arms
(`PlayerCollection::Aggregate`/`AggregateMemory`/`Memory`, `IntCollection::AggregateMemory`,
`TeamCollection::AggregateMemory`, `StringCollection::AggregateMemory`) are implemented; see
`engine-vs-design.md` F-9/F-13.
