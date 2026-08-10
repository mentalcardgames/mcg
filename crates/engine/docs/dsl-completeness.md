---
type: agent_wiki_node
module: crates::engine
scope: [all — DSL construct completeness]
topics: [dsl, grammar, ir-lowering, completeness, status]
last_validated: 2026-08-11
---

# CGDSL Construct Completeness

> **Purpose:** per-construct status of `.cgdsl`: does the **grammar** parse it
> (`crates/front_end/src/grammar.pest`), does the **IR builder** lower it
> (`crates/front_end/src/ir.rs`), and does the **engine** execute it
> (`crates/engine/src/`)? Statuses are verified against the current source and
> the five handoff demo games (`test_games/{blackjack,war,crazy_eights,
> five_card_draw,go_fish}.cgdsl`, driven by `tests/demo_games_test.rs`).
>
> This is the **single status authority** for implementation state. What each
> construct *means* is documented in [`dsl-semantics.md`](./dsl-semantics.md);
> known bugs and divergences with repros live in
> [`engine-vs-design.md`](./engine-vs-design.md).
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
| `team T1 with all` / `with (P:P1, P:P2)` | ✅ | ✅ | ✅ | ✅ | `All` resolves to in-game players (via `Evaluator::resolve_player_collection`); `Any` prompts for one player at setup (setup-`Any`, I-20). |
| `turnorder all` / `(P:P1, P:P2)` / `... random` | ✅ | ✅ | ✅ | ✅ | `random` uses `rand::thread_rng()`. |
| `location Hand on all` / `on P:P1` / `on table` | ✅ | ✅ | ✅ | ✅ | `on all` → one location per player; `on T:T1` → one shared location per team; `on any` prompts for one player (I-20). |
| `card on Deck: Rank(...) for Suit(...)` | ✅ | ✅ | ✅ | ✅ | Cartesian product of all key-value sets (`expand_types`). |
| `token <n> X on Loc` | ✅ | ✅ | ❌ | ❌ | No-op (no token data model). |
| `precedence P on Rank(Ace, Two, ...)` | ✅ | ✅ | ✅ | ✅ | Values ordered low → high. |
| `combo Pair where same Rank` | ✅ | ✅ | ✅ | ✅ | Combos evaluate group-wise (like `where`): `Pair in Hand` returns the paired cards. |
| `memory M on P:P1` / `on table` / `on all` | ✅ | ✅ | ✅ | ✅ | Key is owner-prefixed (`P1_M`, `Table_pot`); team owners → one team-keyed slot (`Red_M`). |
| `memory X <expr> on ...` (typed) | ✅ | ✅ | ✅ | ✅ | Initial value honoured; Player→owner name, TeamCollection→own variant. |
| `points BJ on Rank(Ace: 11, ...)` | ✅ | ✅ | ✅ | ✅ | Map key `"Rank:Ace"` → int. |
| top-level actions (e.g. `shuffle Deck`, `deal 1 ...`) | ✅ | ✅ | ✅ | ✅ | Setup phase is just the flow before the first stage. |

## 2. Actions

