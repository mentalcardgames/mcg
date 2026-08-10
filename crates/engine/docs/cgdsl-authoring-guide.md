---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [cgdsl, authoring, guide, tutorial, blackjack]
last_validated: 2026-08-09
---

# CGDSL Authoring Guide

> **Purpose:** a practical guide for writing `.cgdsl` game definitions. Every
> construct is described with its exact syntax (from `grammar.pest`) and runtime
> behaviour (from the engine). This document is the **authoring reference**; for
> a complete semantics specification see `docs/dsl-semantics.md`.

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

Unlike many card-game DSLs, setup rules **do not** use a `create` keyword:

```
WRONG:  create players P1, P2
RIGHT:  player P1, P2
```

The `create` keyword exists in the grammar but is never used by any rule.

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

Overrides the default turn order. `random` shuffles via `rand::thread_rng()`.

### 3.4 Locations

```
location Hand on all                // one Hand per player
location Hand on P:P1               // Hand only for P1
location Deck on table              // one global Deck
location Discard, Stock on table    // comma-separated: multiple at once
location TeamPile on T:Red          // one TeamPile per team member (since 2026-08-10)
```

The `on` clause accepts any owner expression (`P:Name`, `table`, `all`,
`any`, player collections, team owners, etc.). A **team owner** (`on T:Red`)
creates one location per team member, exactly like `on (P:P1, P:P2)` — the
data model has no shared team-entity pile (P-7).

⚠️ `on any` prompts for one player at setup (I-20, relaxed 2026-08-10) — the
chosen player is substituted before the rule runs. Use `on all` for a
per-player location for everyone.

### 3.5 Cards

```
card on Deck:
  Rank(Ace, Two, Three)
    for Suit(Hearts, Spades)
```

Creates the **cartesian product** of the key-value sets: here 3 ranks × 2
suits = 6 cards. `for` chains create additional dimensions (multi-key
combination). Cards are appended to the named location.

You can also create multiple card types on the same location separated by
commas:

```
card on Deck:
  Rank(Ace) for Suit(Hearts),
  Rank(Two) for Suit(Spades)
```

### 3.6 Precedences

Define a total ordering for a card attribute (used by filter expressions):

```
precedence RankOrd on Rank(A, 2, 3, 4, 5, 6, 7, 8, 9, T, J, Q, K)
```

Shorthand form:

```
precedence RankOrd (Rank:A, Rank:2, Rank:3)
```

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

The int expressions for values **are evaluated** at setup time via
`eval_int`. Runtime expressions like `&I:m of Current` work here.

### 3.8 Combos

Define named card-filter combinations (stored for future use, not evaluated
during play):

```
combo Pair where size == 2 and Rank same
```

See §4.6 for filter syntax.

### 3.9 Memories

```
memory M on table                          // no type → defaults to Int(0)
memory InitialScore 42 on P:P1             // Int, initialised to 42 (value honoured since 2026-08-10)
memory NameOfFirst "Ace" on P:P1           // String, initialised to "Ace"
memory Winner P:P1 on table                // Player, initialised to the evaluated player's name
memory Scores (1, 2, 3) on table           // IntCollection, initialised to the evaluated list
```

Memories are stored in a global `HashMap` keyed as `<Owner>_<MemoryName>`.
The owner determines the initial key prefix, but set/read operations use
whatever owner they specify (§5.6).

⚠️ Setup `with I: 0` syntax does **not** exist in the grammar. Use a bare
expression: `memory Name 42 on P:P1`.

⚠️ **Team owners** (`memory M on T:Red`) create one slot **per team member**
(`P1_M`, `P2_M`, …) — mirroring `location X on T:Red` (§3.4).

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
| Memory ref | `(&I:M of P:P1)` | Requires explicit owner |

The runtime counters increment once per stage-loop iteration:
- `stageroundcounter` — counter of the currently-executing stage
- `stageroundcounter(Play)` — counter of a named stage

