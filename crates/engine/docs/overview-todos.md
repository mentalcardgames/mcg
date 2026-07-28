---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [todos, gaps, unimplemented, stubs, noop, completeness]
last_validated: 2026-07-28
---

# Engine Completeness Audit

## 1. Implemented and Working

### Interpreter (`interpreter/mod.rs`, `quant_driver.rs`, `trace.rs`)

* Full FSM step execution: `Choice`, `Optional`, `Condition`, `EndCondition`,
  `StageRoundCounter`, `EndStage`, `Trigger` dispatch
* Quantifier preprocessor: `DestPlayerAll`, `DestPlayerAny`, `SrcCardsAnyOrRange`
  with synthetic edge chains, `IntRange` validation, re-prompt on invalid card
  selections
* Input buffer (LIFO), `provide_input`, quantifier resume state machine
* Trace emission for every payload arm (`TraceEntry`, `TraceEvent`)

### Controller (`controller/mod.rs`)

* `run_game` with `InputSource::Player` and `InputSource::TestFile`
* Player-fingerprinting validation loop (`validate_player_input`): checks
  `input.player_id == current_player_name` plus range validation
* `Name:` prefix support in test input files (defaults to `"P1"`)
* Optional `MCG_TRACE_LOG` file logging with panic capture
* Event sender + trace sender composition

### Types (`interpreter/types.rs`)

* `Input { player_id, kind }` / `InputKind` enum with five variants and
  accessor delegation (`idx()`, `player_idx()`, `card_selection()`)
* `InputType` (Choice, Optional, ChoosePlayer, ChooseCards)
* `StepResult` enum

### Action System (`action.rs`) — 26 of 31 variants implemented

**SetUp rules** (all 12 variants implemented):
`CreatePlayer`, `CreateTeams`, `CreateTurnorder`, `CreateTurnorderRandom`,
`CreateLocation`, `CreateCardOnLocation`, `CreateCombo`, `CreateMemory`,
`CreateMemoryWithMemoryType`, `CreatePrecedence`, `CreatePointMap`

**Action rules** (11 of 16 implemented):
`ShuffleAction`, `OutAction`, `SetMemory` (Int, String, Player, Team sub-variants),
`ResetMemory`, `CycleAction`, `EndAction` (Turn, CurrentStage, Stage),
`ScoreRule::Score`, `ScoreRule::ScoreMemory`, `WinnerRule::Winner`,
`WinnerRule::WinnerWith`

**Move rules** (3 of 4 implemented):
`Deal`, `Exchange`, `Classic` — all routed through `execute_cardset_move`
with full quantity resolution

### Query Evaluator (`query/`)

* `bool.rs` — **100%** — all `BoolExpr`, `AggregateBool`, `CompareBool`,
  `EndCondition` variants
* `cardset.rs` — **100%** — all `CardSet`, `Group`, `Groupable`, `FilterExpr`,
  `CardPosition`, `check_attr_value_in_cardset` variants
* `int.rs` — complete except 2 `todo!()` sites
* `string.rs` — complete except 1 `todo!()`
* `player.rs` — complete except 1 `todo!()` + 2 silent empty-returns
* `team.rs` — fully implemented

### GameData (`game_data.rs`) — 16 of 22 public methods working correctly

`new`, `add_location`, `add_player`, `get_card`, `find_location_of_card`,
`increment_stage_counter`, `reset_stage_counter`, `get_stage_counter`,
`get_current_player`, `set_player_out`, `set_player_stage_flag`,
`get_current_stage`, `enter_stage`, `ensure_stage_entered`, `get_memory`,
`set_memory`

### TUI (`bin/engine-tui/`)

* Dual-panel layout: game state debug view + IR trace log
* 4 trace detail levels: Choices, Evaluations, Verbose, Last5
* Choose-player and choose-cards cursor navigation (arrows, Space, Enter)
* Keyboard gating: only the current player's inputs are accepted
* Perspective cycling via `p` key
* Turn change separators in trace log (magenta `─── Turn: P2 ───` line)
* Display: "Waiting for X's turn... (you are viewing as Y)" when not current

### CLI (`bin/cgdsl-play.rs`)

* Interactive mode: stdin-driven play with full prompt support
* Test file mode: replay from `.txt` input files
* Current player name tracking via `event_sender`

### Tests

* **228 tests** passing: 213 unit + 13 integration + 2 action
* **15 `.cgdsl` test fixtures** in `test_games/` covering quantifiers, setup
  rules, actions, stage transitions, and turn switching
* Unit tests for: GameData, quantifier, interpreter dispatch, types, controller, debug

---

## 2. Not Implemented

### 2.1 `todo!()` Panics (will crash if reached)

| File | Line | Message | Triggered by |
|------|------|---------|-------------|
| `query/player.rs` | 246 | `"PlayerCollection::Aggregate not yet implemented"` | `end game with winner(for all ...)`, `OutOfPlayer` with Aggregate quantifier |
| `query/int.rs` | 173 | `"IntCollection::AggregateMemory not yet implemented"` | `SizeOf`, `IntCollectionAt` on aggregated int memory |
| `query/int.rs` | 266 | `"TeamCollection::AggregateMemory not yet implemented"` | `SizeOf` on aggregated team memory |
| `query/string.rs` | 60 | `"StringCollection::AggregateMemory not yet implemented"` | `StringCollectionAt`, `SizeOf` on aggregated string memory |

### 2.2 Action Subsystem — Silent No-Ops / Stubs

**Token and card-status actions** (data model never built):

