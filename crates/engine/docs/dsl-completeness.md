---
type: agent_wiki_node
module: crates::engine
scope: [all — DSL construct completeness]
topics: [dsl, grammar, ir-lowering, completeness, status]
last_validated: 2026-08-09
---

# CGDSL Construct Completeness

> **Purpose:** per-construct status of `.cgdsl`: does the **grammar** parse it
> (`crates/front_end/src/grammar.pest`), does the **IR builder** lower it
> (`crates/front_end/src/ir.rs`), and does the **engine** execute it
> (`crates/engine/src/`)? Statuses are verified against the current source and
> the five handoff demo games (`test_games/{blackjack,war,crazy_eights,
> five_card_draw,go_fish}.cgdsl`, driven by `tests/demo_games_test.rs`).
>
> Status key:
> - ✅ **works** — exercised and correct
> - ⚠️ **works with caveats** — see the note
> - ❌ **stub** — parsed, engine does nothing
> - 🚫 **rejected/error** — parsed but errors at runtime

---

## 1. Setup rules (run in declaration order, top-level)

| Construct | Grammar | IR | Engine | Status | Notes |
|---|---|---|---|---|---|
| `player P1, P2` | ✅ | ✅ | ✅ | ✅ | Appends players + initial turn order. `player` (no `create` keyword; `kw_create` exists in the grammar but is unused). |
| `team T1 with all` / `with (P:P1, P:P2)` | ✅ | ✅ | ✅ | ⚠️ | `All` resolves to in-game players (via `quantifier::resolve_player_candidates`); `Any` is rejected (setup-Any guard, I-20). |
| `turnorder all` / `(P:P1, P:P2)` / `... random` | ✅ | ✅ | ✅ | ✅ | `random` uses `rand::thread_rng()`. |
| `location Hand on all` / `on P:P1` / `on table` | ✅ | ✅ | ✅ | ✅ | `on all` → one location per player; `on T:T1` parses but **errors** at runtime (team-owned locations not in the data model); `on any` rejected. |
| `card on Deck: Rank(...) for Suit(...)` | ✅ | ✅ | ✅ | ✅ | Cartesian product of all key-value sets (`expand_types`). |
| `token <n> X on Loc` | ✅ | ✅ | ❌ | ❌ | No-op (no token data model). |
| `precedence P on Rank(Ace, Two, ...)` | ✅ | ✅ | ✅ | ✅ | Values ordered low → high. |
| `combo Pair where same Rank` | ✅ | ✅ | ✅ | ⚠️ | `where`-matching works for size/key filters; `same`/`distinct` inside *combo per-card matching* are broken (see `engine-vs-design.md` D-9). |
| `memory M on P:P1` / `on table` / `on all` | ✅ | ✅ | ✅ | ⚠️ | Key is owner-prefixed (`P1_M`, `Table_pot`). `MemoryType::Player`/`TeamCollection` initialize to `Int(0)` (I-10) — mismatched reads until written. |
| `points BJ on Rank(Ace: 11, ...)` | ✅ | ✅ | ✅ | ✅ | Map key `"Rank:Ace"` → int. |
| top-level actions (e.g. `shuffle Deck`, `deal 1 ...`) | ✅ | ✅ | ✅ | ✅ | Setup phase is just the flow before the first stage. |

## 2. Actions