These are useful for limiting iterations within a stage body (e.g. "stop
dealing after 5 draws").


### 4.2 String expressions

| Form | Example | Notes |
|------|---------|-------|
| Literal | `"Ace"` | Capitalised ident only |
| Key of card | `Rank of top(Hand)` | Attribute value of a card |
| Collection index | `("A","B")[0]` | 0-based index |
| Memory ref | `(&S:M of P:P1)` | Requires explicit owner |

### 4.3 Boolean expressions

| Form | Example | Notes |
|------|---------|-------|
| Comparison | `X == Y`, `X < Y` | Any two int/string/player/team/cardset exprs |
| Set empty | `Hand empty` | True if no cards |
| Set not empty | `Hand not empty` | True if ≥1 card |
| String in cardset | `"Hearts" in Hand` | Any card has attr=string |
| String not in cardset | `"Hearts" not in Hand` | Negated |
| Logical | `not X` | Unary negation |
| Logical | `(X and Y)`, `(X or Y)` | Binary, parenthesised |
| Out-of check | `P:P1 out of game` | Player eliminated? |
| Out-of check | `current out of Play` | Player out of stage? |


### 4.4 Player expressions

| Form | Example | Notes |
|------|---------|-------|
| Literal | `P:Alice` | Named player |
| Runtime | `current` | Current turn player |
| Runtime | `next` | Next eligible (in-game, in-stage) |
| Runtime | `previous` | Previous in turn order |
| Runtime | `competitor` | Another player in same team |
| Index | `turnorder[2]` | Nth in turn order |
| Index | `(P:A,P:B)[0]` | Nth in collection |
| Card owner | `owner of top(Hand)` | Who owns a card? |
| Memory owner | `owner of highest M` | Player with max memory |

⚠️ `owner of highest <memory>` has a known bug: the engine builds the
lookup key as `<memory>_<player>` (e.g. `M_P1`), but `set_memory` and
`score … to memory` write keys as `<player>_<memory>` (e.g. `P1_M`).
These will never match — see §12.

### 4.5 Player collections

| Form | Example | Notes |
|------|---------|-------|
| Explicit | `(P:A, P:B, P:C)` | Comma-separated |
| All | `all` | All in-game players |
| Any | `any` | Prompt player to pick |
| In-game | `playersin` | Players with `in_game=true` |
| Out-of-game | `playersout` | Players with `in_game=false` (eliminated) |
| Others | `others` | All players except `current` |
| Memory | `(&PC:names of P:P1)` | Read from memory |

### 4.6 Card sets

Card sets are the "from" and "to" of move/deal actions, and the target of
boolean checks.

| Form | Example | Notes |
|------|---------|-------|
| Location | `Hand` | Resolves: current player → Table → first match |
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
min of Hand using BJ    // card with lowest point-map value
max of Hand using RankOrd   // card with highest precedence
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

The `for` clause is **mandatory**. Both forms produce the same IR (per-player
fan-out is not yet implemented):

```
stage Play for current 3 times { ... }     // seq (player_expr)
stage Reveal for all 1 times { ... }       // sim (player_collection)
```

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

### 5.3 Stage lifecycle

1. **Entry**: `ensure_stage_entered(name)` runs on first encounter. Marks
   all players `in_stage[name] = true`. Idempotent if the stage is already
   on the stack.
2. **Loop body**: Each iteration evaluates the end condition, then (if not
   exiting) runs the body rules, increments the stage round counter, and
   loops back.
3. **Exit**: On end-condition match or `end stage` action, the stage is
   popped from `stage_stack`.

### 5.4 Trigger rules

Trigger rules fire each time they are encountered in the flow:

```
trigger {
    shuffle Deck
}
```

A `trigger` block inside a stage fires on every iteration (when the flow
reaches it). A top-level `trigger` fires once before any stage. There is
no dedicated `on enter` syntax: a `trigger` at the start of a stage body
fires **every iteration**, so "entry-only" behaviour is expressed with a
1-iteration stage (`stage X for current 1 times`) or a
`if (stageroundcounter == 1)` guard (the counter starts at 0 and increments
once per loop-back).

Triggers are lowered as `Payload::Trigger` edges and dispatched immediately.

---

## 6. Actions

Actions mutate `GameData`. The three move verbs share one syntax and engine
path, but since 2026-08-10 **the verb carries the choice semantics**:

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

The **status** field is **mandatory** — even though the engine ignores it (for now!):

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

**The `<quantity>` field (verb-aware since 2026-08-10):**

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

Randomises the card order in-place. Silently no-ops on error (stderr log).

### 6.3 Cycle (change current player)

```
cycle to next
cycle to P:Bob
cycle to current
```

Sets `current_player` in `turn_order`. `next` walks turn order, skipping
eliminated or out-of-stage players; with no eligible *other* player the turn
wraps onto the current player (I-13, relaxed 2026-08-10), and with nobody
eligible at all `cycle to next` is a **no-op** — games that eliminate players
need no guards, and the stage auto-ends when no players remain (§2.6
semantics).

### 6.4 End scope

```
end turn              // advance to next player
end stage             // leave current stage
end Play              // leave named stage
end game with winner P:P1   // declare the winner(s) and end the game
```

`end turn` calls `next_player()` which scans `turn_order` for the next
eligible player, wrapping onto the current player when it is the only
eligible one (since 2026-08-10 it no longer strands the game).

`end game with winner <players>` eliminates everyone not named (2026-08-10)
and ends the game. **Winner set:** every player still in the game at the end
is a winner (`GameData::winner_names`) — whether declared via
`winner is …` / `end game with winner …` (which eliminate the rest) or
simply because the game ran out of stages with players left in. With nobody
left in the game, the winner set is empty.

### 6.5 Set player out

```
set P:P1 out of game         // eliminate from game entirely
set P:Bob out of Play         // mark out of a specific stage
set current out of game       // self-eliminate
set current out of stage      // exit current stage for this player
```

`out of game` sets `in_game = false`. `out of stage` / `out of <name>` sets
the player's `in_stage[<name>] = false`.

### 6.6 Memory operations

```
// Set memory (action — the target owner is resolved automatically)
M is 42                      // Int
M is "Hello"                  // String
M is P:Alice                  // Player (stored as String)
M is T:Red                    // Team
M is (1, 2, 3)                // IntCollection (evaluated since 2026-08-10)
M is ("A", "B")               // StringCollection
M is (P:Alice, P:Bob)        // PlayerCollection
M is (top(Hand), bottom(Hand)) // CardSet

// Read back (expression — requires explicit owner)
(&I:M of P:P1)   // reads Int memory
score (&I:M of current) to current

// Reset (resets every variant to its typed zero since 2026-08-10)
reset M
```

⚠️ **Write/read owner resolution (D-14, since 2026-08-10):** the write rules
have no `of <owner>` clause, so `M is 5` / `reset M` (and bare reads `&I:M`)
target the **declared owner** when exactly one slot ends in `_M` exists —
`memory pot on table` + `pot is 5` writes `Table_pot` — and otherwise the
**current player's** slot. This replaced the old behaviour where a
table-declared memory was silently written under the current player's name.

⚠️ Collection memory variants are fully evaluated (literals, and
`Memory`-form copies of existing collection slots).

⚠️ Bare `&I:M` (no `of <owner>`) resolves through the same owner resolution
as writes; it errors only when neither a declared slot nor a current player
exists.

### 6.7 Stubs

| Action | Status |
|--------|--------|
| `flip <cardset> to <status>` | ⏳ No-op by design — becomes (de)encryption with card crypto; the status slot exists |
| `place <token> from ... to ...` | ❌ Tokens not modeled |
| `bid <quantity>` (no memory target) | ❌ Recoverable error since 2026-08-10 — use the memory form |
| `bid <quantity> on <memory> of <owner>` | ✅ **Numeric input prompt** (2026-08-10): `any`/range → `InputType::Number` prompt (bounds validated, re-asked); literal → writes `{owner}_{memory}` directly. See §6.8 |
| `demand <type>` / `demand ... as <memory>` | ❌ Semantics undefined |

### 6.8 Numeric input — `bid <quantity> on <memory> of <owner>` (2026-08-10)

```
bid any on Pot of table                  // prompt for any number
bid >= 1 and <= 10 on Bet of table       // prompt, bounded 1..=10
bid 5 on Pot of table                    // literal: write 5, no prompt
```

Asks the current player for a number and stores it in the owner's memory slot
(`InputType::Number`). Out-of-range answers are rejected and re-asked. This is
the DSL surface for betting/ante mechanics while the grammar has no `any` in
pure int expressions (see `engine-vs-design.md` §5).

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

### 7.3 `choose`

```
choose {
    move top(Hand) face down to Discard
    or
    deal 1 from Deck private to Hand
}
```

Presents a **multi-choice** prompt. Each `or`-separated arm is a **sequence**
of flow components executed in order when selected (fixed 2026-08-09 —
previously every component became its own arm). The player selects one arm by
index. There can be 1+ arms (though having only one is equivalent to an
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
`case else:` (no condition) acts as a catch-all. Once a `case:` without a
condition is hit, all subsequent cases are marked **unreachable** with a
diagnostic.

### 7.5 Trigger

```
trigger {
    shuffle Deck
}
```

Fires immediately when encountered. Used for `on enter:` blocks in stages.

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
target value remain.

⚠️ `winner is highest/lowest position` uses turn-order index (lower = earlier
in turn). `winner is highest/lowest <memory>` reads the per-player memory slot
`<player>_<memory>` — this is the correct key format, unlike the
`owner of highest/lowest <memory>` player expression which has a key-order
bug (see §12).

---

## 9. Quantifiers

Quantifiers expand a single edge into multiple runtime paths.

### 9.1 `all` in destination

```
deal 1 from Deck private to Hand of all
```

Builds a **fan-out chain** of synthetic edges — one per player. The FSM
automatically steps through each. No player prompt.

### 9.2 `any` in destination

```
deal 1 from Deck private to Hand of any
```

Issues a **`ChoosePlayer` prompt** so the player picks a target from the
candidate list.

### 9.3 `any` / range in source

The quantity semantics depend on the verb (2026-08-10, see §6.1):

```
deal any from Deck face up to Discard          // NUMBER prompt: "how many?" (1..pile), then top-N
deal >= 1 and <= 3 from Deck face up to Discard // NUMBER prompt bounded 1..3, then top-N
move any from Hand face up to Discard          // ChooseCards prompt: pick 1..all
move >= 1 and <= 3 from Hand face up to Discard // ChooseCards prompt: pick 1..3 (validated, re-prompted)
move 1 from Hand face up to Discard            // ChooseCards prompt: pick exactly one
```

For `move`/`exchange` the chosen card IDs are written to a synthetic memory
slot and consumed by the replacement edge; `deal` substitutes the chosen
count as a literal quantity and deals from the top.

### 9.4 `any` in setup — prompts (since 2026-08-10)

```
location Hand on any       // prompts for one player (I-20)
```

Any quantifier `any` in a setup rule issues a `ChoosePlayer` prompt before
any mutation; the chosen player is substituted into every any-site of the
rule. (Previously rejected with a runtime error.)

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
`for current` + `cycle to next` pair is the standard turn-loop pattern.
The `N times` clause provides a safety cap (e.g. 10 iterations for 5
players × 2 actions each).

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
Since 2026-08-10 no guards are needed around this pattern:

- a busted player is **never prompted again** — once `set current out of
  game` fires, the rest of their turn is skipped (ineligible-player skip,
  I-24);
- `cycle to next` never errors — with no eligible *other* player the turn
  wraps onto the current player (or no-ops when nobody at all is eligible),
  and the stage auto-ends when no players remain in the game.

> **Caveats (see `engine-vs-design.md` D-3):**
> - Standing is **not recorded** — declining only skips this round, so the
>   optional re-asks the same player next round. Bound the stage with
>   `N times` and let players re-decline.

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
explicit). Dealer gets only one card up front — second card comes after all
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
No guards are needed (2026-08-10): a busted player is skipped automatically
(ineligible-player skip, I-24) and `cycle to next` wraps/no-ops instead of
erroring (I-13 relaxed).

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

Each player is scored explicitly — no turn cycling here, because
"one pass per player" cannot be expressed safely with `cycle to next` once
players drop out (D-1). Players whose hand does not beat the dealer are
simply never scored; they are out of game either way.

### 11.6 Winner determination

```
stage End for current 1 times {
    winner is highest score
}
```

Only in-game players (those who did not bust and beat the dealer) are
compared by score. The highest survives; ties are retained (multiple winners
possible). If every player busts, nobody is in game and nobody wins.

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
| `unless` | ❌ Not in grammar | Use `if (not <expr>)` — note `not (X)` with parens does **not** parse (P-8); write `not Hand empty`, `not current out of game` |
| `for <players>` clause in stage | ⚠️ Dropped | All players always in-stage (B-1) |
| SimStage (per-player FSM) | ❌ Not implemented | `build_sim_stage` = same IR as seq (B-3) |
| `flip <cardset> to <status>` | ⏳ No-op by design | Becomes (de)encryption with card crypto; the status slot exists |
| `place <token>` | ❌ Stub | Tokens not in data model |
| `create token` | ❌ Stub | Tokens not in data model |
| `bid <quantity>` (no target) | ❌ Error since 2026-08-10 | Use `bid <qty> on <memory> of <owner>` — the numeric input prompt (§6.8) |
| `demand <type>` | ❌ Stub | Semantics undefined |
| `end game with winner <players>` | ✅ | Declares the winners (everyone else eliminated, 2026-08-10) and ends the game |
| Collection memory writes | ✅ | Fully evaluated since 2026-08-10 |
| `reset memory` on non-Int | ✅ | Resets every variant to its typed zero since 2026-08-10 |
| Memory initial values | ✅ | `memory X 42 on P:P1` honours the value since 2026-08-10 |
| `cycle to next` with one eligible player | ✅ | Self-wraps since 2026-08-10 — no `size(playersin) >= 2` guard needed |
| Prompting eliminated players | ✅ | Never happens since 2026-08-10 (ineligible-player skip, I-24); stages auto-end with an empty winner set |
| Team-owned locations/memories | ✅ | Per-member instances since 2026-08-10 (P-7) |
| Aggregate memory (multi-owner) | ✅ | Implemented 2026-08-09 (`(&I:M of all)` aggregates per-owner slots) |
| `owner of highest/lowest <mem>` | ✅ | Key-order bug fixed 2026-08-09 (`<player>_<mem>`) |

> Statuses above are a convenience excerpt — the authoritative status table is
> [`dsl-completeness.md`](./dsl-completeness.md); divergences with repros live
> in [`engine-vs-design.md`](./engine-vs-design.md).

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
inspect `GameData`, choose options, and see trace events.

### 13.3 Run existing tests

```
cargo test -p cgdsl-engine
```

Tests cover setup, actions, scoring, quantifiers, and query evaluation.

### 13.4 Trace logging

The TUI writes a `mcg-trace.log` file with structured trace entries
(actions, condition evaluations, stage counters, etc.) at
`crates/engine/mcg-trace.log`.

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
move top(Hand) face up to Discard
deal 3 from Deck private to Hand of all
exchange any from Hand face down to Table
shuffle Deck
cycle to next
end turn
end stage
set P:P1 out of game
set current out of Play
M is 42
reset M
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
