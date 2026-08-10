---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [dsl, semantics, specification, reference, interpretation]
last_validated: 2026-08-10
---

# DSL Semantics Reference

> **Purpose:** documents what every construct in the `.cgdsl` language *means*
> to the engine interpreter: DSL syntax → engine behaviour.
>
> **Implementation status is NOT tracked here** — it lives in
> [`dsl-completeness.md`](./dsl-completeness.md) (per-construct status table).
> Known bugs and divergences from the intended design live in
> [`engine-vs-design.md`](./engine-vs-design.md). This page is the *wanted*
> semantics reference: fixture tests should assert the semantics documented
> here.

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

---

### 1.2 `team <name> with <players>`

**DSL:** `team T1 with all` / `team T2 with (P:P1, P:P2)`

**Engine:** Resolves the player collection to player indices and pushes a
`Team { name, players: indices }` onto `gd.teams`. An `all` quantifier resolves
to all in-game players in declaration order.

---

### 1.3 `turnorder <players>` / `turnorder <players> random`

**DSL:** `turnorder all` / `turnorder (P:P2, P:P1) random`

**Engine:** Resolves the player collection to a `Vec<usize>` and assigns it to
`gd.turn_order`, replacing the default declaration-order list. The `random`
variant shuffles the resolved list via `rand::thread_rng()`.

---

### 1.4 `location <name> on <owner>`

**DSL:** `location Hand on all` / `location Stock on table`

**Engine:** `resolve_owner_to_names(&owner, gd)` produces a list of owner names
(`"Table"`, `"P1"`, `"P2"`, ...). For each owner name, calls
`gd.add_location(owner_name, Location { name, cards: vec![] })`. When the owner
is `all`, one `Location` is created per resolved player, each owned by that
player. Table-owned locations are globally visible.

---

### 1.5 `card on <location>: <key>(<value>, ...) [for <key>(<value>, ...)]*`

**DSL:** `card on Stock: Rank(Ace, Two, Three) for Suit(Hearts, Spades)`

**Engine:** Finds the location by name, calls `Evaluator::expand_types(&type_expr)`
to expand type definitions. The `for` clause computes the cartesian product:
expansion is **rank-major with the innermost dimension iterated last** (e.g.
`Rank(Ace, Two) for Suit(D, C)` yields `Ace-D, Ace-C, Two-D, Two-C`). For each
expanded card `HashMap`, calls `gd.add_card(loc_idx, card)` and pushes the
returned ID into `gd.locations[loc_idx].cards`. **Without a `shuffle`, the
creation order IS the deck order** — deterministic fixtures rely on this.

---

### 1.6 `combo <name> where <filter>`

**DSL:** `combo Pair where same Rank`

**Engine:** Pushes `Combo { name, filter }` onto `gd.combos`. The `filter` is
stored as an AST `FilterExpr` node. Combos are evaluated **group-wise** at read
time (like a `where`-clause): `Pair in Hand` returns exactly the cards of
`Hand` matching the combo filter (e.g. the paired cards for `same Rank`).

---

### 1.7 `precedence <name> on <key>(<values>)`

**DSL:** `precedence RankOrd on Rank(A, 2, 3, 4, 5, 6, 7, 8, 9, T, J, Q, K)`

**Engine:** Pushes `Precedence { name, key, values }` onto `gd.precedences`.
The `key` is the attribute name (e.g. `"Rank"`) and `values` is the ordered
list (e.g. `["A", "2", ..., "K"]`), low → high. Precedences define a total
ordering for a card attribute. Also supports the shorthand `key_value_list`
form.

---

### 1.8 `points <name> on <key>(<k>: <int>, ...)`

**DSL:** `points Values on Rank(A: 1, Two: 2, ..., K: 10)`

**Engine:** Evaluates each `int_expr` at setup time. Pushes
`PointMap { name, map: HashMap<String, i32> }` onto `gd.point_maps`. The map
keys are compound `"<key>:<value>"` strings (e.g. `"Rank:A"` → `1`). Also
supports the shorthand `key_value_int_list` form.

---

### 1.9 `memory <name> [<expr>] on <owner>`

**DSL:** `memory M on current` / `memory InitialScore 42 on P:P1` /
`memory Name "Ace" on P:P1`