| Construct | Grammar | IR | Engine | Status | Notes |
|---|---|---|---|---|---|
| `flip X to face up/down/private` | ✅ | ✅ | ❌ | ❌ | Silent no-op, **by design**: the per-card status slot (`GameData::card_statuses`) exists but is unused; `FlipAction` should become (de)encryption when card cryptography lands (`engine-vs-design.md` §1b). |
| `shuffle X` | ✅ | ✅ | ✅ | ✅ | Shuffles only the selected cards in place; unselected cards in the location stay put (fixed 2026-08-09). Eval failures are recoverable errors (were `eprintln!` + continue). |
| `set <players> out of game` / `of stage` / `of Play` | ✅ | ✅ | ✅ | ✅ | `GameSuccessful`/`GameFail` behave identically to `Game` (no success/fail tracking). |
| `M is <expr>` (SetMemory) | ✅ | ✅ | ✅ | ⚠️ | **No `of owner` clause in grammar** — key is prefixed with the *current player's* name (bridge, see `developer-notes.md` §1.1); errors (recoverably) with no current player. Collection types insert typed empty defaults. |
| `reset M` | ✅ | ✅ | ✅ | ⚠️ | Same owner-bridge; only resets `Int` memories (silent no-op otherwise); errors (recoverably) with no current player. |
| `cycle to next` / `cycle to P:P2` | ✅ | ✅ | ✅ | ✅ | No longer panics (fixed 2026-08-09): with no eligible *other* player (`resolve_turn` never considers the current player, I-13) it returns a recoverable `StepResult::Error` ("No next player available"). Games still guard with `if (size(playersin) >= 2)` to keep the turn flowing. |
| `bid ...` / `bid M ...` | ✅ | ✅ | ❌ | ❌ | No-op (semantics never specified). |
| `demand ...` / `demand ... as M` | ✅ | ✅ | ❌ | ❌ | No-op (semantics never specified). |
| `end turn` | ✅ | ✅ | ✅ | ✅ | `next_player()` — with nobody eligible this leaves `current_player = None` (no error). |
| `end stage` | ✅ | ✅ | ✅ | ✅ | `CurrentStage` — leaves the current stage; IR jumps to the stage's exit (unreachable-code check downstream). |
| `end Play` (named stage) | ✅ | ✅ | ✅ | ✅ | `Stage { name }` — jumps to that stage's exit. |
| `end game with winner <players>` | ✅ | ✅ | ⚠️ | ⚠️ | IR jumps straight to the goal state; the action arm is an empty TODO (harmless — the jump ends the game). |
| `deal <qty> from X <status> to Y` | ✅ | ✅ | ✅ | ✅ | Status is parsed but **ignored**. Quantity: literal ints ✅; runtime int exprs evaluated against the **live** state (fixed 2026-08-09) and errors propagate; `any` → `ChooseCards` prompt; range → prompt + re-prompt. |
| `exchange ...` / `move ...` (Classic) | ✅ | ✅ | ✅ | ✅ | Same code path as deal. |
| `place ... token ...` | ✅ | ✅ | ❌ | ❌ | No-op. |

## 3. Flow components

| Construct | Grammar | IR | Engine | Status | Notes |
|---|---|---|---|---|---|
| `stage S for X <end-condition> { ... }` | ✅ | ✅ | ✅ | ⚠️ | **`for X` is parsed but dropped** (B-1): the player clause has no IR effect. Stage semantics: per-iteration loop; end condition checked at stage *entry*; `StageRoundCounter` increments on loop-back. `for all` ≡ `for current` ≡ sequential. |
| `stage S for all until ...` (SimStage) | ✅ | ✅(as seq) | ✅ | ⚠️ | `build_sim_stage` is a copy of the sequential builder (B-3, explicit TODO). |
| End conditions: `N times` | ✅ | ✅ | ✅ | ✅ | Body runs exactly `N` times (counter ≥ N exits). |
| `until <bool>` | ✅ | ✅ | ✅ | ✅ | Exits when the bool is true at entry. |
| `until <bool> and/or N times` | ✅ | ✅ | ✅ | ✅ | Boolean AND/OR counter. |
| `until end` | ✅ | ✅ | ✅ | ✅ | Infinite loop; exit only via `end stage`/`end game` inside. |
| `if (bool) { ... }` | ✅ | ✅ | ✅ | ✅ | No `else` — use two complementary `if`s or `conditional`. |
| `conditional { case (bool): ... case else: ... }` | ✅ | ✅ | ✅ | ⚠️ | `case (A > B)` fails to parse when both operands are complex int exprs (PEG greediness, D-3) — use sequential `if`s. |
| `choose { ... or ... }` | ✅ | ✅ | ✅ | ✅ | One edge per `or`-separated option; each option is a *sequence* of flow components (e.g. `choose { deal X; if Y {} or deal Z }` = two options of two and one components — fixed 2026-08-09, previously every component became its own option). `InputType::Choice`; options labelled by the first payload of each branch. |
| `optional { ... }` | ✅ | ✅ | ✅ | ✅ | Accept → body; decline → nothing (no else-branch action is possible — standing/refusal cannot be recorded). |
| `trigger { ... }` | ✅ | ✅ | ✅ | ✅ | Body auto-executes once (payload itself is a no-op by design). |

