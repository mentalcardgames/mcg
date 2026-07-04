---
type: agent_wiki_node
module: crates::engine
scope: [all — crate-level overview]
topics: [overview, architecture, module-map, domain-concepts, entry-point]
associated_files:
  - crates/engine/src/lib.rs
  - crates/engine/Cargo.toml
  - crates/engine/src/bin/cgdsl-play.rs
  - crates/engine/src/bin/engine-tui/main.rs
last_validated: 2026-07-04
---

# `crates::engine` — Agent Wiki Hub

> Crate name (Cargo): `cgdsl-engine` · Binaries: `engine-tui` (declared, `[[bin]]` at
> `crates/engine/Cargo.toml:8-10`, entry `crates/engine/src/bin/engine-tui/main.rs`) **and**
> `cgdsl-play` (auto-discovered by `rustc` from `crates/engine/src/bin/cgdsl-play.rs`)
> · Edition 2021 · Library target: `crates/engine/src/lib.rs`
> Scope: this wiki covers `crates/engine/**` exclusively. Types from the dependency crate
> `front_end` are referenced only where the engine's public contract depends on them.

This is the **Hub** of the `crates::engine` agent wiki. It holds the "why" and the module map; every
other page is a hyper-focused spoke linked from the [Table of Contents](#table-of-contents) below.

---

## High-Level Purpose

`crates::engine` is the **runtime execution kernel** for MCG's *Card Game DSL* (`cgdsl`). It does
not parse the DSL itself — that is the responsibility of the `front_end` crate. Instead, the engine
consumes a **lowered intermediate representation** (`front_end::ir::Ir<front_end::ir::LoweredPayLoad>`,
a finite-state machine) produced by `front_end` from `.cgdsl` source, and **drives that FSM to
completion** against an initially-empty `crates::engine::game_data::GameData`, mutating the game
state in lock-step with each FSM transition.

Concretely, the engine solves three problems:

1. **State-machine execution** — given the current `front_end::ir::StateID`, inspect the outgoing
   `front_end::ir::Edge`s and advance exactly one transition per
   `crates::engine::interpreter::Interpreter::step` (`crates/engine/src/interpreter/mod.rs`).
2. **State mutation** — translate each traversed edge's `front_end::ir::Payload` into concrete
   writes on `crates::engine::game_data::GameData` (players, locations, cards, memories, stage
   counters, turn order) (`crates/engine/src/action.rs`).
3. **State query / expression evaluation** — evaluate the DSL's expression sub-language
   (`front_end::ast::BoolExpr`, `front_end::ast::IntExpr`, `front_end::ast::CardSet`,
   `front_end::ast::PlayerExpr`, …) over the live `crates::engine::game_data::GameData` to resolve
   conditions, end-conditions, card sets, quantities, and ownership
   (`crates/engine/src/query/`).

A fourth concern, **I/O orchestration**, is handled by `crates/engine/src/controller/`, which
owns the main loop and feeds external input (human or recorded) into the interpreter whenever a
transition stalls on `crates::engine::interpreter::StepResult::NeedsInput`. A fifth concern,
**quantifier preprocessing** (`crates/engine/src/quantifier.rs`), rewrites quantifier-bearing
edges into concrete replacement edges before the mutation layer sees them.

---

## Architectural Pattern

The engine is a **finite-state-machine (FSM) interpreter layered over a data-oriented state store**,
combined with a **visitor/evaluator** over an external expression-tree and a **quantifier
preprocessor** that rewrites quantifier-bearing edges into concrete replacement edges. It is *not*
ECS, *not* actor-model, *not* async. Six cooperating layers compose it:

| Layer | Pattern | Implementation |
|---|---|---|
| Orchestration | **Pull-driven event loop** with an external input source, plus trace logging + panic capture | `crates::engine::controller::Controller::run` (`crates/engine/src/controller/mod.rs:151`) and `run_game` (`controller/mod.rs:31`) |
| Transition | **Single-step FSM interpreter** dispatching on edge payload, with overlay + quantifier resume arms | `crates::engine::interpreter::Interpreter::step` (`crates/engine/src/interpreter/mod.rs:64`) |
| Quantifier preprocessor | **Edge rewrite layer** that expands `Quantifier::All`/`Any` over dest players or card amounts into concrete replacement edges | `crates::engine::quantifier` (`crates/engine/src/quantifier.rs`) + `interpreter/quant_driver.rs` |
| Write side | **Payload → mutation dispatch** (one arm per `front_end::ir::Payload`/`front_end::ast::GameRule` variant) | `crates::engine::action::execute` (`crates/engine/src/action.rs:45`) |
| Read side | **Recursive-descent evaluator** (stateless, associated functions) | `crates::engine::query::Evaluator` (`crates/engine/src/query/mod.rs:173`) |
| State | **Flat, mutable aggregate** (`crates::engine::game_data::GameData`) with index-based references | `crates/engine/src/game_data.rs:24` |

The control flow is strictly synchronous and single-threaded (production logic):
`crates::engine::controller::Controller::run` loops calling
`crates::engine::interpreter::Interpreter::step`; each `step()` may synchronously call into
`crates::engine::quantifier::scan_edge` (quantifier preprocessing),
`crates::engine::query::Evaluator` (reads), and `crates::engine::action::execute` (writes); when
`step()` returns `crates::engine::interpreter::StepResult::NeedsInput`, `run()` consults the
`crates::engine::controller::InputSource` and pushes one `crates::engine::interpreter::Input` back
before the next iteration. There is no scheduler, no green threads, no `Future`s. The only
`std::sync::Arc<Mutex<…>>` usage lives in the trace-logging plumbing (the trace writer and the
shared step counter), and `run_game` wraps the run in `std::panic::catch_unwind` when a trace log
is open. See [`concurrency.md`](./concurrency.md).

---

## Module Hierarchy & Dependency Flow

All seven modules are declared public in `crates/engine/src/lib.rs:1-7` and re-exported at the
crate root (`crates/engine/src/lib.rs:9-13`). Internal dependency flow (arrow = "uses"):

```
                        front_end::ir  (Ir, Edge, StateID, Payload, LoweredPayLoad)
                        front_end::ast (GameRule, ActionRule, SetUpRule, MoveType, …)
                        front_end::validation::parse_document  (used only by the bins + tests)
                                  │
                                  ▼
  ┌─────────────┐    ┌──────────────────┐    ┌───────────────────┐
  │ controller  │──▶ │   interpreter    │──▶ │     action        │  (write side)
  │  (run_game, │    │  (step, edges,   │    │  (execute payload │
  │   InputSrc, │    │   StepResult,    │    │   → GameData mut) │
  │   events,   │    │   trace,         │    └─────────┬─────────┘
  │   trace log)│    │   quant_driver)  │              │
  └─────┬───────┘    └────────┬─────────┘              │
        │                     │ ▲                      ▼
        │                     │ │            ┌───────────────────┐
        │            ┌────────┴─┴───┐        │    game_data      │  (state store)
        │            │  quantifier  │        │  (GameData, Card, │
        │            │ (scan_edge,  │        │   Player, …)      │
        │            │  alloc_synth,│        └─────────▲─────────┘
        │            │  build_chain)│                  │
        │            └──────┬───────┘                  │
        │                   │                          │
        │            ┌──────▼──────────────────────────▼──────┐
        │            │     query (Evaluator: eval_*, resolve_*)│  (read side)
        │            └─────────────────────────────────────────┘
        ▼
 ┌─────────────┐
 │   debug     │  (format/print/save GameData; observability only)
 └─────────────┘
```

The `quantifier` module is a preprocessor that sits **between** `interpreter`'s `step()` and
`action::execute`: it inspects each edge for a quantifier site and, if found, rewrites it into
concrete replacement edges that flow through the *unchanged* `action::execute` path. See
[`lifecycle.md`](./lifecycle.md) §3 "Play Phase" pre-dispatch arms.

Public-facing vs. internal:

- **Public-facing API surface** (the contract external crates may rely on, all re-exported at the
  crate root by `crates/engine/src/lib.rs:9-13`):
  - `crates::engine::controller::{run_game, InputSource}` (`crates/engine/src/controller/mod.rs`);
    `Controller` itself is a private struct.
  - `crates::engine::interpreter::{Interpreter, Interpreter::new, Input, InputType, StepResult,
    IrExt, TraceEntry, TraceEvent}` (`crates/engine/src/interpreter/mod.rs` and siblings).
  - `crates::engine::game_data::{GameData, Card, Combo, Location, OwnerData, Player, PointMap,
    Precedence}` family (`crates/engine/src/game_data.rs`).
  - `crates::engine::query::Evaluator` and its `pub` methods
    (`crates/engine/src/query/mod.rs` + submodules).
  - `crates::engine::debug::{DebugLevel, format_game_data, print_game_data, save_game_data}`
    (`crates/engine/src/debug/mod.rs` + submodules).
  - `crates::engine::quantifier::{PendingKind, PendingQuant, QuantSite}` (plus the
    `SYNTH_MEMORY_KEY` constant) (`crates/engine/src/quantifier.rs`). These are re-exported because
    hosts that build an `Interpreter` by hand need to construct `PendingQuant` only in exotic
    cases; the more common consumer surface is `QuantSite` for diagnostics.
- **Internal-only**: `crates::engine::controller::Controller`
  (`crates/engine/src/controller/mod.rs:137`) is a `struct` (not `pub`);
  `crates::engine::action::execute` and all `execute_*` helpers are `pub fn` but are intended for
  the interpreter's use (no `pub use` re-export of `action` symbols at the crate root beyond the
  module itself); `crates/engine/src/query/`'s private helpers (`eval_int_collection`,
  `eval_group`, `apply_filter`, `card_matches_filter`, `infer_location_from_cards`, …) are
  module-private; `crates::engine::controller::trace_logger::TraceLogger` is `pub(super)` to the
  `controller` module; `crates::engine::interpreter::quant_driver` and `ir_ext::rule_signature` /
  `ir_ext::payload_label` are `pub(super)` to the `interpreter` module.

