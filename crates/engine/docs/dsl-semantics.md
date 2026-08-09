---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [dsl, semantics, specification, reference, interpretation]
last_validated: 2026-07-28
---

# DSL Semantics Reference

> **Purpose:** documents what every construct in the `.cgdsl` language means to
> the engine interpreter. Each entry maps DSL syntax → engine behavior →
> implementation status. This is the authoritative reference for writing tests:
> every fixture test should assert the semantics documented here.

Status key:
- ✅ **Implemented** — works as documented
- ⚠️ **Implemented with limitations** — works but has known constraints (read the NOTE)
- ❌ **Stub** — parsed but engine does nothing
- ❌ **Not implemented** — parser accepts the construct but IR/engine doesn't handle it

---

## 1. Setup Rules

Setup rules execute in declaration order before the first stage. They populate
`GameData` with players, locations, cards, teams, turn order, combos,
precedences, point maps, and memories. Each rule is lowered to a
`Payload::Action(GameRule::SetUp { ... })` edge.

**NOTE:** The grammar has **no `create` keyword**. Setup rules use bare
keywords: `player`, `team`, `turnorder`, `location`, `card on Loc:`, etc.

### 1.1 `player <name>, <name>, ...`

**DSL:** `player P1, P2, P3`

**Engine:** Calls `GameData::add_player(name)` for each name. Each call appends a
new `Player` to `players` with `score: 0`, `in_game: true`, empty `owner` and
`in_stage`. The player's index is also pushed onto `turn_order` (initial
declaration order). After setup, `turnorder` can override the turn order.

**Status:** ✅ Implemented

---

### 1.2 `team <name> with <players>`

**DSL:** `team T1 with all` / `team T2 with (P:P1, P:P2)`

**Engine:** Calls `quantifier::resolve_player_candidates(&player_collection, gd)`
to resolve the player collection to a `Vec<usize>` of player indices. Pushes a
`Team { name, players: indices }` onto `gd.teams`. `All` quantifier resolves to
all in-game players in declaration order.

**Status:** ✅ Implemented

---

### 1.3 `turnorder <players>` / `turnorder <players> random`

**DSL:** `turnorder all` / `turnorder (P:P2, P:P1) random`

**Engine:** Resolves the player collection to a `Vec<usize>` and assigns it to
`gd.turn_order`, replacing the default declaration-order list. The `random`
variant shuffles the resolved list via `rand::thread_rng()`.

**Status:** ✅ Implemented

---

### 1.4 `location <name> on <owner>`

**DSL:** `location Hand on all` / `location Stock on table`

**Engine:** `resolve_owner_to_names(&owner, gd)` produces a list of owner names
(`"Table"`, `"P1"`, `"P2"`, ...). For each owner name, calls
`gd.add_location(owner_name, Location { name, cards: vec![] })`. When the owner
is `all`, one `Location` is created per resolved player, each owned by that
player. Table-owned locations are globally visible.

**Status:** ✅ Implemented

---

### 1.5 `card on <location>: <key>(<value>, ...) [for <key>(<value>, ...)]*`

**DSL:** `card on Stock: Rank(Ace, Two, Three) for Suit(Hearts, Spades)`

**Engine:** Finds the location by name via `gd.locations.iter().position(|l| l.name == location)`.
Calls `Evaluator::expand_types(&type_expr)` to expand type definitions. The
`for` clause computes the cartesian product: each dimension produces a set of
`(key, value)` pairs; the expander combines them. For each expanded card
`HashMap`, calls `gd.add_card(loc_idx, card)` and pushes the returned ID into
`gd.locations[loc_idx].cards`.

**Status:** ✅ Implemented

---

### 1.6 `combo <name> where <filter>`

**DSL:** `combo TwoOfAKind where Rank same`

**Engine:** Pushes `Combo { name, filter }` onto `gd.combos`. The `filter` is
stored as an AST `FilterExpr` node (lowered from the spanned form). Combos are
not evaluated during play; they are stored for future hand-evaluation
extensions.

**Status:** ✅ Implemented (no execution test yet)

---

### 1.7 `precedence <name> on <key>(<values>)`