| Construct | Grammar | IR | Engine | Status | Notes |
|---|---|---|---|---|---|
| `flip X to face up/down/private` | ✅ | ✅ | ❌ | ❌ | Silent no-op, **by design**: the per-card status slot (`GameData::card_statuses`) exists but is unused; `FlipAction` should become (de)encryption when card cryptography lands (see `dsl-semantics.md` §3.8). |
| `shuffle X` | ✅ | ✅ | ✅ | ✅ | Shuffles only the selected cards in place; unselected cards in the location stay put. Eval failures are recoverable errors. |
| `set <players> out of game` / `of stage` / `of Play` | ✅ | ✅ | ✅ | ✅ | `GameSuccessful`/`GameFail` behave identically to `Game` (no success/fail tracking). |
| `M is <expr>` (SetMemory) | ✅ | ✅ | ✅ | ✅ | Owner resolved declared-first, else current player; collections fully evaluated. |
| `reset M` | ✅ | ✅ | ✅ | ✅ | Same owner resolution; resets every variant to its typed zero. |
| `cycle to next` / `cycle to P:P2` | ✅ | ✅ | ✅ | ✅ | Self-wraps when the current player is the only eligible one; no-ops when nobody is eligible — no `size(playersin) >= 2` guard needed. |
| `bid <qty> on <memory> of <owner>` | ✅ | ✅ | ✅ | ✅ | **Numeric input prompt**: `any`/range → `InputType::Number` (bounds validated, re-asked); literal → writes `{owner}_{memory}`. Plain `bid` (no target) → recoverable error. |
| `demand ...` / `demand ... as M` | ✅ | ✅ | ❌ | ❌ | No-op (semantics never specified). |
| `end turn` | ✅ | ✅ | ✅ | ✅ | `next_player()` — advances to the next eligible player (wrapping onto the current player when it is the only eligible one). |
| `end stage` | ✅ | ✅ | ✅ | ✅ | `CurrentStage` — leaves the current stage; IR jumps to the stage's exit (unreachable-code check downstream). |
| `end Play` (named stage) | ✅ | ✅ | ✅ | ✅ | `Stage { name }` — jumps to that stage's exit. |
| `end game with winner <players>` | ✅ | ✅ | ✅ | ✅ | Declared winners eliminate everyone else; the IR jump to the goal ends the game. Winner set = in-game players (`GameData::winner_names`). |
| `deal <qty> from X <status> to Y` | ✅ | ✅ | ✅ | ✅ | **Verb semantics:** `deal` = automatic from the top. Literal quantities deal the top N; `any`/`>= M and <= N` prompt for the **count** (`InputType::Number`, bounds re-validated) then deal that many; a degenerate range (`>= 2 and <= 2`) deals automatically. Status is parsed but **ignored**. Quantity: literal ints ✅; runtime int exprs evaluated against the **live** state and errors propagate. |
| `exchange ...` / `move ...` (Classic) | ✅ | ✅ | ✅ | ✅ | **Verb semantics:** `move`/`exchange` = the player picks. Literal `N` on a non-positional source prompts pick-exactly-N (`min=max=N`, clamped; `SrcCardsExactN`); `any`/ranges prompt as before; positional sources (`top(X)`…) are automatic for any verb. |
| `place ... token ...` | ✅ | ✅ | ❌ | ❌ | No-op. |

## 3. Flow components

| Construct | Grammar | IR | Engine | Status | Notes |
|---|---|---|---|---|---|
| `stage S for X <end-condition> { ... }` | ✅ | ✅ | ✅ | ⚠️ | **`for X` is parsed but dropped** (P-1): the player clause has no IR effect. Stage semantics: per-iteration loop; end condition checked at stage *entry*; `StageRoundCounter` increments on loop-back. `for all` ≡ `for current` ≡ sequential. |
| `stage S for all until ...` (SimStage) | ✅ | ✅(as seq) | ✅ | ⚠️ | `build_sim_stage` is a copy of the sequential builder (P-2, explicit TODO). |
| End conditions: `N times` | ✅ | ✅ | ✅ | ✅ | Body runs exactly `N` times (counter ≥ N exits). |
| `until <bool>` | ✅ | ✅ | ✅ | ✅ | Exits when the bool is true at entry. |
| `until <bool> and/or N times` | ✅ | ✅ | ✅ | ✅ | Boolean AND/OR counter. |
| `until end` | ✅ | ✅ | ✅ | ✅ | Infinite loop; exit only via `end stage`/`end game` inside. |
| `if (bool) { ... }` | ✅ | ✅ | ✅ | ✅ | No `else` — use two complementary `if`s or `conditional`. |
| `conditional { case (bool): ... case else: ... }` | ✅ | ✅ | ✅ | ⚠️ | `case (A > B)` fails to parse when both operands are complex int exprs (PEG greediness, P-8) — use sequential `if`s. |
| `choose { ... or ... }` | ✅ | ✅ | ✅ | ✅ | One edge per `or`-separated option; each option is a *sequence* of flow components (e.g. `choose { deal X; if Y {} or deal Z }` = two options of two and one components). `InputType::Choice`; options labelled by the first payload of each branch. |
| `optional { ... }` | ✅ | ✅ | ✅ | ✅ | Accept → body; decline → nothing (no else-branch action is possible — standing/refusal cannot be recorded). |
| `trigger { ... }` | ✅ | ✅ | ✅ | ✅ | Body executes each time the flow reaches it (the payload itself is a no-op marker). |

