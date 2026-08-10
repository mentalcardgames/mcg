---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [cgdsl, authoring, guide, tutorial, blackjack]
last_validated: 2026-08-11
---

# CGDSL Authoring Guide

> **Purpose:** a practical guide for writing `.cgdsl` game definitions. Every
> construct is described with its exact syntax (from `grammar.pest`) and its
> runtime behaviour (from the engine). This document is the **authoring
> reference**; for the full semantics specification see
> `docs/dsl-semantics.md`.

**Status key:**
- ✅ **Implemented** — works as documented
- ⚠️ **Implemented with limitations** — read the caveat
- ❌ **Stub** — parsed, engine does nothing
- ❌ **Not in grammar** — does not parse at all

---

## 1. Lexical Rules

### 1.1 Identifiers

All user-defined names **must start with a capital letter** (`A`–`Z`),
followed by zero or more letters or digits:

```
MyGame   P1   Hand   Stock   Rank   Ace   BJ
```

Lowercase words are **reserved keywords** (`player`, `stage`, `move`, `deal`,
`score`, …). The parser distinguishes identifiers from keywords purely by
capitalisation — there is no quoting or escaping.

- Player references: `P:Name` (e.g. `P:Alice`, `P:Player1`)
- Team references: `T:Name` (e.g. `T:Red`)

### 1.2 Comments

```
// line comment
/* block comment */
```

### 1.3 String literals

String literals are written `"CapitalWord"`. Because the parser rule is
`"\"" ~ value ~ "\""` and `value = { ident }` (capital-starting), **only
capitalised identifiers can be string literals:**

```
Valid:   "Ace"   "Hearts"   "Spades"
Invalid: "ace"   "hello"   (lowercase — parse error)
```

This matters for boolean comparisons like `Rank of top(Hand) == "Ace"`.

### 1.4 No `create` prefix

Setup rules **do not** use a `create` keyword:

```
WRONG:  create players P1, P2
RIGHT:  player P1, P2
```

(The `create` keyword exists in the grammar but is not used by any rule.)

### 1.5 The `table` keyword

`table` is a special built-in owner representing the global play area. It
requires no `P:` or `T:` prefix. Use it for shared locations like `Deck`,
`Stock`, `Discard`.

---

## 2. File Structure

A `.cgdsl` file is a flat list of **flow components**:

```
<setup-rule>*
<stage-definition>*
<scoring-rule>*
```

Setup rules are typically written once at the top. Stages contain the game
logic. Scoring rules can appear at top level (final scoring) or inside
stages.

---

## 3. Setup Rules

Setup rules execute in declaration order before any stage runs. They
populate players, locations, cards, teams, turn order, and metadata.

### 3.1 Players

```
player P1, P2, P3
```

Creates named players. Initial turn order is declaration order (overridable
via `turnorder` below).

### 3.2 Teams

```
team Red with (P:P1, P:P3)
team Blue with (P:P2, P:P4)
```

Creates a team with a given set of players. The player collection can use
`all` or any player-collection expression (see §4.5).

### 3.3 Turn order

```
turnorder (P:P3, P:P1, P:P2)       // explicit order
turnorder all                       // all players in declaration order
turnorder all random                // all players, shuffled
```

Overrides the default turn order. `random` shuffles.

### 3.4 Locations

```
location Hand on all                // one Hand per player
location Hand on P:P1               // Hand only for P1
location Deck on table              // one global Deck
location Discard, Stock on table    // comma-separated: multiple at once
location TeamPile on T:Red          // one shared TeamPile for the team
```

The `on` clause accepts any owner expression (`P:Name`, `table`, `all`,
`any`, player collections, team owners, …).

- **Team owners** (`on T:Red`) create **one shared location for the whole
  team**, owned by the team entity. The bare name resolves to it while a team
  member is current (`move top(Hand) face up to TeamPile`); the explicit
  `TeamPile of T:Red` addresses it unambiguously.
- `on any` prompts for one player during setup; the chosen player is
  substituted before the rule runs. Use `on all` for a per-player location
  for everyone.

### 3.5 Cards

```
card on Deck:
  Rank(Ace, Two, Three)
    for Suit(Hearts, Spades)
```

Creates the **cartesian product** of the key-value sets: here 3 ranks × 2
suits = 6 cards. `for` chains create additional dimensions (multi-key
combination). Cards are appended to the named location. **Without a
`shuffle`, creation order IS the pile order** — deterministic tests and
fixtures rely on this.

You can also create multiple card types on the same location separated by
commas:

```
card on Deck:
  Rank(Ace) for Suit(Hearts),
  Rank(Two) for Suit(Spades)
```

### 3.6 Precedences

Define a total ordering for a card attribute (used by filter expressions and
`min`/`max`):

```
precedence RankOrd on Rank(A, 2, 3, 4, 5, 6, 7, 8, 9, T, J, Q, K)
```

Shorthand form:

```
precedence RankOrd (Rank:A, Rank:2, Rank:3)
```

Values are ordered low → high.