**Engine:** Calls `gd.add_memory(key, owner_name, memory_type, initial)`
where `key` is `format!("{}_{}", owner_name, name)`. Stores the entry in a
global `HashMap` (`gd.memories`). Since 2026-08-10 the type-expression is
**evaluated at setup time** and honoured as the initial value (previously the
int value was silently dropped and every type defaulted to `Int(0)`):

- `Int { int }` → `MemoryValue::Int(eval_int(int))` (e.g. `memory Pot 100 on
  table` really initialises `Table_pot` to 100)
- `String { string }` → the evaluated string
- `Player { player }` → the evaluated player **name** (as a `String`,
  matching the `SetMemory` storage convention); a player-owned slot
  initialises to its **own owner** instead (so `&P:X of P:Pi` reads the
  slot's own player) — I-10 fixed
- `Team { team }` → the evaluated team name
- Collections (`PlayerCollection`, `StringCollection`, `IntCollection`,
  `TeamCollection`, `LocationCollection`, `CardSet`) → their evaluated
  contents (empty when the collection has no members)
- `None` / other → `MemoryValue::Int(0)`

**NOTE:** The grammar has no `with I:` syntax; the type is just a bare
expression: `memory M 42 on P:P1`.

---

### 1.10 `token <quantity> <name> on <location>`

**DSL:** `token 3 Marker on table`

**Engine:** Empty body. Tokens are not modeled in `GameData` (see
`engine-vs-design.md` D-6).

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

The IR builder lowers a stage to:
```
[entry] → [EndCondition edge]
            ├── false → [StageRoundCounter → body ... → loop back to entry]
            └── true  → [post-stage state]
```

**NOTE:** the end condition is checked at stage *entry* — effects produced
mid-body (e.g. "a player emptied their hand") are observed on the next entry
(see `engine-vs-design.md` D-2).

---

### 2.2 `stage <name> for <players> <N> times { ... }`

**DSL:** `stage Play for current 2 times { ... }`

**Engine:** The stage body runs exactly `N` times (the loop-back
`StageRoundCounter` is compared against `N`). **The `for <players>` clause is
parsed but currently dropped during IR lowering** — all players are marked
in-stage regardless (known bug B-1 / P-1).

---

### 2.3 `stage <name> until <bool> { ... }`

**DSL:** `stage Play until Hand empty { ... }`

**Engine:** Each loop iteration evaluates the boolean expression at entry;
when `true`, the stage exits. The body runs while the condition is `false`.

---

### 2.4 `stage <name> until end { ... }`

**DSL:** `stage Play until end { ... }`

**Engine:** The stage loops forever until an `end stage` / `end <stage_name>`
action fires within the stage body. The exit is triggered by the action's IR
jump, not by condition evaluation.

---

### 2.6 Ineligible-player skip & stage auto-end (2026-08-10)

A player who is **out of the game** or **out of the current stage** is never
offered input, and none of their instructions run:

- Every *skippable* edge (moves, scores, conditions, choices, optionals,
  triggers, quantifier sites) is **advanced through without executing** while
  the current player is ineligible.
- Only **cycle actions** (`cycle to ...`), **end actions** (`end turn` /
  `end stage` / `end <name>`), and the stage bookkeeping (end-condition
  evaluation, round counter, stage exit) still execute — so `cycle to next`
  keeps moving the turn to the next eligible player, and the stage loops
  normally ("the stage skips all instructions except cycle actions until the
  end of the stage; NOT `end stage` — the stage loops again with the new
  player").
- The skip is computed per step and stage-scoped: without a current stage
  (setup, top-level rules) nothing is skipped.

**Auto-end:** at every stage end-condition evaluation the engine also exits
the stage when (a) **no players remain in the game** — the game then runs
out to the goal with an **empty winner set** — or (b) **no players remain in
the stage** (everyone was set out of it). This replaces the former
`cycle to next` error and the silent stuck-game paths.

**`next` / `previous` eligibility (I-13 / D-12, fixed 2026-08-10):**
`next` and `previous` skip ineligible players in their scan direction and
wrap onto the **current player itself** when it is the only eligible one
(`cycle to next` in a 1-player endgame keeps the turn). `cycle to next` with
**no** eligible player at all (not even the current one) is a **no-op** —
the auto-end above terminates the stage. `end turn` behaves the same.

---

### 2.5 `end stage` / `end <stage_name>`

**DSL:** `end stage` (current) / `end Play` (named)

**Engine:** Dispatched via `ActionRule::EndAction`:
- `EndType::Turn` → `gd.next_player()` (walks turn_order to next eligible)
- `EndType::CurrentStage` → `gd.leave_stage(gd.get_current_stage())`
- `EndType::Stage { stage }` → `gd.leave_stage(stage)`

`leave_stage` pops `stage_stack` through and including the named stage. If the
stage is not found on the stack, the entire stack is drained (invariant I-11).

---

### 2.7 Simultaneous Stages (`SimStage`)

**DSL:** (parsed but not distinguished from `SeqStage` at IR level)

**Engine:** `build_sim_stage` lowers to the exact same sequential IR as
`build_seq_stage`. There is no per-player sub-FSM fan-out (known bug B-3 / P-2).

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
how many cards to move (resolved via `resolve_quantity` against the live
state). Card `status` is parsed but **ignored** — cards carry no status
behaviour until the card-encryption work (`engine-vs-design.md` §1b).

When quantifiers (`all`, `any`, `>= M and <= N`) appear on `from` or `to`,
the quantifier preprocessor intercepts the edge before dispatch and issues
`ChooseCards` / `ChoosePlayer` prompts or builds synthetic fan-out chains.

**Combo-source moves** (`move <combo> in <pile> ...`, e.g. laying down a
Rummy set): the preprocessor prompts the player to choose cards from the
*whole* pile and then **validates the choice against the combo's filter**,
re-prompting on mismatch — so `combo Set where (same Rank and size >= 3)`
rejects a two-of-a-kind selection. Combine with a stage loop to lay down
everything: `stage Laydown for current until Set in Hand empty { move Set in
Hand of current private to Table }` (`until <combo> in <pile> empty` is a
valid end condition — a combo group is a cardset).

**NOTE:** an empty source set is a no-op (nothing moves, the destination is
not evaluated); a `where`-filtered destination with no matches resolves to the
base location of the groupable (D-11, fixed 2026-08-09).

---

### 3.2 `shuffle <cardset>`

**DSL:** `shuffle Stock`

**Engine:** Evaluates the `CardSet`, then shuffles **only the selected cards in
place** within their location — unselected cards stay put (e.g.
`shuffle top 3 of Deck` does not discard the rest of the pile). Uses
`rand::thread_rng()`. Evaluation failures are recoverable errors.

---

### 3.3 `cycle to <player>`

**DSL:** `cycle to next` / `cycle to P:P2` / `cycle to current`

**Engine:** Evaluates the `PlayerExpr`, resolves the name to a player index,
finds that index in `turn_order`, and sets
`gd.current_player = Some(turn_position)`. Resolution failures (unknown player,
player not in `turn_order`) are **recoverable errors**.

`next` resolves via `RuntimePlayer::Next` → the next eligible player's name
(wrapping turn order, skipping players who are out of game or out of the
current stage). Since 2026-08-10, with no eligible *other* player the turn
**wraps onto the current player** when it is still eligible (I-13 relaxed —
elimination games no longer need `if (size(playersin) >= 2)` guards), and
`cycle to next` with **no eligible player at all** is a **no-op** (the stage's
auto-end, §2.6, terminates the game). `previous` mirrors `next` with a reverse
scan and the same self-wrap (D-12 fixed).

---

### 3.4 `set <players> out of <game|stage>`

**DSL:**
```
set P:P1 out of game
set P:P2 out of Play
set current out of game
```

**Engine:** Resolves the `Players` to player indices (unknown players or
unevaluable expressions are recoverable errors). For each index:
- `OutOf::Game` / `GameSuccessful` / `GameFail` → `gd.set_player_out(idx)` (sets `in_game = false`)
- `OutOf::CurrentStage` → `gd.set_player_stage_flag(idx, current_stage, false)`
- `OutOf::Stage { name }` → `gd.set_player_stage_flag(idx, name, false)`

`GameSuccessful` and `GameFail` behave identically to `Game` (no
success/fail outcome is tracked — see `engine-vs-design.md` D-9).

---

### 3.5 `end turn` / `end stage` / `end <stage>` / `end game with winner <players>`

| Sub-variant | Behavior |
|-------------|----------|
| `end turn` | `gd.next_player()` — advances to next eligible player in turn order |
| `end stage` | `gd.leave_stage(gd.get_current_stage())` |
| `end <name>` | `gd.leave_stage(name)` |
| `end game with winner <players>` | IR jumps straight to the goal state (game ends); the action arm itself is an empty TODO — the jump makes the stub harmless |

`next_player` calls `resolve_turn`, which scans `turn_order` for the next
player with `in_game && in_stage[current_stage]`. If none found,
`current_player` becomes `None` (stuck game, no error — invariant I-13).

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
- `SetMemory`: Evaluates the `MemoryType` expression (all variants —
  collections are evaluated to their real contents since 2026-08-10;
  previously they inserted typed empty defaults). Inserts the result into
  `gd.memories` under the key `"<Owner>_<memory>"`.
- **Write-owner resolution (D-14, fixed 2026-08-10):** the write rules have
  no `of <owner>` clause in the grammar, so the target owner is resolved as:
  1. the **declared owner** — if exactly one existing slot ends in
     `_{memory}` (e.g. `memory pot on table` → `Table_pot` exists), that
     owner wins (`pot is 5` writes `Table_pot`);
  2. else the **current player** (the legacy bridge);
  3. else a recoverable error (`SetMemoryNoCurrentPlayer` /
     `ResetMemoryNoCurrentPlayer`).
- `ResetMemory`: resets the slot to its **typed zero** for every variant
  (2026-08-10; previously only `Int` was reset and everything else silently
  no-oped): `Int`→0, `String`→`""`, `Team`→`""`, collections→empty.
- Bare memory reads (`&I:M` without `of <owner>`) resolve through the same
  owner resolution as writes (declared owner → current player → error) so
  reads and writes agree (P-4, fixed 2026-08-10).

---

### 3.7 `bid <quantity> on <memory> of <owner>` — the numeric input prompt (2026-08-10)

**DSL:**
```
bid any on Pot of table                  ← prompt for any number
bid >= 1 and <= 10 on Bet of table       ← prompt, bounded 1..=10
bid 5 on Pot of table                    ← literal: write 5, no prompt
```

**Engine:** `bid <qty> on <memory> of <owner>` = "prompt the current player
for a number and store it in the owner's memory slot":

- `Quantity::Int` (literal or runtime int expression) writes the evaluated
  value directly without prompting.
- `any` / `>= M and <= N` issue `NeedsInput(InputType::Number { min, max,
  prompt })` (new 2026-08-10); the answer is `Input::Number { value }`,
  bounds are validated and out-of-range answers are rejected and re-asked
  (both by the controller for `Player`-sourced input and by the
  interpreter's resume path). The resumed edge carries the literal.
- The write targets `{owner}_{memory}` via `resolve_owner_to_name` (Table,
  player, or team name).
- A plain `bid <qty>` **without** a memory target is a recoverable error
  (`BidWithoutMemoryTarget`) — previously a silent no-op (D-7, partially
  fixed: the memory form now has semantics; `demand` remains undefined).

---

### 3.8 `flip <cardset> to <status>`

**DSL:** `flip top(Hand) to face up`

**Engine:** No-op by design until card encryption lands — flipping a card is
(de)encrypting its face. The per-card status slot (`GameData::card_statuses`)
exists but is unused (see `engine-vs-design.md` §1b).

---

### 3.9 `place <token> from <location> to <location>`

**DSL:** `place Marker from Hand to Table`

**Engine:** No-op. Tokens are not modeled in `GameData`.

---

### 3.10 `demand <type>` / `demand <type> as <memory>`

**DSL:** `demand top(Hand)` / `demand "hello" as m`

**Engine:** No-op. Demand mechanics never specified (see `engine-vs-design.md`
D-7).

---

## 4. Scoring Rules

Scoring rules are lowered to `Payload::Action(GameRule::Scoring { ... })` edges
and dispatched through `execute_scoring_rule`.

### 4.1 `score <int_expr> to <players>`

**DSL:** `score 10 to P:P1` / `score (&I:a + 5) to all`

**Engine:**
1. `Evaluator::eval_int(&int_expr, gd)` → `i32`
2. `Evaluator::resolve_players(&players, gd)` → player indices
3. For each index: `gd.players[idx].score += value`

---

### 4.2 `score <int_expr> to <memory> of <players>`

**DSL:** `score 5 to M of P:P1` / `score &I:n to ScoreSlot of all`

**Engine:**
1. `Evaluator::eval_int(&int_expr, gd)` → `i32`
2. Resolves players to player indices.
3. For each index `i`: writes `MemoryValue::Int(value)` into
   `gd.memories["<player_name>_<memory>"]` (per-player keyed).

**NOTE:** Player scores are **not** modified by this rule; only the memory
value is written. Each resolved player gets its own key, so multi-player
targets use separate slots (no last-write-wins).

---

### 4.3 `winner is <players>`

**DSL:** `winner is P:P1` / `winner is (P:P1, P:P2)`

**Engine:**
1. `Evaluator::resolve_players(&players, gd)` → player indices (the winners)
2. For all players NOT in the winner set: `gd.set_player_out(idx)`
3. Winners remain `in_game = true`

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
| `Position` | index of player `i` in `gd.turn_order` (0-based). Players **absent from `turn_order` are excluded** from the comparison (D-10, fixed 2026-08-10 — previously they scored `usize::MAX`, letting a non-participant win `lowest position`) |
| `Memory { key }` | reads `gd.memories["<player_name>_<key>"]`. Players **without the slot are skipped** (D-13, fixed 2026-08-10 — previously treated as 0); a **non-Int** slot is a recoverable error (`WinnerMemoryNotInt`) |

Then finds the target value: `Max` → highest across all in-game players, `Min`
→ lowest. Eliminates all players whose value ≠ target via `set_player_out`.
Ties are honored — all players matching the target value remain in game. With
no in-game players (or none left after exclusion), nothing is eliminated.

**NOTE:** `Position` is interpreted as turn-order index (lower = earlier in
turn). `Memory` reads the **per-player** slot (owner-prefixed), so it is only
useful when the memory was written per player (e.g. via `score ... to memory`).

---

## 5. Control Flow

### 5.1 `choose { A B or C D }`

**DSL:** `choose { move top(Hand) face down to Table or deal 1 from Stock to Hand }`

**Engine:** Each `or`-separated option is a **sequence** of flow components
(`choose { A B or C }` = two options: `[A, B]` and `[C]`). Lowers to a
`Payload::Choice` state with one edge per option; the chosen option's whole
sequence executes in order. `edge_labels` on the IR resolve human-readable
labels from the target state's first payload (action descriptions, condition
text, etc.). The interpreter issues
`NeedsInput(InputType::Choice { options, max_index })`. The player returns
`InputKind::Choice { idx }` (0-based). Out-of-range indices silently stall
(invariant I-8).

---

### 5.2 `optional { A }`

**DSL:** `optional { deal 1 from Stock private to Hand }`

**Engine:** Lowers to a `Payload::Optional` state with two edges: `edges[0]` =
accept (execute the optional body), `edges[1]` = decline (skip — nothing runs,
and no else-branch exists, see `engine-vs-design.md` D-3). The interpreter
issues `NeedsInput(InputType::Optional(prompt))`. The player returns
`InputKind::OptionalAccept` or `InputKind::OptionalDecline`.

---

### 5.3 `if (<bool_expr>) { ... }`

**DSL:**
```
if (Hand empty) { deal 1 from Stock private to Hand }
```

**Engine:** Lowers to a `Payload::Condition { expr, negated }` state with
exactly 2 edges. The interpreter evaluates `eval_bool(&expr, gd)`. The dispatch
formula is: `should_take_else = result != negated`. If true → `edges[0]`
(if-body). If false → `edges[1]` (skip).

See invariant I-3 for the inverted edge-indexing relationship between
`Condition` and `EndCondition`.

**NOTE:** `unless` does **not** exist in the grammar. Use `if (not <expr>)` —
parentheses around the inner bool do **not** parse (`not (X)` is rejected by
the PEG grammar; write `not Hand empty`, `not current out of game`). A leading
`not` before a combo group binds to the boolean, not the combo
(`not Book in Hand of current empty` = "the Book cards are not empty" —
fixed 2026-08-09, F-15).

---

### 5.4 Trigger Rules

**DSL:**
```
trigger {
    move top(Stock) private to Hand
}
```

Triggers are top-level `flow_component` rules that fire immediately when
encountered.

**Engine:** The IR builder wraps trigger-rule bodies in `Payload::Trigger`
edges; the interpreter advances through them without prompting and the body
executes once.

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

---

## 6. Quantifiers

Quantifiers expand a single edge into multiple synthetic edges at runtime. The
quantifier preprocessor (`quantifier.rs` + `interpreter/quant_driver.rs`)
detects quantifier-bearing edges before dispatch and either issues a player
prompt or builds a synthetic fan-out chain.

### 6.1 `all` (DestPlayerAll)

**DSL:** `deal 1 from top(Stock) to Hand of all`

**Engine:** Resolves `all` to all in-game player names. Builds a fan-out chain
of synthetic edges — one per player — where each edge substitutes the player
name into the destination owner. The chain links:
`[synthetic_N → synthetic_N+1 → ... → original_edge.to]`. The FSM advances
through the chain one step at a time, dispatching each per-player edge through
`action::execute`. Dealing is therefore **sequential** (each player's edge sees
the deck after the previous player's edge moved cards).

---

### 6.2 `any` (DestPlayerAny)

**DSL:** `deal 1 from top(Stock) to Hand of any`

**Engine:** Resolves `any` to a list of candidate player names. Issues a
`NeedsInput(InputType::ChoosePlayer { candidates, prompt })` request. The
player selects a candidate by 0-based index. On resume, a single synthetic edge
is created with the chosen player name substituted, and `current_state`
advances through it.

---

### 6.3 `any` (SrcCardsAnyOrRange)

**DSL:** `deal any from Stock private to Hand`

**Engine:** Evaluates the source `CardSet` to produce a list of candidate card
IDs. Issues a `NeedsInput(InputType::ChooseCards { display, min, max, prompt })`
request (for `any`, `min=1`, `max=len`). The player selects indices into
`display`. On resume, the chosen card IDs are written to the synthetic memory
slot (`SYNTH_MEMORY_KEY`, stored owner-prefixed as `Table_…`), a replacement
edge is built that reads from that memory, and `current_state` advances.

---

### 6.4 `>= M and <= N` (IntRange)

**DSL:** `move >= 1 and <= 3 from Stock face down to Discard`

**Engine:** Same flow as `any`, but `min`/`max` come from the `IntRange`. If the
player selects fewer than `min` or more than `max` cards, the controller
re-prompts with a validation error message. The interpreter's resume path also
validates and can issue a new `NeedsInput` on failure.

---

### 6.5 `any` in Setup Rules (Prompted)

**DSL:** `location Hand on any`

**Engine:** Before dispatching a setup rule edge, the quantifier guard
(`setup_contains_any`) checks whether any element collection uses
`Quantifier::Any`. If yes, `step()` issues a `ChoosePlayer` prompt
(`step_setup_any`); on resume the chosen player is substituted into *every*
any-site of the setup (`quantifier::substitute_setup_any`) before any
`GameData` mutation occurs (invariant I-20 — relaxed 2026-08-10 from
"rejected" to "prompted").

---

### 6.6 Synthetic State Lifecycle

- **Allocation** (I-16): Synthetic `StateID`s are allocated from `u32::MAX - 1`
  decrementing via `wrapping_sub`. Never collide with real IR ids (allocated
  from 0 upward).
- **Overlay** (I-17): `pending_overlay` maps synthetic ids to replacement edges.
  Only keyed by synthetic ids — real IR ids are never inserted. The overlay
  shadows `ir.states` only during quantifier dispatch.
- **Memory slot** (I-18): the synthetic card-set slot (stored as
  `Table_{SYNTH_MEMORY_KEY}`) is written just before a card-choice resume edge
  dispatches and removed at the top of `step()` when the FSM returns to a real
  IR state.
- **Resume guard** (I-19): `pending_quant.state` must equal `current_state` for
  resume to fire; mismatch returns `None` without popping either the pending
  quant or the buffered input.

---

## 7. Grammar-level gaps (no engine semantics exist)

These constructs parse but have **no defined semantics** — they are tracked in
`engine-vs-design.md` (D-6/D-7) and `dsl-completeness.md`, not re-annotated
here:

- `unless` — not in the grammar at all; use `if (not <expr>)`.
- `token` / `place` — tokens are not modeled in `GameData`.
- `flip` — status behaviour deferred to card encryption (§1b).
- `bid <qty>` without a memory target — errors since 2026-08-10 (see §3.7);
  `demand` — semantics never specified.
- `for <players>` stage clause — parsed, dropped during lowering (P-1).
- `SimStage` — lowers identically to sequential stages (P-2).
- `end game with winner <players>` — the IR jump to the goal ends the game; the
  action arm is an empty TODO.
