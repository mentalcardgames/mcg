---
type: agent_wiki_node
module: crates::engine
scope: [engine::interpreter, engine::interpreter::quant_driver, engine::action, engine::query, engine::controller, engine::game_data, engine::quantifier]
topics: [error-handling, panics, result, recoverability, silent-noops]
associated_files:
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/interpreter/quant_driver.rs
  - crates/engine/src/action.rs
  - crates/engine/src/query/mod.rs
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/game_data.rs
  - crates/engine/src/quantifier.rs
last_validated: 2026-08-09
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
| Run failure | `Result<GameData, String>` from `crates::engine::controller::run_game` (`crates/engine/src/controller/mod.rs:31`) | Propagated from `crates::engine::interpreter::StepResult::Error` or `Controller::get_input`. |
| Step failure | `crates::engine::interpreter::StepResult::Error(String)` (`crates/engine/src/interpreter/types.rs:54`) | Missing state, bad edge counts, evaluator errors, quantifier-resume validation errors (below). |
| Eval failure | `Result<_, String>` from every `crates::engine::query::Evaluator` method | Division by zero, missing memory/location/precedence/pointmap/combo, type-mismatched memory, out-of-range index, "no current player/stage", etc. |

Representative `crates::engine::query::Evaluator` error strings (verbatim from
`crates/engine/src/query/`): `"Division by zero"` (`crates/engine/src/query/int.rs:21`),
`"No current stage"` (`crates/engine/src/query/int.rs:142`, also
`crates/engine/src/query/player.rs:19`), `"Memory {key} not found"`, `"Memory value is not an
Int"` (and `String`/`Team`/`CardSet`/`PlayerCollection`/… variants), `"Location {name} not
found"`, `"PointMap {name} not found"`, `"Precedence {name} not found"`, `"Combo {name} not
found"`, `"No card at index {idx} in location {loc}"`, `"No card at top of location {loc}"`,
`"No next player available"`, `"No competitor found"`, `"Owner of card position not found"`,
`"No card found for extrema"`, `"resolve_owner_to_name: PlayerCollection cannot resolve to a
single name"` (`crates/engine/src/query/player.rs:291-294`),
`"resolve_owner_to_names: team '{name}' cannot own a location or memory (team-owned locations
are not in the data model)"` (`crates/engine/src/query/player.rs:310-312`),
`"Card position not found in any location"`.

Controller-level errors (in `crates/engine/src/controller/mod.rs`): `"Failed to open test file
{path}: {e}"` (`:206`), `"Failed to read test file {path}: {e}"` (`:210`), `"Test input file
exhausted (input #{})"` (`:223`), `"Invalid test input #{}: expected 'p <N>', got '{}'"
(`:229-233`), `"Invalid test input #{}: player indices start at 1, got 0"` (`:235-238`),
`"Invalid test input #{}: expected 'c <csv>', got '{}'"` (`:248-253`), `"Invalid test input #{}:
card indices start at 1, got 0"` (`:255-258`), `"Invalid test input #{}: expected number, 'y',
'n', 'p <N>', or 'c <csv>', got '{}'"` (`:269-274`), `"Invalid test input #{}: choice indices
start at 1, got 0"` (`:275-280`).

Interpreter-level errors (in `crates/engine/src/interpreter/mod.rs`): `"Current state {state} not
found in IR"` (`:113-117`), `"No outgoing edges from state {state} and not at goal state"`
(`:124-128`), `"Condition state {state} must have exactly 2 edges, found {n}"` (`:230-236`),
`"EndCondition state {state} must have exactly 2 edges, found {n}"` (`:271-277`), `"Failed to get
condition edge"` (`:263`), `"Failed to get end condition edge"` (`:312`), `"No edges found in
state {state}"` (`:359-363`), `"Unexpected input for Optional"` (`:204-207`).

Quantifier-resume / setup-guard errors (post-Stage-5):

