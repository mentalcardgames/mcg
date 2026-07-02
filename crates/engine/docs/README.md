---
type: agent_wiki_node
module: crates::engine
scope: [all — crate-level overview]
topics: [overview, architecture, module-map, domain-concepts, entry-point]
associated_files:
  - crates/engine/src/lib.rs
  - crates/engine/Cargo.toml
  - crates/engine/src/bin/cgdsl-play.rs
last_validated: 2026-07-02
---

# `crates::engine` — Agent Wiki Hub

> Crate name (Cargo): `cgdsl-engine` · Binary: `cgdsl-play` (`crates/engine/src/bin/cgdsl-play.rs`)
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
   `crates::engine::interpreter::Interpreter::step` (`crates/engine/src/interpreter.rs`).
2. **State mutation** — translate each traversed edge's `front_end::ir::Payload` into concrete
   writes on `crates::engine::game_data::GameData` (players, locations, cards, memories, stage
   counters, turn order) (`crates/engine/src/action.rs`).
3. **State query / expression evaluation** — evaluate the DSL's expression sub-language
   (`front_end::ast::BoolExpr`, `front_end::ast::IntExpr`, `front_end::ast::CardSet`,
   `front_end::ast::PlayerExpr`, …) over the live `crates::engine::game_data::GameData` to resolve
   conditions, end-conditions, card sets, quantities, and ownership (`crates/engine/src/query.rs`).

A fourth concern, **I/O orchestration**, is handled by `crates/engine/src/controller.rs`, which
owns the main loop and feeds external input (human or recorded) into the interpreter whenever a
transition stalls on `crates::engine::interpreter::StepResult::NeedsInput`.

---

## Architectural Pattern

The engine is a **finite-state-machine (FSM) interpreter layered over a data-oriented state store**,
combined with a **visitor/evaluator** over an external expression-tree. It is *not* ECS, *not*
actor-model, *not* async. Three cooperating patterns compose it:

| Layer | Pattern | Implementation |
|---|---|---|
| Orchestration | **Pull-driven event loop** with an external input source | `crates::engine::controller::Controller::run` (`crates/engine/src/controller.rs:62`) |
| Transition | **Single-step FSM interpreter** dispatching on edge payload | `crates::engine::interpreter::Interpreter::step` (`crates/engine/src/interpreter.rs:23`) |
| Write side | **Payload → mutation dispatch** (one arm per `front_end::ir::Payload`/`front_end::ast::GameRule` variant) | `crates::engine::action::execute` (`crates/engine/src/action.rs:45`) |
| Read side | **Recursive-descent evaluator** (stateless, associated functions) | `crates::engine::query::Evaluator` (`crates/engine/src/query.rs:177`) |
| State | **Flat, mutable aggregate** (`crates::engine::game_data::GameData`) with index-based references | `crates/engine/src/game_data.rs:24` |

The control flow is strictly synchronous and single-threaded: `crates::engine::controller::Controller::run`
loops calling `crates::engine::interpreter::Interpreter::step`; each `step()` may synchronously call
into `crates::engine::query::Evaluator` (reads) and `crates::engine::action::execute` (writes); when
`step()` returns `crates::engine::interpreter::StepResult::NeedsInput`, `run()` consults the
`crates::engine::controller::InputSource` and pushes one `crates::engine::interpreter::Input` back
before the next iteration. There is no scheduler, no green threads, no `Future`s. (See
[`concurrency.md`](./concurrency.md).)

---

## Module Hierarchy & Dependency Flow

All six modules are declared public in `crates/engine/src/lib.rs:1-6` and re-exported at the crate
root (`crates/engine/src/lib.rs:8-11`). Internal dependency flow (arrow = "uses"):

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

- **Public-facing API surface** (the contract external crates may rely on):
  `crates::engine::controller::run_game`, `crates::engine::controller::InputSource`
  (`crates/engine/src/controller.rs`); `crates::engine::interpreter::Interpreter`,
  `crates::engine::interpreter::Input`, `crates::engine::interpreter::InputType`,
  `crates::engine::interpreter::StepResult` (`crates/engine/src/interpreter.rs`); the
  `crates::engine::game_data::GameData` family of structs and the `crates::engine::game_data::Card`
  type alias (`crates/engine/src/game_data.rs`); `crates::engine::query::Evaluator` and its `pub`
  methods (`crates/engine/src/query.rs`); `crates::engine::debug::DebugLevel`,
  `crates::engine::debug::format_game_data`, `crates::engine::debug::print_game_data`,
  `crates::engine::debug::save_game_data` (`crates/engine/src/debug.rs`).
- **Internal-only**: `crates::engine::controller::Controller` (`crates/engine/src/controller.rs:49`)
  is a `struct` (not `pub`); `crates::engine::action::execute` and all `execute_*` helpers are
  `pub fn` but are intended for the interpreter's use (no `pub use` re-export of `action` symbols at
  crate root beyond the module itself); `crates/engine/src/query.rs`'s private helpers
  (`eval_int_collection`, `eval_group`, `apply_filter`, `card_matches_filter`,
  `infer_location_from_cards`, …) are module-private.

The binary `cgdsl-play` (`crates/engine/src/bin/cgdsl-play.rs`) is a thin CLI driver that wires
`front_end::validation::parse_document` → `front_end::ast::SGame::to_lowered_graph` →
`crates::engine::controller::run_game`; it is **not** part of the library target.

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
- **Event sender** — optional `Box<dyn Fn(&GameData) + Send>` callback invoked after every step;
  the engine's reactive observability seam. See [`observability.md`](./observability.md).

---

## Table of Contents

| Page | Covers | When to read it |
|---|---|---|
| [`data-structures.md`](./data-structures.md) | `GameData` family, consumed IR types, execution types | You need field-level layout of any struct/enum. |
| [`lifecycle.md`](./lifecycle.md) | Construction → setup → play loop → termination; run-loop sequencing | You need to understand *when* things happen. |
| [`invariants.md`](./invariants.md) | 15 hard guardrails (I-1 … I-15) that must never be broken | **Before modifying any engine code.** |
| [`concurrency.md`](./concurrency.md) | Threading model, `Send`/`Sync`, resources, unused deps | You need thread-safety / memory guarantees. |
| [`api-usage.md`](./api-usage.md) | Golden path, manual interpreter driving, extension points | You are integrating the engine from outside. |
| [`error-handling.md`](./error-handling.md) | Error channels, recoverable vs panic, silent no-ops | You are diagnosing a failure or adding a path. |
| [`observability.md`](./observability.md) | `event_sender` callback, `debug.rs` `DebugLevel` dumps | You need telemetry / debugging output. |

> The original monolithic source for this wiki is preserved at
> `crates/engine/ARCHITECTURE.md` for archival diffing; this hub-and-spoke set is the
> authoritative, retrieval-optimized form.