## 4. End conditions & stage bookkeeping

| Construct | Grammar | IR | Engine | Status | Notes |
|---|---|---|---|---|---|
| `StageRoundCounter` payload | — | ✅ | ✅ | ✅ | Applied once per traversal (interpreter is the single mutator, I-5). |
| `EndStage` payload | — | ⚠️ | ✅ | ⚠️ | Never emitted by the IR builder; retained for completeness. |

## 5. Expressions

### BoolExpr
| Construct | Status | Notes |
|---|---|---|
| int compare `A > B` etc. | ✅ | Works in `if (A > B)`; **fails inside `case (...)`/`until (...)` when both sides are complex** (P-8). |
| cardset compare `X == Y` / `!=` | ✅ | Compares `(location, card ids)` tuples. |
| string compare `"A" == "B"` | ✅ | |
| player/team compare | ✅ | |
| `S in X` / `S not in X` (StringInCardSet) | ⚠️ | Matches *any* attribute value of any card in the set. |
| `X empty` / `X not empty` | ✅ | |
| `(A and B)` / `(A or B)` | ✅ | Parenthesised only; single-parenthesized bools (`(X)`) do **not** parse (P-8). |
| `not X` | ✅ | **`not (X)` does not parse** — write `not X` (P-8). |
| `<players> out of <X>` | ✅ | `out of game` / `of stage` / `of Play`; Successful/Fail ≡ Game. |

### IntExpr
| Construct | Status | Notes |
|---|---|---|
| literal, `(A + B)` binary | ✅ | `div` by zero → recoverable error. |
| `turnorder[N]`, collection-at | ✅ | |
| `size(cards X)` | ✅ | Syntax requires the `cards` prefix. `size(playersin)` / `playersout` / `others` ✅; `size((&P:M of all))` aggregates the slot across owners. |
| `sum of X using PM` | ✅ | Sums point-map values over the cardset. |
| `min/max of X using PM` (ExtremaCardset) | ✅ | Returns card id; `min of top(...)` gives the card's value. |
| stage round counters | ✅ | `stageroundcounter`, `stageroundcounter(Stage)`; `"No current stage"` error otherwise. |
| `(&I:M of <owner>)` memory read | ✅ | Owner required — **bare `&I:M` resolves declared-owner → current player**, erroring only when neither exists. |
| collection-memory aggregation `(&I:M of all)` | ✅ | One value per owner holding the slot; missing slot or wrong type → recoverable error. |

### StringExpr / PlayerExpr / TeamExpr
| Construct | Status | Notes |
|---|---|---|
| `"literal"` | ✅ | Quoted. |
| `key of top(Loc)` (KeyOf) | ✅ | |
| `current` / `next` / `previous` / `competitor` | ✅ | `next`/`previous` skip ineligible players and self-wrap when alone; `competitor` = first team-mate ≠ current. |
| `owner of <card position>` / `owner of min/max <memory>` | ✅ | `owner of memory` reads `{player}_{memory}`. |
| player memory `(&P:M of ...)` | ✅ | PlayerCollection memory → first index; String memory → name; Int → error. |
| `team of <player>` | ✅ | |
| player collection memory `(&PC:M of all)` / `(&PC:M of ...)` | ✅ | Reads the slot; the `of all`/multi-owner form aggregates across owners. |