| String | Site | Cause |
|---|---|---|
| `"quantifier 'any' is not supported in setup rules"` | `crates/engine/src/interpreter/mod.rs:157-159` | A `Payload::Action(GameRule::SetUp { setup })` edge whose `setup` contains `Quantifier::Any` — invariant I-20. |
| `"ChoosePlayer idx {idx} out of range ({len})"` | `crates/engine/src/interpreter/quant_driver.rs:220-224` | Resume of `DestPlayerAny` with `idx >= candidates.len()` — invariant I-8. |
| `"ChooseCards index out of range"` | `crates/engine/src/interpreter/quant_driver.rs:388-390` | Resume of `CardsAnyOrRange` / `DestAllThenCards` with a `selected` entry `>= candidate_ids.len()` — invariant I-8. |
| `"dest-player fan-out {n} exceeds cap {cap}"` | `crates/engine/src/quantifier.rs:420-423` and `:513-516` | `build_dest_all_chain` / `build_dest_all_chain_with_memory` returning `Err` when a `DestPlayerAll` resolves to more than `FANOUT_CAP = 64` players. |
| `"selected {count} does not satisfy range {:?}"` | `crates/engine/src/quantifier.rs:480-484` | `validate_int_range` returning `Err` — re-prompts the player (the resume path returns `NeedsInput` again with the error message in the prompt, see `quant_driver.rs:400-407`). |
| `"selected {count} exceeds available {available}"` | `crates/engine/src/quantifier.rs:493-497` | `validate_int_range`'s fallback when the `IntRange` uses a non-literal `IntExpr`. |

---

## 2. Recoverable vs. Unrecoverable Paths

**Recoverable** (surfaced as `Err(String)` / `crates::engine::interpreter::StepResult::Error`): all
`crates::engine::query::Evaluator` `Result` returns; condition/end-condition edge-count violations;
missing current state in the IR; dead-end non-goal states; test-file open/parse/exhaustion errors;
the six quantifier-resume / setup-guard errors above. **Since 2026-08 every DSL-reachable
`action.rs` failure is recoverable too** — `action::execute` and `Interpreter::execute_edge`
return `Result<(), String>`, and the former panic sites (`cycle to next` with no eligible
*other* player, `SetMemory`/`ResetMemory` without a current player, `CreateLocation`/
`CreateCardOnLocation`/`CreatePointMap` setup failures, `Score`/`ScoreMemory`/`CycleAction`
eval failures, `execute_cardset_move` source/dest failures) now terminate `run_game` with
`Err(String)` instead of aborting the process (see `engine-vs-design.md` F-8). These terminate
`crates::engine::controller::run_game` with `Err` and leave
`crates::engine::game_data::GameData` in whatever partially-mutated state it reached (the engine
does **not** roll back applied mutations on error —
`crates::engine::interpreter::Interpreter::execute_edge` has already written before a later
evaluator call can fail). The `validate_int_range` re-prompt path is a partial exception: it
returns `NeedsInput` (not `Error`) so the controller re-asks the player and the run continues.

**Unrecoverable** (process-aborting `panic!` / `.expect()` / `.unwrap()` / `todo!`). Since the
2026-08 fallibility pass these are **internal invariants only** — none are reachable from
well-formed DSL input:

