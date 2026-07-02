---
type: agent_wiki_node
module: crates::engine
scope: [engine::interpreter, engine::action, engine::query, engine::controller, engine::game_data]
topics: [error-handling, panics, result, recoverability, silent-noops]
associated_files:
  - crates/engine/src/interpreter.rs
  - crates/engine/src/action.rs
  - crates/engine/src/query.rs
  - crates/engine/src/controller.rs
  - crates/engine/src/game_data.rs
last_validated: 2026-07-02
---

# Error Handling, Panic Conditions & Diagnostics

The engine is **stringly-typed throughout** — there is no custom `Error` enum despite `thiserror`
being declared in `Cargo.toml` (see [`concurrency.md`](./concurrency.md) §4). This page enumerates
the error channels, the recoverable vs. panic/silent paths, and points to
[`observability.md`](./observability.md) for telemetry. Several panic sites enforce invariants from
[`invariants.md`](./invariants.md); cross-references use the `I-N` IDs.

---

## 1. Error Types

Three error channels exist:

| Channel | Type | Origin |
|---|---|---|
| Run failure | `Result<GameData, String>` from `crates::engine::controller::run_game` (`crates/engine/src/controller.rs:29`) | Propagated from `crates::engine::interpreter::StepResult::Error` or `Controller::get_input`. |
| Step failure | `crates::engine::interpreter::StepResult::Error(String)` (`crates/engine/src/interpreter.rs:177`) | Missing state, bad edge counts, evaluator errors (below). |
| Eval failure | `Result<_, String>` from every `crates::engine::query::Evaluator` method | Division by zero, missing memory/location/precedence/pointmap/combo, type-mismatched memory, out-of-range index, "no current player/stage", etc. |

Representative `crates::engine::query::Evaluator` error strings (verbatim from
`crates/engine/src/query.rs`): `"Division by zero"` (`crates/engine/src/query.rs:387`),
`"No current stage"` (`crates/engine/src/query.rs:508`), `"Memory {key} not found"`, `"Memory
value is not an Int"` (and `String`/`Team`/`CardSet`/… variants), `"Location {name} not found"`,
`"PointMap {name} not found"`, `"Precedence {name} not found"`, `"Combo {name} not found"`,
`"No card at index {idx} in location {loc}"`, `"No card at top of location {loc}"`,
`"No next player available"`, `"No competitor found"`, `"Owner of card position not found"`,
`"No card found for extrema"`, `"resolve_owner_to_name: PlayerCollection cannot resolve to a single
name"`, `"Card position not found in any location"`.

Controller-level errors: `"Failed to open test file: {e}"`, `"Failed to read test file: {e}"`
(`crates/engine/src/controller.rs:118,121`), `"Test input file exhausted"`
(`crates/engine/src/controller.rs:134`), `"Invalid test input #{n}: expected number, 'y', or 'n',
got '{line}'"` (`crates/engine/src/controller.rs:140-145`), `"Invalid test input #{n}: choice
indices start at 1, got 0"` (`crates/engine/src/controller.rs:146-151`).

Interpreter-level errors: `"Current state not found in IR"`
(`crates/engine/src/interpreter.rs:26`), `"No outgoing edges and not at goal state"`
(`crates/engine/src/interpreter.rs:33`), `"Condition state must have exactly 2 edges"`
(`crates/engine/src/interpreter.rs:73`), `"EndCondition state must have exactly 2 edges"`
(`crates/engine/src/interpreter.rs:100`), `"Failed to get condition edge"` / `"Failed to get end
condition edge"` (`crates/engine/src/interpreter.rs:91,122`), `"No edges found"`
(`crates/engine/src/interpreter.rs:141`).

---

## 2. Recoverable vs. Unrecoverable Paths

**Recoverable** (surfaced as `Err(String)` / `crates::engine::interpreter::StepResult::Error`): all
`crates::engine::query::Evaluator` `Result` returns; condition/end-condition edge-count violations;
missing current state in the IR; dead-end non-goal states; test-file open/parse/exhaustion errors.
These terminate `crates::engine::controller::run_game` with `Err` and leave
`crates::engine::game_data::GameData` in whatever partially-mutated state it reached (the engine
does **not** roll back applied mutations on error —
`crates::engine::interpreter::Interpreter::execute_edge` has already written before a later
evaluator call can fail).