Two binaries ship from this crate:

- **`engine-tui`** — the declared `[[bin]]` (`crates/engine/Cargo.toml:8-10`), entry
  `crates/engine/src/bin/engine-tui/main.rs`. A ratatui/crossterm TUI for interactive testing; uses
  `crossbeam-channel` for its input loop. Run via `cargo run -p cgdsl-engine --bin engine-tui`.
- **`cgdsl-play`** — auto-discovered by `rustc` from `crates/engine/src/bin/cgdsl-play.rs` (no
  explicit `[[bin]]` entry). A thin CLI driver that wires `front_end::validation::parse_document`
  → `front_end::ast::SGame::to_lowered_graph` → `crates::engine::controller::run_game`. Run via
  `cargo run -p cgdsl-engine --bin cgdsl-play -- <game.cgdsl> [input.txt]`.

Neither is part of the library target.

---

## Core Domain Concepts

A compact glossary an agent should internalize before touching this crate:

- **FSM / IR** — `front_end::ir::Ir<front_end::ir::LoweredPayLoad>` is a directed graph:
  `HashMap<front_end::ir::StateID, Vec<front_end::ir::Edge<front_end::ir::LoweredPayLoad>>>` plus
  `entry`/`goal` `StateID`s. Each `Edge` carries a `Payload` describing what happens when you
  traverse it. The engine never constructs this graph; it only walks it.
