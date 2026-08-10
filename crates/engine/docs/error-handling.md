---
type: agent_wiki_node
module: crates::engine
scope: [engine::error, engine::interpreter, engine::interpreter::quant_driver, engine::action, engine::query, engine::controller, engine::game_data, engine::quantifier]
topics: [error-handling, panics, result, recoverability, silent-noops, engine-error]
associated_files:
  - crates/engine/src/error.rs
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/interpreter/quant_driver.rs
  - crates/engine/src/action.rs
  - crates/engine/src/query/mod.rs
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/game_data.rs
  - crates/engine/src/quantifier.rs
last_validated: 2026-08-10
---

# Error Handling, Panic Conditions & Diagnostics

The engine's error channel is the **`crates::engine::error::EngineError` enum** (re-exported at the
crate root as `cgdsl_engine::EngineError`), defined with `thiserror` in
`crates/engine/src/error.rs`. Every fallible engine operation returns `Result<_, EngineError>` —
there are no stringly-typed `Result<_, String>` signatures left. The `Display` messages are the
public diagnostic surface (TUI panels, `cgdsl-play` output, trace-log footers, re-prompt prompts)
and are **stable**: they are byte-for-byte the messages the old `String` errors produced, and the
unit tests assert them via `err.to_string()`.

This page enumerates the error channels, the recoverable vs. panic/silent paths, and points to
[`observability.md`](./observability.md) for telemetry. Several panic sites enforce invariants from
[`invariants.md`](./invariants.md); cross-references use the `I-N` IDs.

---

## 1. Error Types

Three error channels exist, all carrying the same `EngineError`:

| Channel | Type | Origin |
|---|---|---|
| Run failure | `Result<GameData, EngineError>` from `crates::engine::controller::run_game` (`crates/engine/src/controller/mod.rs:96`) | Propagated from `crates::engine::interpreter::StepResult::Error` or `Controller::get_input`. |
| Step failure | `crates::engine::interpreter::StepResult::Error(EngineError)` (`crates/engine/src/interpreter/types.rs:71`) | Missing state, bad edge counts, evaluator errors, quantifier-resume validation errors (below). |
| Eval failure | `Result<_, EngineError>` from every `crates::engine::query::Evaluator` method | Division by zero, missing memory/location/precedence/pointmap/combo, type-mismatched memory, out-of-range index, "no current player/stage", etc. |

### 1.1 The `EngineError` variant catalog

`crates/engine/src/error.rs` is the **authoritative catalog** — it groups variants by raising
module and carries a doc comment per variant. The `Display` strings (the messages tests assert
against) are unchanged from the pre-enum string errors; representative examples:

- **Query/evaluator** (`query/`): `DivisionByZero` ("Division by zero"),
  `MemoryNotFound { key }` ("Memory {key} not found"), `MemoryNotInt` /
  `MemoryNotIntFor { key }` / `MemoryNotString` / `MemoryNotTeam` /
  `MemoryNotCardSet` / `MemoryNotPlayerCollection` / … (the "Memory value is not a …"
  family), `MemoryRequiresExplicitOwner { key }`, `NoCurrentPlayer`, `NoCurrentStage`,
  `NoNextPlayerAvailable`, `PreviousPlayerNotFound`, `NoCompetitorFound`,
  `CardOwnerNotFound`, `OwnerOfMemoryNoPlayer`, `EmptyPlayerCollectionMemory`,
  `PlayerNotInAnyTeam { name }`, `IntCollectionAtOutOfRange { idx }`,
  `StringCollectionAtOutOfRange { idx }`, `PointMapNotFound { name }`,
  `NoCardForExtrema`, `NoValueInIntCollection`, `TurnOrderIndexOutOfRange { idx }`,
  `PlayerIndexNotFound { idx }`, `PlayerCollectionAtOutOfRange { idx }`,
  `PlayerCollectionIndexNotFound { idx }`, `ResolvePlayersPlayerNotFound { name }`,
  `ResolvePlayerCollectionPlayerNotFound { name }`, `OwnerNameFromPlayerCollection`,
  `OwnerNameFromTeamCollection`, `TeamCannotOwn { name }`,
  `OwnerNamesFromTeamCollection`, `CardNotFound { card_id }`,
  `CardKeyNotFound { key, card_id }`, `LocationNotFoundForOwner { name, owner }`,
  `LocationNotFound { name }`, `LocationNotFoundForCardPosition { name }`,
  `CardPositionNotFound`, `PrecedenceNotFound { name }`,
  `ValueNotFoundInPrecedence { value, precedence }`, `ComboNotFound { name }`,
  `CardAtIndexNotFound { idx, location }`, `CardAtTopNotFound { location }`,
  `CardAtBottomNotFound { location }`, `NoCardForExtremaPointMap`,
  `NoCardForExtremaPrecedence`.