**Unrecoverable** (process-aborting `panic!` / `.expect()` / `.unwrap()` / `todo!`). These are
invariants the code assumes the IR/data will satisfy; an abnormal IR or DSL input can trigger them:

| Site | Condition | Failure mode |
|---|---|---|
| `crates/engine/src/action.rs:96` | `front_end::ast::SetUpRule::CreateLocation` owner name not found | `.expect("Failed to resolve owner to name")` |
| `crates/engine/src/action.rs:112` | `front_end::ast::SetUpRule::CreateCardOnLocation` location name not found | `.expect("Location not found")` |
| `crates/engine/src/action.rs:208` | `front_end::ast::ActionRule::CycleAction` player expr fails to eval | `.expect("Failed to eval player")` |
| `crates/engine/src/action.rs:213` | `CycleAction` resolved player not in `players` | `.expect("Player not found")` |
| `crates/engine/src/action.rs:218` | `CycleAction` player not in `turn_order` | `.expect("Player not in turn order")` |
| `crates/engine/src/action.rs:332,342` | `crates::engine::action::execute_cardset_move` source/dest `eval_cardset` fails | `.expect("Failed to eval cardset")` / `"Failed to eval dest"` |
| `crates/engine/src/action.rs:347` | destination location index strictly greater than `locations.len()` | `panic!("Could not resolve a destination for move action")` |
| `crates/engine/src/action.rs:362` | destination index **equal to** `locations.len()` (slips past the `>` check at `crates/engine/src/action.rs:345`) | Rust index-out-of-bounds panic at `game_data.locations[dest_loc_idx].cards.push(…)` — note the guard at `:345` uses `>` not `>=`, a latent off-by-one; an agent fixing move validation must change it to `>=` |
| `crates/engine/src/game_data.rs:134` | `crates::engine::game_data::GameData::add_location` owner (non-Table) not in `players` | `.expect("Owner not found")` |
| `crates/engine/src/game_data.rs:192` | `crates::engine::game_data::GameData::next_player` found idx missing from `turn_order` | `.unwrap()` (see I-13 — safe given `resolve_turn`'s contract) |
| `crates/engine/src/query.rs:1590,1609` | `crates::engine::query::Evaluator::resolve_players`/`resolve_player_collection` player eval fails or name missing | `.expect("Failed to eval player")` / `.expect("Player not found")` |
| `crates/engine/src/query.rs:542,635,706,1618` | `IntCollection`/`TeamCollection`/`StringCollection` `AggregateMemory`, or `front_end::ast::PlayerCollection::Aggregate` | `todo!(…)` — panics if a DSL program reaches these arms |

**Silent no-ops** (neither error nor panic — agents must know these exist and do nothing):

- `front_end::ast::ActionRule::FlipAction` (`crates/engine/src/action.rs:161-164`) — payload fields
  ignored entirely.
- `front_end::ast::ActionRule::ShuffleAction` when `eval_cardset` fails
  (`crates/engine/src/action.rs:175-178`) — prints `eprintln!("ShuffleAction failed: {e}")` and
  continues; the pile is left unshuffled.
- `front_end::ast::ActionRule::BidAction`, `BidMemoryAction`, `DemandAction`, `DemandMemoryAction`,
  `front_end::ast::EndType::GameWithWinner`, all `front_end::ast::ScoringRule` variants,
  `front_end::ast::SetUpRule::CreateTokenOnLocation`, `front_end::ast::MoveType::Place`
  (`crates/engine/src/action.rs:121,221-251,239-241,243-251,259-277,320`) — `// TODO` no-ops.
- `front_end::ir::Payload::Trigger` traversal: `crates::engine::interpreter::Interpreter::step`
  advances the state (`execute_edge`) but `crates::engine::action::execute`'s catch-all
  `_ => {}` (`crates/engine/src/action.rs:58`) performs no mutation.
- `front_end::ast::PlayerCollection::AggregateMemory`, `front_end::ast::PlayerCollection::Memory`
  (`crates/engine/src/query.rs:1650-1654`) — return `vec![]` silently.
- Out-of-range `Choice`/`Optional` input (I-8) — silent stall, no error.