### 3.7 Point maps

Assign integer values to card attributes (used by `sum … using`):

```
points BJ on Rank(
  Ace: 11, Two: 2, Three: 3, Four: 4, Five: 5,
  Six: 6, Seven: 7, Eight: 8, Nine: 9, Ten: 10,
  Jack: 10, Queen: 10, King: 10
)
```

Shorthand form:

```
points Values (Rank:Ace:1, Rank:Two:2, Rank:Three:3)
```

The value expressions **are evaluated at setup time**, so runtime
expressions like `&I:M of current` work here.

### 3.8 Combos

Define named card-filter combinations (evaluated group-wise when used, like
a `where` clause):

```
combo Pair where size == 2 and Rank same
```

See §4.6 for filter syntax.

### 3.9 Memories

```
memory M on table                          // no type → Int(0)
memory InitialScore 42 on P:P1             // Int, initialised to 42
memory NameOfFirst "Ace" on P:P1           // String, initialised to "Ace"
memory Winner P:P1 on table                // Player, initialised to the player's name
memory Scores (1, 2, 3) on table           // IntCollection, initialised to the list
```

Memories are stored in a global store keyed as `<Owner>_<MemoryName>` —
`P1_M`, `Table_pot`, … (see §6.6 for how reads and writes pick the owner).

The declared type-expression is **evaluated at setup time** and becomes the
initial value: `memory Pot 100 on table` really starts `Table_pot` at 100.
Typed memories initialise typed slots — a `Player`-typed memory holds the
evaluated player's *name* (or a player-owned slot's own owner), `Team`
holds the team name, collections hold their evaluated contents.

⚠️ Setup `with I: 0` syntax does **not** exist in the grammar. Use a bare
expression: `memory Name 42 on P:P1`.

⚠️ **Team owners** (`memory M on T:Red`) create **one slot per team**, keyed
by the team name (`Red_M`) — matching the read/write addressing
(`(&I:M of T:Red)`, `bid 5 on M of T:Red`).

### 3.10 Tokens

```
token 3 Marker on table
```

❌ **Stub** — tokens are not modeled in `GameData`. Any `token` rule is
effectively ignored.

---

## 4. Expressions

### 4.1 Integer expressions

| Form | Example | Notes |
|------|---------|-------|
| Literal | `42`, `-5` | Plain integer |
| Binary op | `(1 + 2)`, `(X * Y)` | `+`, `-`, `*`, `/`, `mod` |
| Collection index | `(1,2,3)[0]` | 0-based index |
| `size(collection)` | `size((1,2,3))` | Number of elements |
| `sum(collection)` | `sum((1,2,3))` | Sum int elements |
| `sum of X using PM` | `sum of Hand of current using BJ` | Sum card values via point map |
| `min/max of X using PM` | `max of Hand using BJ` | Extrema of card values |
| `min/max(collection)` | `max((1,2,3))` | Extrema of int collection |
| Runtime counter | `stageroundcounter` | Current counter; also `stageroundcounter(StageName)` |
| Memory ref | `(&I:M of P:P1)` | Explicit owner, or bare `&I:M` (see §6.6) |

The runtime counters increment once per stage-loop iteration:
- `stageroundcounter` — counter of the currently-executing stage
- `stageroundcounter(Play)` — counter of a named stage