**DSL:** `precedence RankOrd on Rank(A, 2, 3, 4, 5, 6, 7, 8, 9, T, J, Q, K)`

**Engine:** Pushes `Precedence { name, key, values }` onto `gd.precedences`.
The `key` is the attribute name (e.g. `"Rank"`) and `values` is the ordered
list (e.g. `["A", "2", ..., "K"]`). Precedences define a total ordering for a
card attribute. Also supports the shorthand `key_value_list` form.

**Status:** ✅ Implemented (no execution test yet)

---

### 1.8 `points <name> on <key>(<k>: <int>, ...)`

**DSL:** `points Values on Rank(A: 1, Two: 2, ..., K: 10)`

**Engine:** Evaluates each `int_expr` via `Evaluator::eval_int` at setup time
(`action.rs:169`). Pushes `PointMap { name, map: HashMap<String, i32> }` onto
`gd.point_maps`. The map keys are compound `"<key>:<value>"` strings (e.g.
`"Rank:A"` → `1`). Also supports the shorthand `key_value_int_list` form.

**Status:** ✅ Implemented (no execution test yet)

---

### 1.9 `memory <name> [<expr>] on <owner>`

**DSL:** `memory M on current` / `memory InitialScore 42 on table` /
`memory Name "Ace" on P:P1`

**Engine:** Calls `gd.add_memory(key, owner, memory_type)` where `key` is
`format!("{}_{}", owner_name, name)`. Stores the entry in a global `HashMap`
(`gd.memories`). The optional type-expression determines the initial
`MemoryValue`:
- `Int { int }` → `MemoryValue::Int(0)` (the value is ignored)
- `String { .. }` → `MemoryValue::String("")`
- None / other → `MemoryValue::Int(0)`

**NOTE:** The grammar has no `with I:` syntax; the type is just a bare
expression: `memory M 42 on P:P1`. `Player` type initializes to `Int(0)`
rather than a player-index variant. `TeamCollection` initializes to `Int(0)`
rather than a collection variant. These are known type mismatches (invariant
I-10). Reads through `eval_player` / `eval_team` on these slots will fail.

**Status:** ✅ Implemented (no execution test yet)

---

### 1.10 `token <quantity> <name> on <location>`

**DSL:** `token 3 Marker on table`

**Engine:** Empty body `{}`. Tokens are not modeled in `GameData`.

**Status:** ❌ Stub

---

## 2. Stage Mechanics

### 2.1 Stage Lifecycle

A stage is a named loop with entry, execution, and exit phases:

1. **Entry:** `GameData::ensure_stage_entered(stage_name)` is called on first
   encounter. Idempotent — no-op if already on `stage_stack`. Marks **all**
   players `in_stage[stage_name] = true`.
2. **Execution:** Each iteration fires a `StageRoundCounter` counter increment
   and dispatches the stage body edges. The stage body may include any action,
   choice, optional, or control-flow rules.
3. **Exit:** The stage's `EndCondition` edge determines whether to continue
   (cycle back) or exit. On exit, `leave_stage(stage_name)` pops the stack
   through and including the named stage.

The IR builder (`build_stage`) lowers a stage to:
```
[entry] → [EndCondition edge]
            ├── false → [StageRoundCounter → body ... → loop back to entry]
            └── true  → [post-stage state]
```

**Status:** ✅ Implemented

---

### 2.2 `stage <name> for <players> <N> times { ... }`

**DSL:** `stage Play for current 2 times { ... }`

**Engine:** Lowers to an `EndCondition::UntilBoolRep` with a built-in
iteration counter that exits after `N` traversals of the `StageRoundCounter`
edge. All players (from the `for` clause — but see known bug B-1) are marked
in-stage.

**Status:** ✅ Implemented

---

### 2.3 `stage <name> until <bool> { ... }`

**DSL:** `stage Play until card_set_empty(Hand) { ... }`

**Engine:** Lowers to `EndCondition::UntilBool { bool_expr }`. Each loop
iteration evaluates the boolean expression; when `true`, exits the stage.

**Status:** ✅ Implemented