- **`Payload`** — the sum type of transition kinds (`Condition`, `EndCondition`, `Action`,
  `StageRoundCounter`, `EndStage`, `Choice`, `Optional`, `Trigger`). See
  [`data-structures.md`](./data-structures.md) for the full enum.
- **`GameData`** — the single mutable aggregate holding all runtime state. Index-based, not
  reference-based: cards/players/locations are referred to by `usize` ids. See
  [`data-structures.md`](./data-structures.md).
- **`Card`** — a schemaless `HashMap<String, String>` of attributes (e.g. `Rank → Ace`), stored
  only in `crates::engine::game_data::GameData::cards` and referenced elsewhere by id.
- **Turn order / current player** — `crates::engine::game_data::GameData::turn_order` holds player
  indices; `crates::engine::game_data::GameData::current_player` is an index **into `turn_order`**,
  not into `players`. See invariant I-1 in [`invariants.md`](./invariants.md).
- **Stage stack** — `crates::engine::game_data::GameData::stage_stack` is a `Vec<String>` of nested
  stage names; `get_current_stage()` returns the top. `leave_stage` pops until (and including) the
  named stage (I-11).
- **Step** — one FSM transition. `crates::engine::interpreter::Interpreter::step` returns a
  `crates::engine::interpreter::StepResult` (`Ok` / `NeedsInput` / `GameOver` / `Error`). See
  [`lifecycle.md`](./lifecycle.md).