- **Action** (`action.rs`): context-wrapping variants that embed the failing AST expression
  (`{owner:?}`, `{int_expr:?}`, `{cardset:?}`, `{player:?}`) and a boxed `source`:
  `CreateLocationOwnerResolution`, `CreateCardOnLocationLocationNotFound`,
  `CreateMemoryOwnerResolution`, `CreateMemoryWithTypeOwnerResolution`,
  `CreatePointMapIntEval`, `ShuffleActionEval`, `SetMemoryIntEval`,
  `SetMemoryStringEval`, `SetMemoryPlayerEval`, `SetMemoryTeamEval`,
  `SetMemoryNoCurrentPlayer`, `ResetMemoryNoCurrentPlayer`,
  `CycleActionPlayerEval`, `CycleActionPlayerNotFound`,
  `CycleActionTurnOrderNotFound`, `ScoreIntEval`, `ScoreMemoryIntEval`,
  `MoveFromCardsetEval`, `MoveDestCardsetEval`, `MoveDestLocationOutOfRange`.
- **Interpreter** (`interpreter/mod.rs`): `CurrentStateNotFoundInIr { state }`,
  `NoOutgoingEdges { state }`, `NoEdgesFound { state }`, `ConditionEdgeCount { state, found }`,
  `EndConditionEdgeCount { state, found }`, `ConditionEdgeMissing`,
  `EndConditionEdgeMissing`, `UnexpectedInputForOptional`.
- **Quantifier** (`quantifier.rs`, `quant_driver.rs`): `DestPlayerFanoutExceedsCap { n, cap }`,
  `SelectionDoesNotSatisfyRange { count, range }` (re-prompt message, see §2),
  `SelectionExceedsAvailable { count, available }`, `ChoosePlayerIdxOutOfRange { idx, len }`,
  `ChooseCardsIndexOutOfRange` (both invariant I-8).
- **Controller / test input** (`controller/mod.rs`): `TestFileOpen { path, source }`,
  `TestFileRead { path, source }` (both wrap `std::io::Error`), `TestInputExhausted`,
  `InvalidTestInputP`, `InvalidTestInputPlayerZero`, `InvalidTestInputC`,
  `InvalidTestInputCardZero`, `InvalidTestInputNumber`, `InvalidTestInputChoiceZero`,
  and `InternalPanic { message }` — an internal-invariant panic caught and converted by
  `run_game_with` with `RunOptions::capture_panics(true)` (see §2).

Design notes:

- **Large AST payloads are boxed** (`Box<Owner>`, `Box<IntExpr>`, `Box<CardSet>`,
  `Box<PlayerExpr>`, `Box<IntRange>`) and wrapping variants box their `source:
  Box<EngineError>` — this keeps the enum small (`clippy::result_large_err` clean) and
  gives a cheap `Error::source()` chain to the root cause.
- **`EngineError::kind()`** returns a coarse [`ErrorKind`] classifier
  (`Query` / `Action` / `Interpreter` / `Quantifier` / `Input` / `Internal`) for hosts that want to
  group or handle errors without matching every variant — pure additive API on the enum.
- The `validate_int_range` re-prompt path formats the error into the re-prompt prompt
  (`"{}. Please choose again."` in `quant_driver.rs`) — the `Display` of
  `SelectionDoesNotSatisfyRange` / `SelectionExceedsAvailable` is part of the UI contract.
- There is deliberately **no `PartialEq`** on `EngineError`: equality is message-comparison
  via `to_string()` in tests, never structural.

---

## 2. Recoverable vs. Unrecoverable Paths

**Recoverable** (surfaced as `Err(EngineError)` / `crates::engine::interpreter::StepResult::Error`):
all `crates::engine::query::Evaluator` `Result` returns; condition/end-condition edge-count
violations; missing current state in the IR; dead-end non-goal states; test-file
open/parse/exhaustion errors; the quantifier-resume / setup-guard errors listed in §1. **Since
2026-08 every DSL-reachable `action.rs` failure is recoverable too** — `action::execute` and
`Interpreter::execute_edge` return `Result<(), EngineError>`, and the former panic sites (`cycle
to next` with no eligible *other* player, `SetMemory`/`ResetMemory` without a current player,
`CreateLocation`/`CreateCardOnLocation`/`CreatePointMap` setup failures,
`Score`/`ScoreMemory`/`CycleAction` eval failures, `execute_cardset_move` source/dest failures)
now terminate `run_game` with `Err(EngineError)` instead of aborting the process (see
`engine-vs-design.md` F-8). These terminate `crates::engine::controller::run_game` with `Err` and
leave `crates::engine::game_data::GameData` in whatever partially-mutated state it reached (the
engine does **not** roll back applied mutations on error —
`crates::engine::interpreter::Interpreter::execute_edge` has already written before a later
evaluator call can fail). The `validate_int_range` re-prompt path is a partial exception: it
returns `NeedsInput` (not `Error`) so the controller re-asks the player and the run continues.

**Unrecoverable** (process-aborting `panic!` / `.expect()` / `.unwrap()` / `todo!`). Since the
2026-08 fallibility pass these are **internal invariants only** — none are reachable from
well-formed DSL input:

| Site | Condition | Failure mode |
|---|---|---|
| `crates/engine/src/game_data.rs:139-156` | `crates::engine::game_data::GameData::add_location` owner (non-Table) not in `players` | `panic!("add_location: owner {} not found in players", owner_name)` — unreachable via DSL since `CreateLocation` resolves the owner first (F-8) |
| `crates/engine/src/game_data.rs:228-246` | `crates::engine::game_data::GameData::next_player` found idx missing from `turn_order` | `panic!("next_player: next_player {} not found in turn_order {:?}", next_player, self.turn_order)` (see I-13 — safe given `resolve_turn`'s contract) |
| `crates/engine/src/quantifier.rs:138` | `crates::engine::quantifier::alloc_synth` `serde_json::from_value` failure | `.expect("StateID deserialisation from a valid u32 cannot fail")` — the input is `Value::from(raw: u32)` and `StateID` derives `Deserialize` as a transparent newtype around `u32`, so this expect is unreachable by construction. Listed for completeness; it does not fire on any real input. |

**The quantifier subsystem introduces no new *real* panic sites.** The only `.expect` in
`crates/engine/src/quantifier.rs` is the unreachable-by-construction one in `alloc_synth` (above);
`validate_int_range` returns `Err(EngineError)` (not a panic) at `quantifier.rs:454-485`;
`build_dest_all_chain` returns `Err` at `:413-440`; `build_dest_all_chain_with_memory` returns
`Err` at `:505-534`. The resume arms in `crates/engine/src/interpreter/quant_driver.rs` likewise
return `StepResult::Error` on bad input (see the variant list in §1), never `panic!`. The
setup-`Any` guard prompts for a player (`step_setup_any`), never `panic!` (`interpreter/mod.rs:160-163`).

> **Panic capture:** `run_game`/`run_game_with` catch panics inside the run loop
> (`crates/engine/src/controller/mod.rs`) in two situations: (1) **always**, when the caller set
> `RunOptions::capture_panics(true)` — the panic message is logged to the trace file as
> `=== Panic: <msg> ===` if a trace log is open, then returned as
> `Err(EngineError::InternalPanic { message })` instead of aborting the process; (2) **only when a
> trace log is open** and `capture_panics` is `false` (the legacy default) — the panic is logged
> and then `resume_unwind`ed so it surfaces to the caller after being logged. Without a trace log
> and without `capture_panics`, panics propagate untouched. See
> [`observability.md`](./observability.md) §3.2.

**Silent no-ops** (neither error nor panic — agents must know these exist and do nothing):

- `front_end::ast::ActionRule::FlipAction` (`crates/engine/src/action.rs:207-211`) — payload fields
  ignored entirely; the per-card status slot (`GameData::card_statuses`) exists but is unused.
  Intended to become (de)encryption when card cryptography lands (see `engine-vs-design.md` §1b).
- `front_end::ast::ActionRule::BidAction` (plain `bid <qty>` without a memory target) is
  **no longer a silent no-op** — since 2026-08-10 it returns
  `EngineError::BidWithoutMemoryTarget` (see `dsl-semantics.md` §3.7).
- `front_end::ast::ActionRule::BidMemoryAction`, `DemandAction`, `DemandMemoryAction`,
  `front_end::ast::SetUpRule::CreateTokenOnLocation`, `front_end::ast::MoveType::Place`
  (`crates/engine/src/action.rs`) — `// TODO` no-ops. (`BidMemoryAction` is **implemented**
  since 2026-08-10: literal quantities write the owner's slot; `any`/ranges are prompted by
  the interpreter as `InputType::Number`. `EndType::GameWithWinner` is **implemented** since
  2026-08-10: the declared winners eliminate everyone else before the IR jumps to the goal.)
- `front_end::ir::Payload::Trigger` traversal: `crates::engine::interpreter::Interpreter::step`
  advances the state (`execute_edge`) but `crates::engine::action::execute`'s catch-all
  `_ => {}` performs no mutation.
- Out-of-range `Choice`/`Optional` input (I-8) — silent stall, no error.

**Former silent no-ops now fixed (2026-08):** `ShuffleAction` eval failures (were
`eprintln!` + continue — now `Err(EngineError::ShuffleActionEval)`), `PlayerCollection::AggregateMemory`/`Memory`
(returned `vec![]` silently — now read/aggregate real slots or error), and the four
collection-memory `todo!()` arms (now implemented).

**NOT silent no-ops** (the quantifier arms — they actively mutate or prompt):

- `QuantSite::DestPlayerAll` / `DestPlayerAny` / `SrcCardsAnyOrRange` (`interpreter/mod.rs:137-153`)
  build a synthetic overlay chain or issue a `NeedsInput` prompt — they do real work.
- The resume arms in `quant_driver.rs:307-479` write the synthetic memory slot, build/insert
  replacement edges, and advance `current_state` — they do real work.
- The `SYNTH_MEMORY_KEY` cleanup (`interpreter/mod.rs:71-85`) removes a slot — it does real work
  (and is itself an invariant, I-18).

A quantifier site is never a silent no-op; verify this remains true if you add a new
`QuantSite` variant.