---

### 2.4 `stage <name> until end { ... }`

**DSL:** `stage Play until end { ... }`

**Engine:** Lowers to `EndCondition::UntilEnd`. The stage exits when an
`end stage` or `end <stage_name>` action fires within the stage body. The
`EndCondition` edge itself always returns `false` for `UntilEnd` — the exit is
triggered by the action, not by condition evaluation.

**Status:** ✅ Implemented

---

### 2.5 `end stage` / `end <stage_name>`

**DSL:** `end stage` (current) / `end Play` (named)

**Engine:** Dispatched via `ActionRule::EndAction`:
- `EndType::Turn` → `gd.next_player()` (walks turn_order to next eligible)
- `EndType::CurrentStage` → `gd.leave_stage(gd.get_current_stage())`
- `EndType::Stage { stage }` → `gd.leave_stage(stage)`

`leave_stage` pops `stage_stack` through and including the named stage. If the
stage is not found on the stack, the entire stack is drained (invariant I-11).

**Status:** ✅ Implemented (no fixture test yet for explicit `end stage` action)

---

### 2.6 Simultaneous Stages (`SimStage`)

**DSL:** (parsed but not distinguished from `SeqStage` at IR level)

**Engine:** `build_sim_stage` lowers to the exact same sequential IR as
`build_seq_stage`. There is no per-player sub-FSM fan-out. Per-player
participant collections (`for Y`) are dropped during IR lowering (B-1, B-3).

**Status:** ❌ Not implemented — known bug (B-3)

---

## 3. Action Rules

Action rules execute inside stage bodies or trigger rules. They mutate
`GameData` directly.

### 3.1 `move` / `deal` / `exchange`

**DSL:**
```
move top(Hand) face up to Table
deal 3 from top(Stock) private to Hand of all
exchange any from Stock face down to Discard
```

**Engine:** All three route through `execute_cardset_move(from, quantity, status, to, gd)`.
The `from` and `to` are evaluated via `eval_cardset`, returning
`(location_idx, Vec<card_ids>)`. Cards are removed from the source location's
`cards` vec and appended to the destination's. The `quantity` parameter limits
how many cards to move (resolved via `resolve_quantity`). Card `status` is
checked against the move action but not stored on cards (cards have no status
field in `GameData`).

When quantifiers (`all`, `any`, `>= M and <= N`) appear on `from` or `to`,
the quantifier preprocessor intercepts the edge before dispatch and issues
`ChooseCards` / `ChoosePlayer` prompts or builds synthetic fan-out chains.

**Status:** ✅ Implemented

---

### 3.2 `shuffle <cardset>`

**DSL:** `shuffle Stock`

**Engine:** Evaluates the `CardSet`, retrieves the `card_ids` vec from the
resolved location, shuffles in place via `rand::thread_rng()`, and writes the
shuffled vec back to `gd.locations[loc_idx].cards`. If evaluation fails, the
failure is printed to stderr and the location is left unchanged (silent no-op on
error).

**Status:** ✅ Implemented

---

### 3.3 `cycle to <player>`

**DSL:** `cycle to next` / `cycle to P2` / `cycle to current`

**Engine:** Evaluates the `PlayerExpr` via `eval_player`, resolves the name to a
player index, finds that index in `turn_order`, and sets
`gd.current_player = Some(turn_position)`. Panics if the resolved player is not
in `players` or not in `turn_order`.

`next` resolves via `RuntimePlayer::Next` → `eval_player` evaluates to the next
eligible player's name (wrapping turn order). `current` resolves to the current
player's name.

**Status:** ✅ Implemented

---

### 3.4 `set <players> out of <game|stage>`

**DSL:**
```
set P1 out of game
set P2 out of Play
set current out of game
```

**Engine:** Resolves the `Players` to a `Vec<usize>` of player indices. For
each index:
- `OutOf::Game` / `GameSuccessful` / `GameFail` → `gd.set_player_out(idx)` (sets `in_game = false`)
- `OutOf::CurrentStage` → `gd.set_player_stage_flag(idx, current_stage, false)`
- `OutOf::Stage { name }` → `gd.set_player_stage_flag(idx, name, false)`

