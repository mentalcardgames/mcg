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
last_validated: 2026-07-28
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
the six quantifier-resume / setup-guard errors above. These terminate
`crates::engine::controller::run_game` with `Err` and leave
`crates::engine::game_data::GameData` in whatever partially-mutated state it reached (the engine
does **not** roll back applied mutations on error —
`crates::engine::interpreter::Interpreter::execute_edge` has already written before a later
evaluator call can fail). The `validate_int_range` re-prompt path is a partial exception: it
returns `NeedsInput` (not `Error`) so the controller re-asks the player and the run continues.

**Unrecoverable** (process-aborting `panic!` / `.expect()` / `.unwrap()` / `todo!`). These are
invariants the code assumes the IR/data will satisfy; an abnormal IR or DSL input can trigger them:

| Site | Condition | Failure mode |
|---|---|---|
| `crates/engine/src/action.rs:92-95` | `front_end::ast::SetUpRule::CreateLocation` owner fails `resolve_owner_to_names` | `panic!("CreateLocation: failed to resolve owner {:?}: {}", owner, e)` |
| `crates/engine/src/action.rs:108-115` | `front_end::ast::SetUpRule::CreateCardOnLocation` location name not found | `panic!("CreateCardOnLocation: location {:?} not found", location)` |
| `crates/engine/src/action.rs:254-291` | `ActionRule::SetMemory` — any `MemoryType` expression eval failure, or no current player to key the memory to | `panic!("SetMemory Int/String/Player/Team eval failed: {e}")` / `panic!("SetMemory requires a current player")` |
| `crates/engine/src/action.rs:296-301` | `ActionRule::ResetMemory` with no current player | `panic!("ResetMemory requires a current player")` |
| `crates/engine/src/action.rs:306-309` | `front_end::ast::ActionRule::CycleAction` player expr fails to eval | `panic!("CycleAction: failed to eval player {:?}: {}", player, e)` — reachable when `cycle to next` runs with no eligible *other* player (see I-13: `resolve_turn` never considers the current player) |
| `crates/engine/src/action.rs:313-318` | `CycleAction` resolved player not in `players` | `panic!("CycleAction: player {} not found in game_data.players", player_name)` |
| `crates/engine/src/action.rs:323-327` | `CycleAction` player not in `turn_order` | `panic!("CycleAction: player_idx {} not in turn_order {:?}", player_idx, game_data.turn_order)` |
| `crates/engine/src/action.rs:375-380` | `ScoreRule::Score` int expr eval fails | `panic!("Score: failed to eval int {:?}: {}", int, e)` |
| `crates/engine/src/action.rs:387-392` | `ScoreRule::ScoreMemory` int expr eval fails | `panic!("ScoreMemory: failed to eval int {:?}: {}", int, e)` |
| `crates/engine/src/action.rs:513-517` | `crates::engine::action::execute_cardset_move` source `eval_cardset` fails | `panic!("execute_cardset_move: failed to eval from cardset {:?}: {}", from, e)` |
| `crates/engine/src/action.rs:529-533` | `execute_cardset_move` dest `eval_cardset` fails | `panic!("execute_cardset_move: failed to eval dest cardset {:?}: {}", to, e)` |
| `crates/engine/src/action.rs:538-545` | destination location index out of range | `panic!("execute_cardset_move: dest_loc_idx {} >= locations.len() {} (cardset expr: {:?})", dest_loc_idx, game_data.locations.len(), to)` — the guard uses `>=`, so the index-out-of-bounds panic at `game_data.locations[dest_loc_idx]` (`:557`) is unreachable. (Older revisions used `>` and let `dest_loc_idx == locations.len()` slip past into a Rust index panic; that latent off-by-one is fixed.) |
| `crates/engine/src/game_data.rs:130-136` | `crates::engine::game_data::GameData::add_location` owner (non-Table) not in `players` | `panic!("add_location: owner {} not found in players", owner_name)` |
| `crates/engine/src/game_data.rs:197-210` | `crates::engine::game_data::GameData::next_player` found idx missing from `turn_order` | `panic!("next_player: next_player {} not found in turn_order {:?}", next_player, self.turn_order)` (see I-13 — safe given `resolve_turn`'s contract) |
| `crates/engine/src/query/player.rs:201-207` | `crates::engine::query::Evaluator::resolve_players` player eval fails | `panic!("resolve_players: failed to eval player {:?}: {}", player, e)` |
| `crates/engine/src/query/player.rs:208-214` | `resolve_players` resolved player name not in `players` | `panic!("resolve_players: player {} not found in game_data", name)` |
| `crates/engine/src/query/player.rs:221-230` | `crates::engine::query::Evaluator::resolve_player_collection` (Literal arm) player eval fails | `panic!("resolve_player_collection: failed to eval player {:?}: {}", player_expr, e)` |
| `crates/engine/src/query/player.rs:240` | `front_end::ast::PlayerCollection::Aggregate` reached via `resolve_player_collection` | `todo!("PlayerCollection::Aggregate not yet implemented")` — panics if a DSL program reaches this arm directly. The quantifier subsystem **intercepts** `Aggregate { Quantifier }` *before* this call site (`crates/engine/src/quantifier.rs:140-153`), so the engine's own quantifier paths never trigger this `todo!`. |
| `crates/engine/src/query/int.rs:170-173` | `IntCollection::AggregateMemory` | `todo!("IntCollection::AggregateMemory not yet implemented: {:?}", multi)` |
| `crates/engine/src/query/int.rs:257-260` | `TeamCollection::AggregateMemory` | `todo!("TeamCollection::AggregateMemory not yet implemented: {:?}", multi)` |
| `crates/engine/src/query/string.rs:55-58` | `StringCollection::AggregateMemory` | `todo!("StringCollection::AggregateMemory not yet implemented: {:?}", multi)` |
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
  ignored entirely.
- `front_end::ast::ActionRule::ShuffleAction` when `eval_cardset` fails
  (`crates/engine/src/action.rs:178-180`) — prints `eprintln!("ShuffleAction failed: {}", e)` and
  continues; the pile is left unshuffled.
- `front_end::ast::ActionRule::BidAction`, `BidMemoryAction`, `DemandAction`, `DemandMemoryAction`,
  `front_end::ast::EndType::GameWithWinner`,
  `front_end::ast::SetUpRule::CreateTokenOnLocation`, `front_end::ast::MoveType::Place`
  (`crates/engine/src/action.rs:124, 236-266, 254-256, 274-292, 335`) — `// TODO` no-ops.
- `front_end::ir::Payload::Trigger` traversal: `crates::engine::interpreter::Interpreter::step`
  advances the state (`execute_edge`) but `crates::engine::action::execute`'s catch-all
  `_ => {}` (`crates/engine/src/action.rs:57`) performs no mutation.
- `front_end::ast::PlayerCollection::AggregateMemory`, `front_end::ast::PlayerCollection::Memory`
  (`crates/engine/src/query/player.rs:278-282`) — return `vec![]` silently.
- Out-of-range `Choice`/`Optional` input (I-8) — silent stall, no error.

**NOT silent no-ops** (the quantifier arms — they actively mutate or prompt):

- `QuantSite::DestPlayerAll` / `DestPlayerAny` / `SrcCardsAnyOrRange` (`interpreter/mod.rs:131-150`)
  build a synthetic overlay chain or issue a `NeedsInput` prompt — they do real work.
- The resume arms in `quant_driver.rs:213-334` write the synthetic memory slot, build/insert
  replacement edges, and advance `current_state` — they do real work.
- The `SYNTH_MEMORY_KEY` cleanup (`interpreter/mod.rs:65-79`) removes a slot — it does real work
  (and is itself an invariant, I-18).

A quantifier site is never a silent no-op; verify this remains true if you add a new
`QuantSite` variant.