## 4. End conditions & stage bookkeeping

| Construct | Grammar | IR | Engine | Status | Notes |
|---|---|---|---|---|---|
| `StageRoundCounter` payload | — | ✅ | ✅ | ✅ | Applied once per traversal (interpreter is the single mutator, I-5). |
| `EndStage` payload | — | ⚠️ | ✅ | ⚠️ | Never emitted by the IR builder; retained for completeness. |

## 5. Expressions

### BoolExpr
| Construct | Status | Notes |
|---|---|---|
| int compare `A > B` etc. | ✅ | Works in `if (A > B)`; **fails inside `case (...)`/`until (...)` when both sides are complex** (D-3). |
| cardset compare `X == Y` / `!=` | ✅ | Compares `(location, card ids)` tuples. |
| string compare `"A" == "B"` | ✅ | |
| player/team compare | ✅ | |
| `S in X` / `S not in X` (StringInCardSet) | ⚠️ | Matches *any* attribute value of any card in the set. |
| `X empty` / `X not empty` | ✅ | |
| `(A and B)` / `(A or B)` | ✅ | Parenthesised only; single-parenthesized bools (`(X)`) do **not** parse (D-2). |
| `not X` | ✅ | **`not (X)` does not parse** — write `not X` (D-2). |
| `<players> out of <X>` | ✅ | `out of game` / `of stage` / `of Play`; Successful/Fail ≡ Game. |

### IntExpr
| Construct | Status | Notes |
|---|---|---|
| literal, `(A + B)` binary | ✅ | `div` by zero → recoverable error. |
| `turnorder[N]`, collection-at | ✅ | |
| `size(cards X)` | ✅ | Syntax requires the `cards` prefix. `size(playersin)` / `playersout` / `others` ✅; `size((&P:M of all))` aggregates the slot across owners (implemented 2026-08-09). |
| `sum of X using PM` | ✅ | Sums point-map values over the cardset. |
| `min/max of X using PM` (ExtremaCardset) | ✅ | Returns card id; `min of top(...)` gives the card's value. |
| stage round counters | ✅ | `stageroundcounter`, `stageroundcounter(Stage)`; `"No current stage"` error otherwise. |
| `(&I:M of <owner>)` memory read | ✅ | Owner required — bare `&I:M` parses but **errors** ("memory access requires an explicit owner"). |
| collection-memory aggregation `(&I:M of all)` | ✅ | Implemented 2026-08-09: one value per owner holding the slot; missing slot or wrong type → recoverable error. |

### StringExpr / PlayerExpr / TeamExpr
| Construct | Status | Notes |
|---|---|---|
| `"literal"` | ✅ | Quoted. |
| `key of top(Loc)` (KeyOf) | ✅ | |
| `current` / `next` / `previous` / `competitor` | ⚠️ | `next` errors ("No next player available") when no eligible *other* player — **recoverable since 2026-08-09** (`cycle to next` no longer panics); `previous` ignores in-game/stage flags; `competitor` = first team-mate ≠ current. |
| `owner of <card position>` / `owner of min/max <memory>` | ✅ | `owner of memory` key order fixed 2026-08-09 (`P1_M`, was `M_P1`). |
| player memory `(&P:M of ...)` | ✅ | PlayerCollection memory → first index; String memory → name; Int → error. |
| `team of <player>` | ✅ | |
| player collection memory `(&PC:M of all)` / `(&PC:M of ...)` | ✅ | Implemented 2026-08-09: reads the slot; the `of all`/multi-owner form aggregates across owners (was a silent `vec![]`). |