Out-of-range indices silently no-op (via `Vec::get_mut` returning `None`).

**Status:** ✅ Implemented (no fixture test yet)

---

### 3.5 `end turn` / `end stage` / `end <stage>` / `end game with winner <players>`

| Sub-variant | Behavior |
|-------------|----------|
| `end turn` | `gd.next_player()` — advances to next eligible player in turn order |
| `end stage` | `gd.leave_stage(gd.get_current_stage())` |
| `end <name>` | `gd.leave_stage(name)` |
| `end game with winner <players>` | ❌ Stub — empty body |

`next_player` calls `resolve_turn`, which scans `turn_order` for the next
player with `in_game && in_stage[current_stage]`. If none found,
`current_player` becomes `None` (stuck game, no error — invariant I-13).

**Status:** `end turn` / `end stage` / `end <name>` ✅ Implemented (no fixture test yet).
`end game with winner <players>` ❌ Stub.

---

### 3.6 `set memory` / `reset memory`

**DSL (approximate — see grammar for exact forms):**
```
M is 42                  ← Int
M is "Hello"             ← String
M is P:Player1           ← Player (stored as String)
M is T:TeamA             ← Team
M is (1, 2, 3)           ← IntCollection
M is ("a", "b")          ← StringCollection
M is (P:P1, P:P2)        ← PlayerCollection
reset M
```

**Engine:**
- `SetMemory`: Evaluates the `MemoryType` expression. Inserts the result into
  `gd.memories` under the key `"<CurrentPlayerName>_<memory>"` (i.e., the
  current player is automatically used as the owner prefix —
  `action.rs:267`). Int, String, Player (stored as String), and Team
  variants have full evaluation. PlayerCollection, StringCollection,
  IntCollection, LocationCollection, CardSet, and TeamCollection are stubs
  that insert empty or mismatched defaults.
- `ResetMemory`: Prefixes the memory name with the current player name
  (`action.rs:277`), then calls `gd.reset_memory(&key)` which zeros the
  value if it is `MemoryValue::Int`, silently no-ops otherwise.
- Memory reads (e.g. `&I:M` in a score expression) **require an explicit
  owner** via `&I:M of <owner>` or `(&I:M of <owner>)`. Bare `&I:M`
  (without `of`) is valid in the grammar but fails at runtime
  (`query/mod.rs:197`).

**Status:** ⚠️ Int, String, Player, Team implemented. Collection variants are
stubs (insert empty defaults). TeamCollection has a type mismatch.
`reset_memory` only affects Int memories. Memory keys are prefixed with the
current player for write operations, and read operations require explicit
owner syntax.

---

### 3.7 `flip <cardset> to <status>`

**DSL:** `flip top(Hand) to face up`

**Engine:** Empty body `{}`. Cards have no status field in `GameData`.

**Status:** ❌ Stub — cards lack a status data model

---

### 3.8 `place <token> from <location> to <location>`

**DSL:** `place Marker from Hand to Table`

**Engine:** Empty body `{}`. Tokens are not modeled in `GameData`.

**Status:** ❌ Stub — tokens not in data model

---

### 3.9 `bid <quantity>` / `bid <quantity> on <memory> of <owner>`

**DSL:** `bid 5` / `bid &I:n on pot of Table`

**Engine:** Empty body `{}`. Bidding mechanics never specified.

**Status:** ❌ Stub — semantics undefined

---

### 3.10 `demand <type>` / `demand <type> as <memory>`

**DSL:** `demand top(Hand)` / `demand "hello" as m`

**Engine:** Empty body `{}`. Demand mechanics never specified.

**Status:** ❌ Stub — semantics undefined

---

## 4. Scoring Rules

Scoring rules are lowered to `Payload::Action(GameRule::Scoring { ... })` edges
and dispatched through `execute_scoring_rule`.

### 4.1 `score <int_expr> to <players>`

**DSL:** `score 10 to P1` / `score (&I:a + 5) to all`