These are useful for limiting iterations within a stage body (e.g. "stop
dealing after 5 draws"). The counter is `0` on the stage's first pass.

### 4.2 String expressions

| Form | Example | Notes |
|------|---------|-------|
| Literal | `"Ace"` | Capitalised ident only |
| Key of card | `Rank of top(Hand)` | Attribute value of a card |
| Collection index | `("A","B")[0]` | 0-based index |
| Memory ref | `(&S:M of P:P1)` | Explicit owner, or bare `&S:M` (see §6.6) |

### 4.3 Boolean expressions

| Form | Example | Notes |
|------|---------|-------|
| Comparison | `X == Y`, `X < Y` | Ints: `== != < > <= >=`; equality on strings/players/teams/cardsets |
| Set empty | `Hand empty` | True if no cards |
| Set not empty | `Hand not empty` | True if ≥1 card |
| String in cardset | `"Hearts" in Hand` | Any card has attr=string |
| String not in cardset | `"Hearts" not in Hand` | Negated |
| Logical | `not X` | Unary negation |
| Logical | `(X and Y)`, `(X or Y)` | Binary, parenthesised |
| Out-of check | `P:P1 out of game` | Player eliminated? |
| Out-of check | `current out of Play` | Player out of stage? |

⚠️ Grammar quirks: `not (X)` does **not** parse — write `not X`; a single
bool in parens `(X)` does not parse either; `case (A > B)`/`until (A > B)`
fail when both operands are complex int expressions (use `if (A > B)` or
split the condition).

### 4.4 Player expressions

| Form | Example | Notes |
|------|---------|-------|
| Literal | `P:Alice` | Named player |
| Runtime | `current` | Current turn player |
| Runtime | `next` | Next eligible player (in-game and in-stage); wraps onto `current` when it is the only eligible one |
| Runtime | `previous` | Previous eligible player, same rules as `next` |
| Runtime | `competitor` | Another player in same team |
| Index | `turnorder[2]` | Nth in turn order |
| Index | `(P:A,P:B)[0]` | Nth in collection |
| Card owner | `owner of top(Hand)` | Who owns a card? |
| Memory owner | `owner of highest M` | Player with max memory |

### 4.5 Player collections

| Form | Example | Notes |
|------|---------|-------|
| Explicit | `(P:A, P:B, P:C)` | Comma-separated |
| All | `all` | All in-game players |
| Any | `any` | Prompt player to pick one |
| In-game | `playersin` | Players with `in_game=true` |
| Out-of-game | `playersout` | Players with `in_game=false` (eliminated) |
| Others | `others` | All in-game players except `current` |
| Memory | `(&PC:names of P:P1)` | Read from memory |

### 4.6 Card sets

Card sets are the "from" and "to" of move/deal actions, and the target of
boolean checks.

| Form | Example | Notes |
|------|---------|-------|
| Location | `Hand` | Resolves: current player → their team → Table → first match |
| Location of owner | `Hand of P:Alice` | Explicit owner |
| Location of owner | `Hand of current` | Current player's location |
| Card position | `top(Hand)` | First card |
| Card position | `bottom(Hand)` | Last card |
| Card position | `Hand[2]` | 0-based index |
| With filter | `Hand where Rank is "Ace"` | Filtered subset |
| Combo match | `Pair in Hand` | Cards matching combo |
| Memory | `(&CS:myCards of P:P1)` | Read from memory |

#### Card position by value

```
min of Hand using BJ          // card with lowest point-map value
max of Hand using RankOrd     // card with highest precedence
```

#### Filter expressions

Used in `where` clauses and `combo` definitions:

| Filter | Example | Notes |
|--------|---------|-------|
| Size | `size == 2` | Exact card count; also `!=`, `<`, `>`, `<=`, `>=` |
| Attribute value | `Rank is "Ace"` | Card has this attr |
| Attribute not value | `Rank is not "Two"` | Negated |
| Same attribute | `Rank same` | All cards share attr value |
| Distinct attribute | `Suit distinct` | All cards differ on attr |
| Adjacent | `Rank adjacent using RankOrd` | Values are consecutive |
| Higher than | `Rank higher than "Three" using RankOrd` | Compares via precedence |
| Lower than | `Rank lower than "Jack" using RankOrd` | Compares via precedence |
| Combo | `Pair` | Matches combo definition |
| Not combo | `not Pair` | Does not match |
| Binary | `(Rank same and Suit distinct)` | Combined with `and`/`or` |

---

## 5. Stages & End Conditions

### 5.1 Stage syntax

```
stage <Name> for <player> <end-condition> {
    <flow-component>+
}
```

The `for` clause is **mandatory**:

```
stage Play for current 3 times { ... }     // seq (player_expr)
stage Reveal for all 1 times { ... }       // sim (player_collection)
```

⚠️ The `for` clause currently has **no effect** — every stage includes all
players (`for current` ≡ `for all`), and both forms produce the same IR.
The stage body runs as a loop for the *current* player; per-player
participation (`set … out of <stage>`) and the turn loop (`cycle to next`)
are the ways to scope who acts.

### 5.2 End conditions

| Condition | Syntax | Behaviour |
|-----------|--------|-----------|
| Fixed iterations | `N times` | Exits after N stage-round increments |
| Until bool | `until <bool>` | Exits when bool becomes true |
| Until end | `until end` | Exits via `end stage` / `end <name>` action |
| Until bool + count | `until <bool> and N times` | Exits when bool is true AND counter ≥ N |
| Until bool + count | `until <bool> or N times` | Exits when bool is true OR counter ≥ N |

Examples:

```
stage Draw for current 5 times { ... }
stage Draw for current until Hand empty { ... }
stage Draw for current until end { ... }
```

**Auto-end (no players left):** at every end-condition evaluation the stage
also exits when **no players remain in the game** (the game then runs out to
the goal with an empty winner set) or **no players remain in this stage**
(everyone was set out of it). `until end` stages therefore cannot loop
forever after everyone is eliminated.

### 5.3 Stage lifecycle

1. **Entry**: `ensure_stage_entered(name)` runs on first encounter. Marks
   all players `in_stage[name] = true`. Idempotent if the stage is already
   on the stack.
2. **Loop body**: Each iteration evaluates the end condition, then (if not
   exiting) runs the body rules, increments the stage round counter, and
   loops back.
3. **Exit**: On end-condition match or `end stage` action, the stage is
   popped from `stage_stack`.

⚠️ The end condition is checked at stage *entry* — effects produced mid-body
(e.g. "a player emptied their hand") are only observed on the next entry, up
to one full rotation late. Bound such stages with `N times` to guarantee
termination.

### 5.4 Ineligible players are skipped (no prompts to eliminated players)

A player who is **out of the game**, or **out of the current stage**, is
never offered input and none of their instructions run:

- Their instruction edges (moves, scores, conditions, choices, optionals,
  triggers, quantifier prompts) are **skipped** — advanced through without
  executing.
- Only **cycle actions** (`cycle to …`), **end actions** (`end turn` /
  `end stage` / `end <name>`), and the stage bookkeeping still execute — so
  `cycle to next` keeps moving the turn to the next eligible player, and the
  stage loops normally with the new player.

The skip applies while the condition holds and stops as soon as a cycle
lands on an eligible player. Outside a stage (setup, top-level rules)
nothing is skipped.

### 5.5 Trigger rules

Trigger rules fire each time they are encountered in the flow:

```
trigger {
    shuffle Deck
}
```

A `trigger` block inside a stage fires on **every iteration** (when the flow
reaches it). A top-level `trigger` fires once before any stage. There is no
dedicated `on enter` syntax: entry-only behaviour is expressed with a
1-iteration stage (`stage X for current 1 times`) or an
`if (stageroundcounter == 0)` guard (the counter is 0 on the first pass).

Triggers are lowered as marker edges and dispatched immediately.

---

## 6. Actions

Actions mutate `GameData`. The three move verbs share one syntax and engine
path, but **the verb carries the choice semantics**:

- `deal` — **automatic**: the cards come off the **top** of the collection.
  `deal 3 from Deck` = "draw 3"; the player never chooses which cards.
- `move` / `exchange` — **the player picks the cards** from the collection
  (prompted), unless a position is given.

### 6.1 Moving cards

```
move <quantity> from <cardset> <status> to <cardset>
deal <quantity> from <cardset> <status> to <cardset>
exchange <quantity> from <cardset> <status> to <cardset>
```

The **status** field is **mandatory** — even though the engine ignores it
(for now):

```
move top(Hand) face up to Discard
deal 2 from Deck private to Hand of current
exchange any from Stock face down to Hand
```

| Status | Syntax |
|--------|--------|
| Face up | `face up` |
| Face down | `face down` |
| Private | `private` |

Cards are removed from **all** locations (not just the source) before being
added to the destination. This is a brute-force approach that works because
each card is globally unique.

**The `<quantity>` field (verb-aware):**

| Quantity | `deal` (automatic, from the top) | `move` / `exchange` (player picks) |
|----------|----------------------------------|-----------------------------------|
| Literal `N` | Deal the top N cards | **Prompt: pick exactly N cards** (e.g. `move 1 from Hand` = "pick one") |
| `all` | All cards | All cards |
| `any` | **Prompt: how many?** (1..pile size), then deal that many | Prompt: pick 1..N cards |
| `>= M and <= N` | **Prompt: how many?** (M..N), then deal that many | Prompt: pick M..N cards |
| Degenerate range (`>= 2 and <= 2`) | Automatic: deal 2 (no prompt) | Prompt: pick exactly 2 |
| Omitted *(no quantity)* | All cards | All cards |

**A positional source makes the move automatic for any verb** — the position
already chose the card(s): `move 1 from top(Stock)`, `move top(Hand)`,
`move 1 from Hand[2]`, `deal 1 from top(Deck)`. A `where`-filtered source is
*not* positional — `move 1 from Hand where Rank is "Ace"` prompts the player
to pick one Ace.

The exact-N prompt clamps to the available cards (`move 5` over a 2-card pile
asks for exactly 2), and a 0/empty quantity or empty source is a no-op.

**Examples:**

```
deal 3 from Deck private to Hand of current   // draw 3, automatic
deal any from Deck private to Hand of current // "how many?" then draw
move 1 from Hand private to Discard           // pick one card to discard
move >= 2 and <= 5 from Hand private to Discard  // pick 2..5 to discard
move top(Hand) face up to Table               // automatic (positional)
```

When `<quantity>` is omitted, all cards from the source set are moved.

### 6.2 Shuffling

```
shuffle Deck
shuffle Hand of current
```

Randomises the selected cards in place within their location (unselected
cards stay put — `shuffle top 3 of Deck` only shuffles the top three).
Evaluation failures are recoverable errors.

### 6.3 Cycle (change current player)

```
cycle to next
cycle to P:Bob
cycle to current
```

Sets `current_player` in `turn_order`. `next` walks turn order, skipping
eliminated or out-of-stage players. With no eligible *other* player the turn
**wraps onto the current player** when it is still eligible; with nobody
eligible at all, `cycle to next` is a **no-op**. Elimination games need no
guards — and the stage auto-ends when no players remain (§5.2).

`previous` mirrors `next` (reverse scan, same eligibility rules), so
`cycle to previous` works for turn-order reversal.

### 6.4 End scope

```
end turn              // advance to next player
end stage             // leave current stage
end Play              // leave named stage
end game with winner P:P1   // declare the winner(s) and end the game
```

`end turn` advances to the next eligible player, wrapping onto the current
player when it is the only eligible one (it never strands the game).

`end game with winner <players>` eliminates everyone not named and ends the
game. **Winner set:** every player still in the game at the end is a winner
— whether declared via `winner is …` / `end game with winner …` (which
eliminate the rest) or simply because the game ran out of stages with
players left in. With nobody left in the game, the winner set is empty.

### 6.5 Set player out

```
set P:P1 out of game         // eliminate from game entirely
set P:Bob out of Play         // mark out of a specific stage
set current out of game       // self-eliminate
set current out of stage      // exit current stage for this player
```

`out of game` sets `in_game = false`. `out of stage` / `out of <name>` sets
the player's `in_stage[<name>] = false`. Once out, the player is skipped by
the turn flow (§5.4) and never prompted.

### 6.6 Memory operations

```
// Set memory (action — the target owner is resolved automatically)
M is 42                      // Int
M is "Hello"                  // String
M is P:Alice                  // Player (stored as String)
M is T:Red                    // Team
M is (1, 2, 3)                // IntCollection
M is ("A", "B")               // StringCollection
M is (P:Alice, P:Bob)        // PlayerCollection
M is (top(Hand), bottom(Hand)) // CardSet

// Read back (expression)
(&I:M of P:P1)   // reads Int memory, explicit owner
score (&I:M of current) to current

// Reset (every variant to its typed zero)
reset M
```

**Owner resolution (reads and writes):** the write rules have no
`of <owner>` clause, so `M is 5` / `reset M` (and bare reads like `&I:M`
without `of <owner>`) target:

1. the **declared owner** — when exactly one existing slot ends in `_M`
   (`memory pot on table` declares `Table_pot`, so `pot is 5` writes
   `Table_pot`);
2. otherwise the **current player's** slot;
3. otherwise a recoverable error.

Reads with an explicit owner (`&I:M of P:P1`, `&I:M of table`) always use
that owner.

Collection memory variants are fully evaluated: literals (`M is (1, 2, 3)`)
and `Memory`-form copies of existing collection slots.

### 6.7 Numeric input — `bid <quantity> on <memory> of <owner>`

```
bid any on Pot of table                  // prompt for any number
bid >= 1 and <= 10 on Bet of table       // prompt, bounded 1..=10
bid 5 on Pot of table                    // literal: write 5, no prompt
```

Asks the current player for a number and stores it in the owner's memory
slot. Out-of-range answers are rejected and re-asked. This is the DSL
surface for betting/ante mechanics.

⚠️ A plain `bid <quantity>` **without** a memory target is a recoverable
error — always use the `on <memory> of <owner>` form.

### 6.8 Stubs

| Action | Status |
|--------|--------|
| `flip <cardset> to <status>` | ⏳ No-op by design — becomes (de)encryption with card crypto; the status slot exists |
| `place <token> from ... to ...` | ❌ Tokens not modeled |
| `demand <type>` / `demand ... as <memory>` | ❌ Semantics undefined |

---

## 7. Control Flow & Player Prompts

### 7.1 `if`

```
if (<bool-expr>) {
    <flow-component>+
}
```

No `else` clause. Use `if` or `conditional` for branching.

### 7.2 `optional`

```
optional {
    deal 1 from Deck private to Hand
}
```

Presents an **accept/decline** prompt to the current player. Accept → runs
the body. Decline → skips.

⚠️ Decline runs **nothing** — "standing" or "refusing" is not recorded, so
an optional re-asks the same player on the next round. Bound the stage with
`N times` and let players re-decline, or model the choice with a memory.

### 7.3 `choose`

```
choose {
    move top(Hand) face down to Discard
    or
    deal 1 from Deck private to Hand
}
```

Presents a **multi-choice** prompt. Each `or`-separated arm is a **sequence**
of flow components executed in order when selected. The player selects one
arm by index. There can be 1+ arms (a single arm is equivalent to an
`optional`).

```
// Multi-statement arms: arm 1 = deal + conditional draw, arm 2 = pass.
choose {
    deal 1 from Deck private to Hand of P:P1
    if (size(cards Deck) == 0) {
        deal 1 from Deck private to Hand of P:P2
    }
    or
    score 1 to P:P1
}
```

### 7.4 `conditional`

```
conditional {
    case (X == 0):
        score 1 to current
    case (X == 1):
        score 2 to current
    case else:
        score 3 to current
}
```

Evaluates each `case` condition in order. The first matching case executes.
`case else:` (no condition) acts as a catch-all; cases after it are
unreachable (a diagnostic is emitted).

### 7.5 Trigger

```
trigger {
    shuffle Deck
}
```

Fires each time it is encountered in the flow — every stage iteration for a
trigger inside a stage body, once for a top-level trigger (see §5.5).

---

## 8. Scoring & Winners

### 8.1 Score rules

```
score <int-expr> to <players>                // add to player.score
score <int-expr> to <memory> of <players>    // write to memory
```

Examples:

```
score 10 to P:P1
score (5 + 3) to (P:P1, P:P2)
score sum of Hand of current using BJ to current
score 42 to ScoreSlot of P:P1
```

`score N to M of <players>` writes the per-player slot `{player}_M` and does
**not** touch `Player::score`.

### 8.2 Winner rules

```
winner is P:P1                    // explicit: P1 wins, others eliminated
winner is (P:P1, P:P2)           // multiple winners (tie)
winner is highest score           // highest player.score wins
winner is lowest score            // lowest player.score wins
winner is highest position        // earliest in turn order wins
winner is lowest position         // latest in turn order wins
winner is highest M               // highest memory value wins
winner is lowest M                // lowest memory value wins
```

Explicit winners: all other players are set `in_game = false`.

Extrema winners: all in-game players are compared; only those matching the
target value remain (ties are kept). `position` uses the 0-based turn-order
index (players absent from the turn order are excluded). Memory extrema
(`winner is highest M`) read the per-player slot `{player}_M`; players
without the slot are skipped, and a non-Int slot is an error.

### 8.3 The winner set at game end

Every game ends with a **winner set**: the players still `in_game` when the
FSM reaches the goal (in declaration order; empty when nobody won). Explicit
winner declarations reduce to the same rule, because they eliminate everyone
else. The winner set is displayed by all tooling at game end — the TUI trace
log (`GameOver: winners: P1, P3`), the trace file footer
(`=== GameOver after N steps — winners: P1, P3 ===`), and `cgdsl-play`'s
summary — and is available programmatically via `GameData::winner_names()`.

---

## 9. Quantifiers

Quantifiers expand a single edge into multiple runtime paths.

### 9.1 `all` in destination

```
deal 1 from Deck private to Hand of all
```

Builds a **fan-out chain** of synthetic edges — one per player. The FSM
automatically steps through each. No player prompt. Dealing is sequential:
each player's edge sees the deck after the previous player's edge moved
cards.

### 9.2 `any` in destination

```
deal 1 from Deck private to Hand of any
```

Issues a **`ChoosePlayer` prompt** so the player picks a target from the
candidate list.

### 9.3 Source quantities — the verb decides

The quantity semantics depend on the verb (§6.1):

```
deal any from Deck face up to Discard          // NUMBER prompt: "how many?" (1..pile), then top-N
deal >= 1 and <= 3 from Deck face up to Discard // NUMBER prompt bounded 1..3, then top-N
move any from Hand face up to Discard          // ChooseCards prompt: pick 1..all
move >= 1 and <= 3 from Hand face up to Discard // ChooseCards prompt: pick 1..3 (validated, re-prompted)
move 1 from Hand face up to Discard            // ChooseCards prompt: pick exactly one
```

For `move`/`exchange` the chosen card IDs are written to a synthetic memory
slot and consumed by the replacement edge; `deal` substitutes the chosen
count as a literal quantity and deals from the top. Both chain with player
quantifiers (`move 1 from Hand of any …` prompts for the player first, then
the card) and with dest fan-outs (`deal any from Deck to Hand of all` asks
the count once, then deals to every player).

### 9.4 `any` in setup

```
location Hand on any       // prompts for one player
```

Any quantifier `any` in a setup rule issues a `ChoosePlayer` prompt before
any mutation; the chosen player is substituted into every any-site of the
rule.

---

## 10. Common Patterns & Cookbook

### 10.1 Turn loop

```
stage Play for current 10 times {
    optional {
        // player action here
    }
    cycle to next
}
```

The `cycle to next` at the end of each iteration advances the turn. The
`for current` + `cycle to next` pair is the standard turn-loop pattern. The
`N times` clause provides a safety cap (e.g. 10 iterations for 5 players × 2
actions each).

### 10.2 Hit or stand (Blackjack)

```
optional {
    deal 1 from Deck private to Hand of current
    if (sum of Hand of current using BJ > 21) {
        set current out of game
    }
}
```

`optional` gives the player a hit/stand choice. Accept = draw a card;
decline = stand (skip). The `if` checks for bust (>21) and auto-eliminates.
No guards are needed: a busted player is never prompted again (their
remaining instructions are skipped, §5.4), `cycle to next` never errors
(§6.3), and the stage auto-ends when nobody is left (§5.2).

> **Caveat:** standing is **not recorded** — declining only skips this
> round, so the optional re-asks the same player next round. Bound the stage
> with `N times` and let players re-decline.

### 10.3 Deal N cards per player

```
deal 3 from Deck private to Hand of all
```

The `all` quantifier fans out one `deal` per player automatically.

### 10.4 Score each player's hand

```
stage Score for current 3 times {
    score sum of Hand of current using BJ to current
    cycle to next
}
```

Uses `sum of … using <pointmap>` to compute a hand total and writes it to
the player's `score` field.

### 10.5 Per-round counter check

```
if (stageroundcounter > 3) {
    set current out of game
}
```

Checks how many times the current stage has looped. Useful for time-limit
mechanics.

### 10.6 Eliminate low-scoring players

```
winner is highest score
```

All in-game players are compared; anyone not at the maximum score is
eliminated. For Blackjack this handles the survivors after busted players
were eliminated in play and players who did not beat the dealer were never
scored (see §11.5).

### 10.7 Check hand is empty

```
if (Hand empty) {
    deal 5 from Deck private to Hand of current
}
```

Replenishes hand when empty. Note the syntax: `Hand empty`, not
`card_set_empty(Hand)`.

### 10.8 Conditional branching with `case`

```
conditional {
    case (sum of Hand of current using BJ == 21):
        score 10 to current
    case (sum of Hand of current using BJ > 21):
        set current out of game
    case else:
        cycle to next
}
```

First exact-match wins. `case else:` catches all remaining branches.

### 10.9 Ask the player for a number

```
bid any on Pot of table                  // "bid how much?" (any amount)
bid >= 1 and <= 10 on Bet of table       // "bid how much?" (1..=10)
```

Store the answer in a table memory, then read it back with `&I:M of table`.
A complete betting round: prompt each player in a turn loop, then compare the
memories with `winner is highest Bet`.

---

## 11. Blackjack Walkthrough

Below is the annotated `blackjack.cgdsl` from `test_games/`, built up
section by section. This game models a 3-player table vs. a dealer (casino
rules: dealer hits on <18, stands on ≥18, Ace = 11).

### 11.1 Setup — players, table, deck

```
player P1, P2, P3
turnorder (P:P1, P:P2, P:P3)
location Hand on all
location DealerHand on table
location Deck on table

card on Deck:
  Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King)
    for Suit(Diamonds, Clubs, Hearts, Spades)

points BJ on Rank(
  Ace: 11, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6,
  Seven: 7, Eight: 8, Nine: 9, Ten: 10,
  Jack: 10, Queen: 10, King: 10
)

shuffle Deck
```

Three players with `Hand` per player (via `all`). Dealer gets a separate
`DealerHand` on `table`. The `BJ` point map assigns Blackjack scoring values
(Ace=11, face cards=10). Shuffle before dealing.

### 11.2 Deal — 2 cards per player, 1 for dealer

```
stage Deal for current 1 times {
    deal 2 from Deck private to Hand of P:P1
    deal 2 from Deck private to Hand of P:P2
    deal 2 from Deck private to Hand of P:P3
    deal 1 from Deck private to DealerHand
}
```

Manual per-player deals (could use `of all` with fan-out, but this is
explicit). Dealer gets only one card up front — the second comes after all
players finish (§11.4).

### 11.3 Player turns — hit or stand

```
stage Play for current 12 times {
    optional {
        deal 1 from Deck private to Hand of current
        if (sum of Hand of current using BJ > 21) {
            set current out of game
        }
    }
    cycle to next
}
```

12 iterations = 4 full rounds (4 × 3 players). `optional` prompts hit/stand.
Accept = draw one card, then check bust. Bust → `set current out of game`.
No guards are needed: a busted player is skipped automatically (§5.4) and
`cycle to next` wraps or no-ops instead of erroring (§6.3).

### 11.4 Dealer — auto-play

```
stage Dealer for current 10 times {
    if (sum of DealerHand using BJ < 17) {
        deal 1 from Deck private to DealerHand
    }
}
```

Dealer hits while hand < 17. 10 iterations bounds the worst case. No turn
cycling needed — the `if` guard alone stops the dealer.

### 11.5 Scoring — compare against the dealer

```
stage Score for current 1 times {
    if (sum of Hand of P:P1 using BJ > sum of DealerHand using BJ) {
        score sum of Hand of P:P1 using BJ to P:P1
    }
    if (sum of Hand of P:P2 using BJ > sum of DealerHand using BJ) {
        score sum of Hand of P:P2 using BJ to P:P2
    }
    if (sum of Hand of P:P3 using BJ > sum of DealerHand using BJ) {
        score sum of Hand of P:P3 using BJ to P:P3
    }
}
```

Each player is scored explicitly — the dealer plays once, so there is
nothing to cycle. Players whose hand does not beat the dealer are simply
never scored; they are out of game either way.

### 11.6 Winner determination

```
stage End for current 1 times {
    winner is highest score
}
```

Only in-game players (those who did not bust and beat the dealer) are
compared by score. The highest survives; ties are retained (multiple winners
possible). If every player busts, nobody is in game and nobody wins — the
winner set is empty, and the tooling reports `no winners`.

### 11.7 Full file

```
player P1, P2, P3
turnorder (P:P1, P:P2, P:P3)
location Hand on all
location DealerHand on table
location Deck on table

card on Deck:
  Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King)
    for Suit(Diamonds, Clubs, Hearts, Spades)

points BJ on Rank(
  Ace: 11, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6,
  Seven: 7, Eight: 8, Nine: 9, Ten: 10,
  Jack: 10, Queen: 10, King: 10
)

shuffle Deck

stage Deal for current 1 times {
  deal 2 from Deck private to Hand of P:P1
  deal 2 from Deck private to Hand of P:P2
  deal 2 from Deck private to Hand of P:P3
  deal 1 from Deck private to DealerHand
}

stage Play for current 12 times {
  optional {
    deal 1 from Deck private to Hand of current
    if (sum of Hand of current using BJ > 21) {
      set current out of game
    }
  }
  cycle to next
}

stage Dealer for current 10 times {
  if (sum of DealerHand using BJ < 17) {
    deal 1 from Deck private to DealerHand
  }
}

stage Score for current 1 times {
  if (sum of Hand of P:P1 using BJ > sum of DealerHand using BJ) {
    score sum of Hand of P:P1 using BJ to P:P1
  }
  if (sum of Hand of P:P2 using BJ > sum of DealerHand using BJ) {
    score sum of Hand of P:P2 using BJ to P:P2
  }
  if (sum of Hand of P:P3 using BJ > sum of DealerHand using BJ) {
    score sum of Hand of P:P3 using BJ to P:P3
  }
}

stage End for current 1 times {
  winner is highest score
}
```

---

## 12. Not Implemented / Known Gaps

| Construct | Status | Notes |
|-----------|--------|-------|
| `unless` | ❌ Not in grammar | Use `if (not <expr>)` — note `not (X)` with parens does **not** parse; write `not Hand empty`, `not current out of game` |
| `for <players>` clause in stage | ⚠️ Dropped | All players always in-stage; the clause has no effect (P-1) |
| SimStage (per-player FSM) | ❌ Not implemented | `stage X for all …` runs the same sequential IR as `for current` (P-2) |
| `flip <cardset> to <status>` | ⏳ No-op by design | Becomes (de)encryption with card crypto; the status slot exists |
| `place <token>` / `create token` | ❌ Stub | Tokens not in the data model |
| `demand <type>` | ❌ Stub | Semantics undefined |
| Card status / hidden information | ❌ | No per-player visibility; face-up/down/private is deferred to the crypto work |
| Dice / arbitrary RNG | ❌ Not in grammar | Simulate with shuffled decks |
| Numeric prompts in arbitrary int expressions | ⚠️ | `any`/ranges work in move/deal quantities and `bid … on <memory>`; `score any to …` does not parse |

> The authoritative per-construct status table is
> [`dsl-completeness.md`](./dsl-completeness.md); known divergences with
> repros live in [`engine-vs-design.md`](./engine-vs-design.md).

---

## 13. Running & Debugging

### 13.1 Parse-only check

```
cargo run -p front_end --bin cgdsl2json -- path/to/game.cgdsl
```

Outputs the lowered IR as JSON. Fails immediately if the grammar rejects
the file.

### 13.2 Interactive TUI

```
just tui crates/engine/test_games/blackjack.cgdsl
```

Launches a ratatui terminal UI where you can step through the game,
inspect `GameData`, choose options, and see trace events. The trace log
shows every transition, including the final `GameOver: winners: …` line.

### 13.3 Run existing tests

```
cargo test -p cgdsl-engine
```

Tests cover setup, actions, scoring, quantifiers, and query evaluation.

### 13.4 Trace logging

Set `MCG_TRACE_LOG=path` (or pass `--log <path>` to `cgdsl-play`) to write a
structured trace file: one line per FSM transition, a stamped header, and a
footer naming the winner set.

---

## 14. Quick Reference

### Setup cheatsheet

```
player P1, P2
team Red with (P:P1, P:P2)
turnorder (P:P2, P:P1)
location Hand on all
location Deck, Discard on table
card on Deck: Suit(Hearts, Spades) for Rank(Ace, King)
precedence Ord on Rank(A, K, Q, J)
points Values on Rank(A:11, K:10, Q:10, J:10)
combo Pair where size == 2 and Rank same
memory MyVar on table
memory Pot 100 on table
```

### Stage cheatsheet

```
stage Name for current N times { ... }
stage Name for current until <bool> { ... }
stage Name for current until end { ... }
stage Name for all N times { ... }
```

### Action cheatsheet

```
deal 3 from Deck private to Hand of all     // draw 3, automatic
deal any from Deck private to Hand          // "how many?" then draw
move 1 from Hand private to Discard         // pick exactly one card
move >= 2 and <= 5 from Hand private to Discard  // pick 2..5
move top(Hand) face up to Table             // automatic (positional)
exchange any from Hand face down to Table   // pick 1..N
shuffle Deck
cycle to next
end turn
end stage
end game with winner P:P1
set P:P1 out of game
set current out of Play
M is 42
reset M
bid any on Pot of table                     // numeric input prompt
```

### Scoring cheatsheet

```
score 10 to P:P1
score sum of Hand using BJ to current
score 5 to M of current
winner is P:P1
winner is highest score
winner is lowest position
```

### Expression cheatsheet

```
42, -5, (1 + 2)
size((1,2,3)), sum((1,2,3))
sum of Hand of current using BJ
min of Deck using Values, max of Deck using Values
stageroundcounter
(&I:M of current), (&S:Name of P:P1)
Hand empty, Hand not empty
Rank of top(Hand) == "Ace"
Hand where Rank is "Ace"
current, next, previous
owner of top(Hand)
```