- **Evaluator** — `crates::engine::query::Evaluator`, a zero-sized struct used as a namespace for
  stateless read-only associated functions over `&GameData`.
- **Input source** — `crates::engine::controller::InputSource` (`Player` closure or `TestFile`),
  the single seam for external I/O. See [`api-usage.md`](./api-usage.md).
- **Event sender** — optional `Box<dyn Fn(&GameData) + Send>` callback invoked after every loop
  iteration; the engine's coarse-grained reactive observability seam. See
  [`observability.md`](./observability.md) §1.
- **Trace sender** — optional `Box<dyn Fn(TraceEntry) + Send>` callback invoked once per FSM
  *transition* with a `TraceEntry`; the engine's fine-grained structured-logging seam (post-Stage
  -5). See [`observability.md`](./observability.md) §2.
- **Quantifier site** — an edge whose `Payload::Action(GameRule::Action)` subtree contains a
  `Quantifier::All`/`Any` over a dest `PlayerCollection`, or an `Any`/`IntRange` `Quantity`. The
  preprocessor (`crates::engine::quantifier`) rewrites such edges into concrete replacement edges
  (fan-out for `All`, single substituted edge for `Any`) or issues a `ChoosePlayer`/`ChooseCards`
  prompt. See [`lifecycle.md`](./lifecycle.md) §3.
- **Synthetic `StateID`** — a `StateID` allocated by `crates::engine::quantifier::alloc_synth`
  from `u32::MAX - 1` decrementing (so it never collides with a densely-from-0-allocated real IR
  id). Replacement edges live in `Interpreter::pending_overlay` keyed by synthetic ids. See
  invariant I-16 in [`invariants.md`](./invariants.md).

---

## Table of Contents

| Page | Covers | When to read it |
|---|---|---|
| [`interfaces.md`](./interfaces.md) | Public API inventory, data flow, UI↔engine contract, observability seams | External host / UI author starting integration. **Read first if integrating from outside.** |
| [`data-structures.md`](./data-structures.md) | `GameData` family, consumed IR types, execution types, trace types, `IrExt` | You need field-level layout of any struct/enum. |
| [`lifecycle.md`](./lifecycle.md) | Construction → setup → play loop → termination; run-loop sequencing, quantifier pre-dispatch arms | You need to understand *when* things happen. |
| [`invariants.md`](./invariants.md) | 20 hard guardrails (I-1 … I-20) that must never be broken | **Before modifying any engine code.** |
| [`known-bugs.md`](./known-bugs.md) | 3 bugs in front_end/DSL that manifest at engine runtime (B-1 … B-3) | You are debugging a stage/turn/player-scope issue not explained by invariants. |
| [`concurrency.md`](./concurrency.md) | Threading model, `Send`/`Sync`, resources, unused deps | You need thread-safety / memory guarantees. |
| [`api-usage.md`](./api-usage.md) | Golden path, manual interpreter driving, extension points | You are integrating the engine from outside. |
| [`error-handling.md`](./error-handling.md) | Error channels, recoverable vs panic, silent no-ops, quantifier error strings | You are diagnosing a failure or adding a path. |
| [`observability.md`](./observability.md) | `event_sender` callback, `trace_sender` callback, `MCG_TRACE_LOG` file, `debug` `DebugLevel` dumps | You need telemetry / debugging output. |

> The original monolithic source for this wiki is preserved at
> `crates/engine/ARCHITECTURE.md` for archival diffing; this hub-and-spoke set is the
> authoritative, retrieval-optimized form.