**Engine:**
1. `Evaluator::eval_int(&int_expr, gd)` → `i32`
2. `Evaluator::resolve_players(&players, gd)` → `Vec<usize>`
3. For each index: `gd.players[idx].score += value`

**Status:** ✅ Implemented

---

### 4.2 `score <int_expr> to <memory> of <players>`

**DSL:** `score 5 to M of P1` / `score &I:n to ScoreSlot of all`

**Engine:**
1. `Evaluator::eval_int(&int_expr, gd)` → `i32`
2. Resolves players to player indices.
3. For each index `i`: writes `MemoryValue::Int(value)` into
   `gd.memories["<player_name>_<memory>"]` (per-player keyed —
   `action.rs:370`).

**NOTE:** Player scores are **not** modified by this rule; only the memory
value is written. Each resolved player gets its own key, so multi-player
targets use separate slots (unlike the old global-slot model).

**Status:** ✅ Implemented — per-player keyed, no last-write-wins issue

---

### 4.3 `winner is <players>`

**DSL:** `winner is P1` / `winner is (P:P1, P:P2)`

**Engine:**
1. `Evaluator::resolve_players(&players, gd)` → `Vec<usize>` (the winners)
2. For all players NOT in the winner set: `gd.set_player_out(idx)`
3. Winners remain `in_game = true`

**Status:** ✅ Implemented

---

### 4.4 `winner is <extrema> <winner_type>`

**DSL:**
```
winner is highest score
winner is lowest score
winner is highest position
winner is lowest position
winner is highest m          ← memory-backed
```

**Engine:** For each in-game player, computes a comparable value based on
`WinnerType`:

| WinnerType | Value per player `i` |
|------------|---------------------|
| `Score` | `gd.players[i].score` (as `usize`) |
| `Position` | index of player `i` in `gd.turn_order` (0-based). Missing → `usize::MAX` |
| `Memory { key }` | reads `gd.memories[key]`, expects `Int(n)` → `n as usize`, else `0` |

Then finds the target value: `Max` → highest across all in-game players, `Min`
→ lowest. Eliminates all players whose value ≠ target via `set_player_out`.
Ties are honored — all players matching the target value remain in game.

**NOTE:** `Position` is interpreted as turn-order index (lower = earlier in
turn). This may not match the intended DSL semantics but is the only
well-defined positional value available. `Memory` reads from the global memory
slot; since it's the same value for all players, it is only useful when the
memory has been set with the score of the player(s) to be compared.

**Status:** ⚠️ Implemented — Position and Memory interpretations may differ from
DSL intent

---

## 5. Control Flow

### 5.1 `choose { A B or C D }`

**DSL:** `choose { move top(Hand) face down to Table or deal 1 from Stock to Hand }`

**Engine:** Each `or`-separated option is a **sequence** of flow components
(`choose { A B or C }` = two options: `[A, B]` and `[C]` — fixed 2026-08-09,
previously every component became its own option). Lowers to a
`Payload::Choice` state with one edge per option; the chosen option's whole
sequence executes in order. `edge_labels` on the IR resolve human-readable
labels from the target state's first payload (action descriptions, condition
text, etc.). The interpreter issues
`NeedsInput(InputType::Choice { options, max_index })`. The player returns
`InputKind::Choice { idx }` (0-based). The engine takes `edges[idx]` and
executes it. Out-of-range indices silently stall (invariant I-8).

**Status:** ✅ Implemented

---

### 5.2 `optional { A }`

**DSL:** `optional { deal 1 from Stock private to Hand }`

**Engine:** Lowers to a `Payload::Optional` state with two edges: `edges[0]` =
accept (execute the optional body), `edges[1]` = decline (skip). The interpreter
issues `NeedsInput(InputType::Optional(prompt))`. The player returns
`InputKind::OptionalAccept` (idx→0, accept) or `InputKind::OptionalDecline`
(idx→1, decline).

**Status:** ✅ Implemented

---

### 5.3 `if (<bool_expr>) { ... }`

**DSL:**
```
if (Hand empty) { deal 1 from Stock private to Hand }
```

