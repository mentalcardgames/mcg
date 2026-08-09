---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [design-decisions, todos, gaps, known-bugs, completeness]
last_validated: 2026-07-28
---

# Developer Notes

Design decisions, completeness audit, and known bugs — consolidated for the next maintainer.

---

## 1. Design Decisions

### 1.1 Memory is Owned and Scoped

Memory is the engine's name-value store (`GameData::memories: HashMap<String, MemoryValue>`).
Every memory slot is owned by an entity — a Player or the Table.
This ownership is encoded by **prefixing the key** in the flat HashMap:

| Owner | Prefix | Example key |
|-------|--------|-------------|
| Player `P1` | `"P1_"` | `"P1_M"` |
| Player `P2` | `"P2_"` | `"P2_score"` |
| Table | `"Table_"` | `"Table_pot"` |

**Setup time:** `CreateMemory` / `CreateMemoryWithMemoryType` resolve the `owner` to names and insert `"{owner}_{name}"`.

**Runtime reads:** All evaluator memory arms use `Evaluator::resolve_memory_key` / `resolve_collection_memory_key`. When the AST carries a `WithOwner` variant (`&I:M of P1`), the owner is used directly. The parser already supports `of <owner>` in all memory-reference grammar rules (`&I:M of ...`, `&P:M of ...`, etc.). The bare `Memory { memory }` variant (no owner) returns an error.

**Runtime writes (bridge):** `SetMemory` (`M is 42`) and `ResetMemory` (`reset M`) lack an owner clause in the grammar. As a bridge, they prefix the key with the current player name. Each site is flagged with `// NOTE(grammar-gap)`. When the grammar adds `of <owner>` to these rules, the bridge code is replaced with the explicit owner from the AST.

**Why not a nested data structure?** `GameData` is `Clone` and the flat `HashMap` is serializable. Owner-prefixed keys need zero type-system changes — just string formatting at the access site, with guaranteed non-collision.

### 1.2 Scoring: ScoreMemory Semantics

`score N to M of P1` writes the evaluated int to memory slot `"P1_M"`. It does **not** affect `Player::score`. Only `score N to P1` (without a memory clause) mutates `Player::score`.

### 1.3 Scoring: WinnerWith::Position Interpretation

`winner is highest position` / `winner is lowest position` uses the player's index in `turn_order` (0-based). `turn_order [P2, P1, P3]` → P2=0, P1=1, P3=2. `highest position` → P3 wins. `lowest position` → P2 wins. Players not in `turn_order` get `usize::MAX`. This interpretation may not match the intended DSL semantics.

### 1.4 CGDSL Identifiers Must Start With Capital Letter

Per the Pest grammar `ident = { &capital ~ ... }`, names must start with a capital letter. Memory named `m` fails to parse; use `M`. This applies to: memory names, stage names, team names, combo names, precedence names, point map names, location names, and token names.

---

## 2. Completeness Audit

### 2.1 Implemented and Working

**Interpreter:** Full FSM step execution, quantifier preprocessor (DestPlayerAll / DestPlayerAny / SrcCardsAnyOrRange), input buffer (LIFO), trace emission for all payload arms.

**Controller:** `run_game` with `InputSource::Player` and `InputSource::TestFile`, player-fingerprinting validation, `Name:` prefix in test files, `MCG_TRACE_LOG` file logging.

**Types:** `Input { player_id, kind }` / `InputKind` with accessor delegation, `InputType` (Choice, Optional, ChoosePlayer, ChooseCards).

**Actions — 26 of 31 variants implemented:**
SetUp (all 12): CreatePlayer, CreateTeams, CreateTurnorder/Random, CreateLocation, CreateCardOnLocation, CreateCombo, CreateMemory/WithMemoryType, CreatePrecedence, CreatePointMap.
Action (11/16): ShuffleAction, OutAction, SetMemory, ResetMemory, CycleAction, EndAction (Turn/Stage), Score, ScoreMemory, Winner, WinnerWith.
Move (3/4): Deal, Exchange, Classic.

**Query Evaluator:** `bool.rs` and `cardset.rs` — 100%. `int.rs`, `string.rs`, `player.rs` — complete except 4 `todo!()` panics and 2 silent empty-returns. Full unit test suite (5 files, ~180 tests).

**GameData:** 16 of 22 methods working correctly. Memory ownership model implemented (prefixed keys). 6 documented known bugs (see §2.4).

**TUI:** Dual-panel layout, 4 trace detail levels, choose-player/cards navigation, keyboard gating, turn-change separators.