### CardPosition / CardSet / filters
| Construct | Status | Notes |
|---|---|---|
| `top(Loc)` / `bottom(Loc)` / `Loc[N]` | ✅ | Index 0 = top. Bare location names resolve: current player's → their team's → Table's → any. |
| `min/max of X using PM` / `using Precedence` | ✅ | |
| `X of <owner>` (GroupOwner) | ✅ | Plain-location fast path (owner-resolved). `where`-groups are owner-resolved; team owners resolve to the team's shared pile; collection owners error. |
| `X where <filter>` | ✅ | Filters: `size(...)`, `same K`, `distinct K`, `adjacent K using P`, `K higher/lower than "V" using P`, `K is "V"`/`is not`, `combo C`/`not combo C`, binary `(A and B)`. An empty filter result reports the base location. |
| `<combo> in X` / `not <combo> in X` | ✅ | Read-side syntax is the combo *name* (no `combo` keyword): `Pair in Hand`. Combos evaluate group-wise like `where`. **Lay-down moves** (`move <combo> in X …`) prompt the player to choose cards from the pile and **validate** the choice against the combo filter, re-prompting on mismatch; **0 cards = skip**; combine with `until <combo> in X empty` for a lay-down-all stage loop. Read-side evaluation still over-approximates pairs (D-16). |
| cardset memory `(&CS:M of ...)` | ⚠️ | Location inferred from the first card; falls back to location-0 sentinel (I-14) — a dest move may target the wrong pile. |

## 6. Scoring

| Construct | Status | Notes |
|---|---|---|
| `score <int> to <players>` | ✅ | Adds to `Player::score`. |
| `score <int> to M of <players>` (ScoreMemory) | ✅ | Writes `{player}_{M}` (does **not** touch `Player::score`). |
| `winner is <players>` | ✅ | Eliminates everyone not named. |
| `winner is min/max score` | ✅ | Among in-game players; ties → multiple winners; no candidates → nobody eliminated. |
| `winner is min/max position` | ✅ | Turn-order index (0-based); players absent from `turn_order` are **excluded**. |
| `winner is min/max <memory>` | ✅ | Int memories; players without the slot are skipped; a non-Int slot is a recoverable error. |

## 7. Input contract (host-facing)

| Prompt | Answer | Notes |
|---|---|---|
| `Choice { options, max_index }` | `Choice { idx }`, `idx ≤ max_index` | 0-based in the API; test files are 1-based. |
| `Optional(prompt)` | `OptionalAccept` \| `OptionalDecline` | Decline takes the "no" edge. |
| `ChoosePlayer { candidates }` | `ChoosePlayer { idx }`, `idx < len` | From `Hand of any` dest quantifiers. |
| `ChooseCards { display, min, max }` | `ChooseCards { selected }` | Indices **into `display`**; min/max enforced by the controller; range violations re-prompt. |
| `Number { min, max, prompt }` | `Number { value }` | From `bid any on <memory> of <owner>`; bounds enforced by controller + interpreter resume; TestFile line `n <N>`. |

Ineligible players are never prompted (I-24): a player out of the game or out
of the current stage has their instruction edges skipped instead (only
cycle/end actions and stage bookkeeping run), and stages auto-end when no
players remain in the game or in the stage.

## 8. Known parse-level quirks (PEG)

1. `not (X)` and `(X)` (single bool in parens) do not parse - the binary wrapper requires `bool_op`. Write `not X` or `if (X)`.
2. `not <combo> in X empty` — a leading `not` before a combo group **binds to the boolean** (the positive spelling `combo in X not empty` also works).
3. `case (A > B)` / `until (A > B)` fail when both operands are non-literal int exprs (the inner `int_expr_bool` greedily consumes the whole parenthesis). Workaround: `if (A > B)` or split into two conditions.
4. Parenthesised cardsets in moves (`deal (X) ...`) fail — the quantity slot tries to parse the `(`. Write `deal X ...` (where-clauses bind fine without parens).
5. `size(cards X)` needs the `cards` keyword; `playersin`/`playersout`/`others` are single tokens (no spaces).
6. String literals are double-quoted (`Rank is "Ace"`); filter keyword is `is`, not `==`.
7. Where-clauses precede the owner: `Hand where Rank is "Ace" of next`.
8. All identifiers start with a capital letter (Pest `ident` rule).