**Engine:** Lowers to a `Payload::Condition { expr, negated }` state with
exactly 2 edges. The interpreter evaluates `eval_bool(&expr, gd)`. The dispatch
formula is: `should_take_else = result != negated`. If true → `edges[0]`
(if-body). If false → `edges[1]` (skip/else).

See invariant I-3 for the inverted edge-indexing relationship between
`Condition` and `EndCondition`.

**NOTE:** `unless` does **not** exist in the grammar. Use `if (not (<expr>))`
instead.

**Status:** ✅ Implemented

---

### 5.4 Trigger Rules

**DSL:**
```
trigger {
    move top(Stock) private to Hand
}
```

Triggers are top-level `flow_component` rules that fire immediately when
encountered. They can also appear as `on enter:` blocks in stage
definitions (lowered identically).

**Engine:** The IR builder wraps trigger-rule bodies in `Payload::Trigger` edges.
In the interpreter, `Payload::Trigger` dispatches identically to `Action` edges
— it calls `execute_edge(edge)` which means the edge payload is executed via
`action::execute()`. The Trigger payload itself is a catch-all in
`action::execute` (`_ => {}`), meaning the interpreter dispatches the
sub-edge's payload directly.

**Status:** ✅ Implemented

---

### 5.5 Edge Ordering Convention (I-3)

`Condition` and `EndCondition` states both require exactly 2 edges but use
inverted indexing:

| Payload | edges[0] means | edges[1] means |
|---------|---------------|---------------|
| `Condition { negated: false }` | take the if-body | skip (else) |
| `Condition { negated: true }` | skip (unless-body) | take the unless-body |
| `EndCondition { ... }` | exit the stage | continue (loop back) |

Assertion: `Condition` edge 0 is the "false"/skip branch under the
`should_take_else` formula. `EndCondition` edge 0 is the "true"/exit branch
under the `should_exit` formula. Any change to the IR builder's edge ordering or
the interpreter dispatch must mirror both.

**Status:** ✅ Implemented (documented invariant)

---

## 6. Quantifiers

Quantifiers expand a single edge into multiple synthetic edges at runtime. The
quantifier preprocessor (`quantifier.rs` + `interpreter/quant_driver.rs`)
detects quantifier-bearing edges before dispatch and either issues a player
prompt or builds a synthetic fan-out chain.

### 6.1 `all` (DestPlayerAll)

**DSL:** `deal 1 from top(Stock) to Hand of all`

**Engine:** Resolves `all` to all in-game player names via
`resolve_player_candidates`. Builds a fan-out chain of synthetic edges — one
per player — where each edge substitutes the player name into the destination
owner. The chain links: `[synthetic_N → synthetic_N+1 → ... → original_edge.to]`.
The FSM advances through the chain one step at a time, dispatching each
per-player edge through `action::execute`.

**Status:** ✅ Implemented

---

### 6.2 `any` (DestPlayerAny)

**DSL:** `deal 1 from top(Stock) to Hand of any`

**Engine:** Resolves `any` to a list of candidate player names. Issues a
`NeedsInput(InputType::ChoosePlayer { candidates, prompt })` request. The
player selects a candidate by 0-based index. On resume, a single synthetic edge
is created with the chosen player name substituted, and `current_state`
advances through it.

**Status:** ✅ Implemented

---

### 6.3 `any` (SrcCardsAnyOrRange)

**DSL:** `deal any from Stock private to Hand`

**Engine:** Evaluates the source `CardSet` to produce a list of candidate card
IDs. Issues a `NeedsInput(InputType::ChooseCards { display, min, max, prompt })`
request (for `any`, `min=1`, `max=len`). The player selects indices into
`display`. On resume, the chosen card IDs are written to the synthetic memory
slot (`SYNTH_MEMORY_KEY`), a replacement edge is built that reads from that
memory, and `current_state` advances.

**Status:** ✅ Implemented

---

### 6.4 `>= M and <= N` (IntRange)

**DSL:** `move >= 1 and <= 3 from Stock face down to Discard`

**Engine:** Same flow as `any`, but `min`/`max` come from the `IntRange`. If the
player selects fewer than `min` or more than `max` cards, the controller
re-prompts with a validation error message. The interpreter's resume path also
validates and can issue a new `NeedsInput` on failure.