### CardPosition / CardSet / filters
| Construct | Status | Notes |
|---|---|---|
| `top(Loc)` / `bottom(Loc)` / `Loc[N]` | ✅ | Index 0 = top. Bare location names resolve: current player's → Table's → any. |
| `min/max of X using PM` / `using Precedence` | ✅ | |
| `X of <owner>` (GroupOwner) | ✅ | Plain-location fast path (owner-resolved). `where`-groups are owner-resolved since 2026-08-09 (D-7); team/collection owners error. |
| `X where <filter>` | ✅ | Filters: `size(...)`, `same K`, `distinct K`, `adjacent K using P`, `K higher/lower than "V" using P`, `K is "V"`/`is not`, `combo C`/`not combo C`, binary `(A and B)`. An empty filter result reports the base location (fixed 2026-08-09 — was the location-0 sentinel). |
| `<combo> in X` / `not <combo> in X` | ✅ | Read-side syntax is the combo *name* (no `combo` keyword): `Pair in Hand`. Combos evaluate group-wise like `where` (fixed 2026-08-09 — per-card `same`/`distinct` matching was broken). |
| cardset memory `(&CS:M of ...)` | ⚠️ | Location inferred from the first card; falls back to location-0 sentinel (I-14/D-15) — a dest move may target the wrong pile. |

## 6. Scoring

| Construct | Status | Notes |
|---|---|---|
| `score <int> to <players>` | ✅ | Adds to `Player::score`. |
| `score <int> to M of <players>` (ScoreMemory) | ✅ | Writes `{player}_{M}` (does **not** touch `Player::score`). |
| `winner is <players>` | ✅ | Eliminates everyone not named. |
| `winner is min/max score` | ✅ | Among in-game players; ties → multiple winners; no candidates → nobody eliminated. |
| `winner is min/max position` | ⚠️ | Turn-order index (0-based); players absent from `turn_order` get `usize::MAX` — likely not the intended semantics (see `developer-notes.md` §1.3). |
| `winner is min/max <memory>` | ⚠️ | Int memories; missing/negative → 0; non-Int → 0. |

## 7. Input contract (host-facing)

| Prompt | Answer | Notes |
|---|---|---|
| `Choice { options, max_index }` | `Choice { idx }`, `idx ≤ max_index` | 0-based in the API; test files are 1-based. |
| `Optional(prompt)` | `OptionalAccept` \| `OptionalDecline` | Decline takes the "no" edge. |
| `ChoosePlayer { candidates }` | `ChoosePlayer { idx }`, `idx < len` | From `Hand of any` dest quantifiers. |
| `ChooseCards { display, min, max }` | `ChooseCards { selected }` | Indices **into `display`**; min/max enforced by the controller; range violations re-prompt. |

## 8. Known parse-level quirks (PEG)

1. `not (X)` and `(X)` (single bool in parens) do not parse — the binary wrapper requires `bool_op`. Write `not X` or `if (X)`.
2. `case (A > B)` / `until (A > B)` fail when both operands are non-literal int exprs (the inner `int_expr_bool` greedily consumes the whole parenthesis). Workaround: `if (A > B)` or split into two conditions.
3. Parenthesised cardsets in moves (`deal (X) ...`) fail — the quantity slot tries to parse the `(`. Write `deal X ...` (where-clauses bind fine without parens).
4. `size(cards X)` needs the `cards` keyword; `playersin`/`playersout`/`others` are single tokens (no spaces).
5. String literals are double-quoted (`Rank is "Ace"`); filter keyword is `is`, not `==`.
6. Where-clauses precede the owner: `Hand where Rank is "Ace" of next`.
7. All identifiers start with a capital letter (Pest `ident` rule).