**CLI:** Interactive + test-file mode with player name tracking.

**Tests:** 406 unit (1 ignored) + 57 integration tests passing across 11 `tests/` files (2026-08). 63 `.cgdsl` fixture files in `test_games/` (including the five handoff demo games: `blackjack.cgdsl`, `war.cgdsl`, `crazy_eights.cgdsl`, `five_card_draw.cgdsl`, `go_fish.cgdsl`).

### 2.2 Not Implemented

**`todo!()` Panics (4 sites):**

| File | Line | Message | Triggered by |
|------|------|---------|-------------|
| `query/player.rs:246` | `PlayerCollection::Aggregate` | `end game with winner(for all ...)`, `OutOfPlayer` |
| `query/int.rs:173` | `IntCollection::AggregateMemory` | `SizeOf`, `IntCollectionAt` |
| `query/int.rs:266` | `TeamCollection::AggregateMemory` | `SizeOf` of TeamCollection |
| `query/string.rs:60` | `StringCollection::AggregateMemory` | `StringCollectionAt`, `SizeOf` |

**Silent No-Ops / Stubs (11 action variants):**

Token/card-status: `CreateTokenOnLocation`, `FlipAction`, `MoveType::Place` — data model never built.

Bidding/demand: `BidAction`, `BidMemoryAction`, `DemandAction`, `DemandMemoryAction` — semantics undefined.

Game end: `EndType::GameWithWinner` — empty body.

SetMemory collection sub-variants: `PlayerCollection`, `StringCollection`, `TeamCollection`, `IntCollection`, `LocationCollection`, `CardSet` — insert empty defaults instead of evaluating.

**Query — Silent Empty Returns:**

| File:line | Variant | Returns |
|-----------|---------|---------|
| `query/player.rs:277` | `PlayerCollection::AggregateMemory` | `vec![]` |
| `query/player.rs:282` | `PlayerCollection::Memory` | `vec![]` |

**GameData — Known Quirks (documented in invariants.md):**

| Quirk | Invariant | File:line |
|-------|-----------|-----------|
| `add_card` ignores `_location_id` | I-6 | `game_data.rs:156` |
| `resolve_turn` never considers current player | I-13 | `game_data.rs:261` |
| `leave_stage` drains entire stack if stage absent | I-11 | `game_data.rs:252` |
| `add_memory` wrong init for Player/TeamCollection | I-10 | `game_data.rs:280,286` |
| `reset_memory` only affects Int | — | `game_data.rs:305` |
| `CycleAction`/`cycle to next` panics when no eligible *other* player exists | I-13 | `action.rs:309` |
| `ShuffleAction` only shuffles the selected cards in place | — | `action.rs:192` |

### 2.3 Blocked by Missing Dependencies

- **Bidding/Demand**: semantics never specified.
- **GameWithWinner**: depends on scoring completion (now implemented, but GameWithWinner dispatch still empty).
- **Collection memory aggregation**: the 4 `todo!()` sites need multi-owner iteration logic in the evaluator.
- **Card status (FaceUp/FaceDown/Private)**: `FlipAction` exists but cards lack a status field.
- **Tokens**: `CreateTokenOnLocation` / `Place` exist but tokens are not in the data model.

---

## 3. Known Bugs (Front-End Origin)

Bugs in `front_end` (parser, AST, IR lowering) that manifest at engine runtime.

### B-1: `for Y` player collection dropped during IR lowering

**Severity:** Medium. **DSL:** `stage Play for <collection> ...` — any `for Y` where Y is not `current`. Parsed but dropped. All players end up in-stage regardless. `SeqStage` and `SimStage` both produce the same sequential IR.

**Fix:** Carry participant collection into IR payloads. `ir.rs` lines 565-641 (build_seq_stage) and 648-730 (build_sim_stage).

### B-2: Quantifier `Any` todo! remains for winner/OutOfPlayer

**Severity:** Low. Setup and Move-dest quantifiers work. Remaining `todo!()` at `query/player.rs:246` reachable via `end game with winner(for all ...)` / `OutOfPlayer`.

**Fix:** Complete `PlayerCollection::Aggregate` evaluator arm or intercept at engine level (as done for Move-dest quantifiers).

### B-3: SimStage produces identical sequential IR

**Severity:** Low. `build_sim_stage` at `ir.rs:648` has explicit TODO. No simultaneous/parallel execution — all stages are sequential regardless of type.

**Fix:** Front-end: parallel sub-FSMs or a `SimStage`-specific payload. Tied to B-1.

---

*Last updated: 2026-07-28*