**Status:** ✅ Implemented

---

### 6.5 `any` in Setup Rules (Rejected)

**DSL:** `location Hand on any`

**Engine:** Before dispatching a setup rule edge, the quantifier guard
(`setup_contains_any`) checks whether any element collection uses
`Quantifier::Any`. If yes, returns
`StepResult::Error("quantifier 'any' is not supported in setup rules")`
before any `GameData` mutation occurs (invariant I-20).

**Status:** ✅ Implemented (rejected with error)

---

### 6.6 Synthetic State Lifecycle

- **Allocation** (I-16): Synthetic `StateID`s are allocated from `u32::MAX - 1`
  decrementing via `wrapping_sub`. Never collide with real IR ids (allocated
  from 0 upward).
- **Overlay** (I-17): `pending_overlay` maps synthetic ids to replacement edges.
  Only keyed by synthetic ids — real IR ids are never inserted. The overlay
  shadows `ir.states` only during quantifier dispatch.
- **Memory slot** (I-18): `SYNTH_MEMORY_KEY` (`"__quantifier_overlay_cards"`)
  is written just before a card-choice resume edge dispatches and removed at the
  top of `step()` when the FSM returns to a real IR state.
- **Resume guard** (I-19): `pending_quant.state` must equal `current_state` for
  resume to fire; mismatch returns `None` without popping either the pending
  quant or the buffered input.

**Status:** ✅ Implemented

---

## 7. Known Gaps

| Construct | Status | Reason |
|-----------|--------|--------|
| `unless` | ❌ | Not in grammar. Use `if (not (<expr>))`. |
| `SimStage` (simultaneous play) | ❌ | `build_sim_stage` produces same IR as `SeqStage`. Per-player sub-FSMs not built. B-3. |
| `for <Y>` clause in stage | ❌ | Parsed into AST, dropped during IR lowering. All players always in-stage. B-1. |
| `token <quantity> <name> on <location>` | ❌ | Tokens not modeled in `GameData`. Empty body. |
| `flip <cardset> to <status>` | ❌ | Cards have no status field. Empty body. |
| `place <token> from ... to ...` | ❌ | Tokens not modeled. Empty body. |
| `bid <quantity>` / `bid memory ...` | ❌ | Semantics never specified. Empty body. |
| `demand <type>` / `demand memory ...` | ❌ | Semantics never specified. Empty body. |
| `end game with winner <players>` | ❌ | `GameWithWinner` dispatch is empty. |
| `owner of highest/lowest <memory>` | ⚠️ | Key-order bug in `query/player.rs:93`: builds `<mem>_<player>` instead of `<player>_<mem>`, can never match writes from `set_memory`/`score … to memory`. |
| `score N to <memory> of <players>` | ✅ | Per-player keyed; each resolved player gets its own `<player>_<mem>` slot (no last-write-wins). |
| `winner is highest/lowest position` | ⚠️ | Interpreted as turn-order index. May not match DSL intent. |
| `set memory` collection variants | ⚠️ | PlayerCollection/StringCollection/IntCollection/LocationCollection/CardSet insert empty defaults instead of evaluating. TeamCollection inserts Int(0) (type mismatch). |
| `reset memory` on non-Int | ⚠️ | Only zeros `MemoryValue::Int`; silently ignores other types. `reset` prefixes key with current player name. |
| `add_memory` Player/TeamCollection init | ⚠️ | I-10: Player → Int(0), TeamCollection → Int(0). Type mismatch. |
| `Aggregate` in query evaluator | ⚠️ | `todo!()` panic in `query/player.rs:246`. Reachable via `end game with winner(for all ...)` / `OutOfPlayer`. |
| `AggregateMemory` in query | ⚠️ | `todo!()` in `int.rs:173`, `int.rs:266`, `string.rs:60`. Multi-owner memory aggregation not implemented. |
| `PlayerCollection::Memory` / `AggregateMemory` | ⚠️ | Silent empty `vec![]` return in `query/player.rs:277,282`. Memory-backed player collections not supported. |
