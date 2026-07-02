---
type: agent_wiki_node
module: crates::engine
scope: [engine::game_data, engine::interpreter, engine::controller, front_end::ir]
topics: [data-model, state-store, types, ir, memory]
associated_files:
  - crates/engine/src/game_data.rs
  - crates/engine/src/interpreter.rs
  - crates/engine/src/controller.rs
  - crates/front_end/src/ir.rs
last_validated: 2026-07-02
---

# Data Structures & State Model

Every runtime value in `crates::engine` lives in or is referenced from the flat aggregate
`crates::engine::game_data::GameData`. This page documents (1) the `GameData` family, (2) the
`front_end::ir` types the engine consumes, and (3) the execution-time types (`Interpreter`,
`Controller`, `Input`, `StepResult`, `InputType`). For *how* they are sequenced at runtime, see
[`lifecycle.md`](./lifecycle.md); for *what must not be violated*, see
[`invariants.md`](./invariants.md).

---

## 1. The State Store: `GameData` and its Family

All runtime state lives in a single aggregate, `crates::engine::game_data::GameData`
(`crates/engine/src/game_data.rs:24`). It derives only `Clone` (no `Debug`, no `Serialize` —
serialization is handled separately by `crates/engine/src/debug.rs`).

```rust
// crates/engine/src/game_data.rs:22
pub type Card = HashMap<String, String>;

// crates/engine/src/game_data.rs:24-39
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

A `crates::engine::game_data::Card` is deliberately schemaless — a bag of string key/value
attributes (e.g. `Rank → Ace`, `Suite → Hearts`). Cards are stored **only** in
`crates::engine::game_data::GameData::cards: Vec<Card>` and referenced elsewhere by `usize` index
(a "card id"). Locations hold card ids, not cards.

| Struct | Location | Fields | Role |
|---|---|---|---|
| `crates::engine::game_data::OwnerData` | `crates/engine/src/game_data.rs:54` | `locations: Vec<usize>` | Ownership of location indices; held by both `GameData::table` and each `Player`. |
| `crates::engine::game_data::Player` | `crates/engine/src/game_data.rs:60` | `name, score: i32, owner: OwnerData, in_game: bool, in_stage: HashMap<String,bool>` | Per-player state; `in_stage` tracks participation per named stage. |
| `crates::engine::game_data::Location` | `crates/engine/src/game_data.rs:69` | `name: String, cards: Vec<usize>` | A named pile; `cards` is an ordered list of card ids. |
| `crates::engine::game_data::Team` | `crates/engine/src/game_data.rs:75` | `name, players: Vec<usize>` | Named group of player indices. |
| `crates::engine::game_data::Combo` | `crates/engine/src/game_data.rs:81` | `name: String, filter: front_end::ast::FilterExpr` | A named, reusable card filter (from `front_end::ast`). |
| `crates::engine::game_data::Precedence` | `crates/engine/src/game_data.rs:88` | `name, key: String, values: Vec<String>` | Ordered values on one key, low→high. Used by `Adjacent`/`Higher`/`Lower`/`ExtremaPrecedence`. |
| `crates::engine::game_data::PointMap` | `crates/engine/src/game_data.rs:96` | `name, map: HashMap<String,i32>` | Maps `"key:value"` → points. Used by `SumOfCardSet`, `ExtremaCardset`, `ExtremaPointMap`. |

```rust
// crates/engine/src/game_data.rs:42-51
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
represented as `MemoryValue::Team(String)` holding one team name (see
`crates/engine/src/query.rs:637-647`), and `front_end::ast::MemoryType::TeamCollection` initializes
to `MemoryValue::Int(0)` (`crates/engine/src/game_data.rs:261`), a known mismatch documented as
invariant I-10 in [`invariants.md`](./invariants.md).

---

## 2. The IR the Engine Consumes (defined in `front_end::ir`)

The engine does not own its IR types; it parameterizes over `front_end::ir`:

```rust
// crates/front_end/src/ir.rs:43
pub struct StateID(u32);            // Copy, Eq+Hash+Ord

// crates/front_end/src/ir.rs:53-60
pub struct Edge<T: serde::Serialize> { pub to: StateID, pub payload: T, pub meta: Option<Vec<Meta>> }

// crates/front_end/src/ir.rs:79-83
pub struct Ir<T: serde::Serialize> { pub states: HashMap<StateID, Vec<Edge<T>>>, pub entry: StateID, pub goal: StateID }
```

`front_end::ir::Payload<Ctx>` is the sum of transition kinds
(`crates/front_end/src/ir.rs:252-268`):

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

The engine operates on the *lowered* specialization (`crates/front_end/src/ir.rs:313-322`):

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

---

## 3. The Execution Types

```rust
// crates/engine/src/interpreter.rs:15-20
pub struct Interpreter {
    pub ir: Ir<LoweredPayLoad>,
    pub game_data: GameData,
    pub input_buffer: Vec<Input>,
    pub current_state: StateID,
}

// crates/engine/src/interpreter.rs:156-161  (LIFO stack; see invariant I-7)
#[derive(Clone, Debug, PartialEq)]
pub enum Input { Choice { idx: usize }, OptionalAccept, OptionalDecline }

// crates/engine/src/interpreter.rs:173-178
pub enum StepResult { Ok, NeedsInput(InputType), GameOver, Error(String) }

// crates/engine/src/interpreter.rs:180-187
#[derive(Clone)]
pub enum InputType { Choice { options: Vec<String>, max_index: usize }, Optional(String) }
```

`crates::engine::interpreter::Input::idx` (`crates/engine/src/interpreter.rs:164-170`) normalizes
all three variants to a 0-based edge index: `Choice{idx}` → `idx`, `OptionalAccept` → `0`,
`OptionalDecline` → `1`.

```rust
// crates/engine/src/controller.rs:15-18
pub enum InputSource {
    Player(Box<dyn Fn(InputType) -> Input + Send + Sync>),
    TestFile(PathBuf),
}

// crates/engine/src/controller.rs:49-57  (internal — NOT re-exported)
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

`crates::engine::query::Evaluator` (`crates/engine/src/query.rs:177`) is a zero-sized `pub struct`
used purely as a namespace for associated functions (`eval_bool`, `eval_int`, `eval_string`,
`eval_player`, `eval_team`, `eval_cardset`, `eval_card_position`, `eval_end_condition`,
`eval_compare`, `eval_int_compare`, `resolve_players`, `resolve_player_collection`,
`resolve_owner_to_name`, `resolve_quantity`, `expand_types`, `check_attr_value_in_cardset`). It
holds **no state**; every method takes `&GameData` (or `&mut GameData` for none of them — all reads
are immutable).