| Variant | Behavior | File:line |
|---------|----------|-----------|
| `CreateTokenOnLocation` | Empty `{}` — tokens are not in the data model | `action.rs:124` |
| `FlipAction` | Empty `{}` — cards have no status field | `action.rs:164` |
| `MoveType::Place` | Empty `{}` — token placement never spec'd | `action.rs:372` |

**Bidding and demand** (game-design semantics undefined):

| Variant | Behavior | File:line |
|---------|----------|-----------|
| `BidAction` | Empty `{}` | `action.rs:272` |
| `BidMemoryAction` | Empty `{}` | `action.rs:275` |
| `DemandAction` | Empty `{}` | `action.rs:294` |
| `DemandMemoryAction` | Empty `{}` | `action.rs:297` |

**Scoring and winner determination**:
`ScoreRule::Score`, `ScoreRule::ScoreMemory`, `WinnerRule::Winner`, and
`WinnerRule::WinnerWith` are implemented as of 2026-07-28. Score adds the
evaluated int to each resolved player's `.score` field. ScoreMemory writes
the int to a global memory slot (per-player memory not yet available).
Winner eliminates all non-winner playrs. WinnerWith compares Score, Position,
or Memory across in-game players and eliminates non-matching players.

**Game end**:

| Variant | Behavior | File:line |
|---------|----------|-----------|
| `EndType::GameWithWinner` | Empty `{}` — cannot declare a winner on end | `action.rs:290` |

**SetMemory collection sub-variants** (insert empty defaults instead of evaluating):

| Sub-variant | Default inserted | File:line |
|-------------|-----------------|-----------|
| `MemoryType::PlayerCollection` | `vec![]` | `action.rs:233` |
| `MemoryType::StringCollection` | `vec![]` | `action.rs:234` |
| `MemoryType::TeamCollection` | `Int(0)` (wrong!) | `action.rs:235` |
| `MemoryType::IntCollection` | `vec![]` | `action.rs:236` |
| `MemoryType::LocationCollection` | `vec![]` | `action.rs:237` |
| `MemoryType::CardSet` | `vec![]` | `action.rs:238` |

### 2.3 Query Evaluator — Silent Empty Returns

| File | Line | Variant | Returns |
|------|------|---------|---------|
| `query/player.rs` | 277 | `PlayerCollection::AggregateMemory` | Empty `vec![]` |
| `query/player.rs` | 282 | `PlayerCollection::Memory` | Empty `vec![]` |

### 2.4 GameData — Known Bugs (documented in `invariants.md`)

| Bug | Invariant | Impact | File:line |
|-----|-----------|--------|-----------|
| `add_card` ignores `_location_id` parameter | I-6 | Caller must manually push card into location | `game_data.rs:156` |
| `resolve_turn` / `next_player` never considers current player | I-13 | 2-player games can deadlock when the other player is ineligible | `game_data.rs:261-274` |
| `leave_stage` drains entire stack if stage not found | I-11 | Silent stage stack corruption — `while` loop never `break`s | `game_data.rs:252-259` |
| `add_memory` initializes `Player` to `Int(0)`, `TeamCollection` to `Int(0)` | I-10 | Type mismatch — reads of these will fail | `game_data.rs:280,286` |
| `reset_memory` only resets `Int` memories | — | Non-Int memories silently ignored | `game_data.rs:305-311` |
| `execute_cardset_move` uses `>` not `>=` for dest index guard | — | Off-by-one: `== locations.len()` slips past guard, triggers Rust index-out-of-bounds at `action.rs:391` | `action.rs:370` |

---

## 3. Blocked by Missing Dependencies

These features exist as AST variants and dispatch locations but could not be
completed because their semantics were never specified or required other
subsystems to exist first.

### Bidding / Demand

`BidAction`, `BidMemoryAction`, `DemandAction`, `DemandMemoryAction` are
no-ops. These require a game-design spec defining what a bid does, how demands
interact with player input round-trips, and whether they are synchronous
(blocking) or asynchronous. The engine handles player input via `NeedsInput`
(Choice/Optional/ChoosePlayer/ChooseCards), but bidding/demand were intended
as different interaction patterns.

### `GameWithWinner`

Requires a winner-determining expression evaluator that references scores or
player state. Depends on the scoring subsystem being implemented first.

### Collection memory evaluation

The 4 `todo!()` sites in `query/` all involve aggregating memory over multiple
owners. Primitive memory reads (single-owner Int, String, CardSet) work.
The aggregation path (`AggregateMemory`) needs the evaluator to iterate owners,
sum or collect, and return — but the multi-owner iteration logic was never
plumbed through the evaluator.

### Card status (FaceUp / FaceDown / Private)

`FlipAction` exists as an AST/action variant but cards in `GameData` have no
status field. Implementation requires:
* A status enum added to `GameData::Card` or `Location`
* Query evaluator support for filtering by status
* Integration with the existing move logic

### Tokens

`CreateTokenOnLocation` and `MoveType::Place` are stubs. Tokens were never
added to the `GameData` data model. Implementation requires:
* A `Token` type (string? struct?)
* Location-level token storage
* Move logic for token transfers
* Query support (`TokenSet`, `eval_token`, etc.)

---

## 4. Summary

| Category | Count | Status |
|----------|-------|--------|
| Action variants implemented | 26 / 31 | 84% |
| Action variants stubbed / no-op | 11 | See §2.2 |
| `todo!()` panics in query | 4 | See §2.1 |
| Silent empty-returns in query | 2 | See §2.3 |
| GameData methods fully working | 16 / 22 | 73% |
| GameData known bugs | 6 | See §2.4 |
| Feature areas blocked by missing deps | 5 | See §3 |
| Tests passing | 230 | — |
| Test fixtures | 15 `.cgdsl` files | — |