| Site | Condition | Failure mode |
|---|---|---|
| `crates/engine/src/game_data.rs:130-136` | `crates::engine::game_data::GameData::add_location` owner (non-Table) not in `players` | `panic!("add_location: owner {} not found in players", owner_name)` — unreachable via DSL since `CreateLocation` resolves the owner first (F-8) |
| `crates/engine/src/game_data.rs:197-210` | `crates::engine::game_data::GameData::next_player` found idx missing from `turn_order` | `panic!("next_player: next_player {} not found in turn_order {:?}", next_player, self.turn_order)` (see I-13 — safe given `resolve_turn`'s contract) |
| `crates/engine/src/quantifier.rs:122` | `crates::engine::quantifier::alloc_synth` `serde_json::from_value` failure | `.expect("StateID deserialisation from a valid u32 cannot fail")` — unreachable by construction (`Value::from(u32)` into a transparent newtype) |
| `crates/engine/src/quantifier.rs:122` | `crates::engine::quantifier::alloc_synth` `serde_json::from_value` failure | `.expect("StateID deserialisation from a valid u32 cannot fail")` — the input is `Value::from(raw: u32)` and `StateID` derives `Deserialize` as a transparent newtype around `u32`, so this expect is unreachable by construction. Listed for completeness; it does not fire on any real input. |

**The quantifier subsystem introduces no new *real* panic sites.** The only `.expect` in
`crates/engine/src/quantifier.rs` is the unreachable-by-construction one in `alloc_synth` (above);
`validate_int_range` returns `Err` (not a panic) at `quantifier.rs:454-485`; `build_dest_all_chain`
returns `Err` at `:413-440`; `build_dest_all_chain_with_memory` returns `Err` at `:505-534`. The
resume arms in `crates/engine/src/interpreter/quant_driver.rs` likewise return
`StepResult::Error` on bad input (see the table in §1), never `panic!`. The setup-`Any` guard
returns `StepResult::Error`, never `panic!` (`interpreter/mod.rs:157-159`).

> **Panic capture:** when a trace log is open, `run_game` wraps `controller.run()` in
> `std::panic::catch_unwind(AssertUnwindSafe(...))` (`crates/engine/src/controller/mod.rs:98-117`),
> logs the panic message to the trace file as `=== Panic: <msg> ===`, then `resume_unwind`s so the
> panic surfaces to the caller after being logged. Without a trace log, panics propagate untouched.
> See [`observability.md`](./observability.md) §3.2.

**Silent no-ops** (neither error nor panic — agents must know these exist and do nothing):

- `front_end::ast::ActionRule::FlipAction` (`crates/engine/src/action.rs:164-167`) — payload fields
  ignored entirely; the per-card status slot (`GameData::card_statuses`) exists but is unused.
  Intended to become (de)encryption when card cryptography lands (see `engine-vs-design.md` §1b).
- `front_end::ast::ActionRule::BidAction`, `BidMemoryAction`, `DemandAction`, `DemandMemoryAction`,
  `front_end::ast::EndType::GameWithWinner`,
  `front_end::ast::SetUpRule::CreateTokenOnLocation`, `front_end::ast::MoveType::Place`
  (`crates/engine/src/action.rs`) — `// TODO` no-ops.
- `front_end::ir::Payload::Trigger` traversal: `crates::engine::interpreter::Interpreter::step`
  advances the state (`execute_edge`) but `crates::engine::action::execute`'s catch-all
  `_ => {}` performs no mutation.
- Out-of-range `Choice`/`Optional` input (I-8) — silent stall, no error.

**Former silent no-ops now fixed (2026-08):** `ShuffleAction` eval failures (were
`eprintln!` + continue — now `Err`), `PlayerCollection::AggregateMemory`/`Memory`
(returned `vec![]` silently — now read/aggregate real slots or error), and the four
collection-memory `todo!()` arms (now implemented).

**NOT silent no-ops** (the quantifier arms — they actively mutate or prompt):

- `QuantSite::DestPlayerAll` / `DestPlayerAny` / `SrcCardsAnyOrRange` (`interpreter/mod.rs:131-150`)
  build a synthetic overlay chain or issue a `NeedsInput` prompt — they do real work.
- The resume arms in `quant_driver.rs:213-334` write the synthetic memory slot, build/insert
  replacement edges, and advance `current_state` — they do real work.
- The `SYNTH_MEMORY_KEY` cleanup (`interpreter/mod.rs:65-79`) removes a slot — it does real work
  (and is itself an invariant, I-18).

A quantifier site is never a silent no-op; verify this remains true if you add a new
`QuantSite` variant.
