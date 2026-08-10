---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [cgdsl, authoring, guide, tutorial, beginner]
last_validated: 2026-08-11
---

# CGDSL Authoring Guide

A `.cgdsl` file is a complete description of a card game: the players, the
cards, the piles, and every rule of play. This guide explains each piece from
the ground up, in the order you will use it. Every idea comes with a small,
commented example — you can copy any of them into a file and run it.

To follow along:

- `just tui path/to/your_game.cgdsl` — step through the game in a terminal UI
- `cargo run -p cgdsl-engine --bin cgdsl-play -- path/to/your_game.cgdsl` —
  run it in the terminal and answer the prompts

The engine reads your file, sets up the table, and then follows your rules
step by step, pausing whenever a player must make a decision.

---

## 1. What a game definition looks like

A game file is a flat list of rules, in three groups, in this order:

```
<setup>     who plays, what cards exist, where piles are
<stages>    the rounds of play
<scoring>   who wins
```

Here is a complete, working game — the smallest one that still plays like a
card game. Read the comments; every line is explained.

```
// THE HIGHEST CARD
// Two players each draw a card. The higher card scores a point.
// Ten rounds; most points wins.

player P1, P2                          // two players, P1 goes first
location Hand on all                   // each player owns their own Hand pile
location Deck on table                 // one shared Deck on the table

card on Deck:                          // a 12-card deck, ordered low to high
  Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen)

points CardValue on Rank(              // a number for each rank, for comparing
  Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6,
  Seven: 7, Eight: 8, Nine: 9, Ten: 10, Jack: 11, Queen: 12
)

stage Round for current 10 times {     // play ten rounds
  deal 1 from Deck private to Hand of P:P1   // top card of the Deck, automatically
  deal 1 from Deck private to Hand of P:P2

  if (sum of Hand of P:P1 using CardValue > sum of Hand of P:P2 using CardValue) {
    score 1 to P:P1                    // P1's card is higher
  }
  if (sum of Hand of P:P2 using CardValue > sum of Hand of P:P1 using CardValue) {
    score 1 to P:P2
  }

  deal 1 from Hand of P:P1 private to Deck   // put the played cards back
  deal 1 from Hand of P:P2 private to Deck
  shuffle Deck                         // mix before the next round
}
stage End for current 1 times {
  winner is highest score              // most points wins; ties draw
}
```

Even this tiny game introduces the core ideas you will use in every game:
players, piles, cards with attributes, point values, stages, conditions,
actions, and scoring. The rest of this guide explains each one properly.

---

## 2. Names, players, and the table

### 2.1 Names

Everything you create — players, piles, cards, stages, memories — needs a
name. Names must **start with a capital letter**:

```
P1   Hand   Deck   Round   Pot   BJ
```

Lowercase words are the language's own keywords (`player`, `stage`, `deal`,
`score`, …), so they are reserved.

### 2.2 Players and the table

There are two kinds of "people" in a game: **players** and the **table**.

- A player is someone who can act. You refer to a player with a `P:` prefix:
  `P:Alice`, `P:P1`.
- The table is the shared play area — the place where shared piles like the
  Deck and the Discard live. You refer to it with the word `table`.

```
player P1, P2, P3       // three players, in this order
```

Players act in turn order. By default the order is the order you declared
them; `turnorder` overrides it:

```
turnorder (P:P3, P:P1, P:P2)    // P3 starts, then P1, then P2
turnorder all                   // declaration order (the default)
turnorder all random            // shuffled order
```

### 2.3 Teams

Players can be grouped into teams:

```
team Red with (P:P1, P:P3)
team Blue with (P:P2, P:P4)
```

You refer to a team with a `T:` prefix: `T:Red`. A team can own things —
**one shared instance for the whole team**:

```
location TrickPile on T:Red      // ONE pile shared by the Red team
memory  TeamScore on T:Red       // ONE memory slot, named Red_TeamScore
```

While a Red member is the current player, the bare name `TrickPile` finds the
team's pile; `TrickPile of T:Red` always finds it explicitly.

---

## 3. Piles and cards

### 3.1 Locations (piles)

Cards live in **locations** — piles with names. Each location belongs to an
owner: a player, the table, or a team.

```
location Hand on all          // one Hand per player (P1's Hand, P2's Hand, …)
location Hand on P:P1         // a Hand only for P1
location Deck on table        // one global Deck
location Discard, Stock on table   // several at once, comma-separated
location TeamPile on T:Red    // one shared pile for the whole team
```

`on all` is the usual way to give every player the same pile — a hand, a
personal discard, whatever. Each player gets their own copy.

When you use a pile's bare name, the engine finds it in this order: the
current player's own pile, their team's pile, the table's pile, then the
first pile with that name anywhere. When you need to be exact, say who owns
it: `Hand of P:P2`, `Hand of current`.

### 3.2 Cards

A card is just a set of **attributes** — named labels with values. You create
cards by listing attribute values; the engine makes one card for every
combination:

```
card on Deck:
  Rank(Ace, Two, Three)       // 3 ranks
    for Suit(Hearts, Spades)  // × 2 suits = 6 cards
```

This makes six cards: Ace-of-Hearts, Ace-of-Spades, Two-of-Hearts, and so on.
Every card is unique — two identical cards are two different cards, which
matters for duplicate decks.

You can create several different groups of cards in one block:

```
card on Deck:
  Rank(Ace) for Suit(Hearts),     // one card
  Rank(Two) for Suit(Spades)      // another card
```

**Important:** without a `shuffle`, cards come off the Deck in creation
order — the first card created is the top of the pile. Shuffle if you want
randomness:

```
shuffle Deck
```

### 3.3 Ordering and values

Many games need "which card is higher" or "how much is this card worth".
Two declarations give you that:

```
// A total ordering of one attribute, low → high:
precedence RankOrd on Rank(A, 2, 3, 4, 5, 6, 7, 8, 9, T, J, Q, K)

// A number per attribute value, for sums and comparisons:
points CardValue on Rank(
  Ace: 1, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6,
  Seven: 7, Eight: 8, Nine: 9, Ten: 10, Jack: 11, Queen: 12
)
```

You use them like this:

```
sum of Hand of current using CardValue    // add up the point values of a hand
max of Hand of current using RankOrd      // the single highest card
Rank of top(Hand) == "Ace"                // is the top card an Ace?
```

### 3.4 Named card patterns (combos)

A **combo** is a named card pattern you can reuse — a pair, a set, a flush:

```
combo Pair where (size == 2 and same Rank)
combo Set where (size >= 3 and same Rank)
```

You can check whether a pile contains a combo, and move the matching cards:

```
Pair in Hand                        // the paired cards in the hand
move Set in Hand of current private to Table   // lay down a set (validated)
```

---

## 4. Memories: the game's notebook

A **memory** is a named slot that stores a value — the game's notebook.
Memories belong to an owner, and their names are prefixed with the owner:
`P1_M`, `Table_pot`, `Red_TeamScore`.

```
memory Pot 100 on table          // a table slot starting at 100
memory M on P:P1                 // a slot for P1, starting at 0
memory Winner P:P1 on table      // a slot holding a player's name
```

You set a memory during play, read it back, and reset it:

```
Pot is 5                    // set (targets the declared owner, else the current player)
(&I:Pot of table)           // read it back in an expression
reset Pot                   // back to its zero value
```

Memories can hold more than numbers — names, teams, and lists:

```
M is P:Alice                // a player's name
M is (1, 2, 3)              // a list of numbers
M is (P:Alice, P:Bob)       // a list of players
```

---

## 5. Stages: the rounds of play

A **stage** is a named loop — one round, one turn, one phase of the game.
Its body runs again and again until an **end condition** is met.

```
stage <Name> for <player> <end-condition> {
    <the rules of this stage>
}
```

The `for <player>` part names the player the stage is about (`current` is the
usual choice) — but note that **every stage currently includes all players**;
you scope who acts with `cycle to next` and `set … out of` (below).

### 5.1 End conditions

| You write | The stage stops when… |
|---|---|
| `5 times` | it has run 5 times |
| `until Deck empty` | the condition becomes true |
| `until end` | an action says `end stage` |
| `until Hand empty or 5 times` | either condition |

Examples:

```
stage Draw for current 5 times { … }            // exactly five draws
stage Play for current until Hand empty { … }    // until someone empties their hand
stage Play for current until end { … }           // forever, until told to stop
```

### 5.2 The standard turn loop

Most games are a repeating "your turn, then the next player" loop. This is
the pattern you will use constantly:

```
stage Play for current 10 times {
    // …ask the current player to do something…
    cycle to next          // hand the turn to the next player
}
```

`cycle to next` moves the turn forward, skipping players who are out of the
game or out of the stage. When the current player is the only one left who
can act, the turn stays with them; when nobody can act at all, the game ends
by itself. You never need to guard against that.

### 5.3 When a player is out

`set … out of game` removes a player from the game entirely; `set … out of
<stage>` removes them from one stage:

```
set P:P1 out of game        // eliminated
set current out of game     // self-eliminate (e.g. a bust in Blackjack)
set current out of Play     // leave only this stage
```

From then on the engine never asks that player for input and never runs their
instructions. If no players are left in the game, the stage — and then the
game — ends by itself, with an empty winner set.

---

## 6. Actions: what the game can do

Actions are the verbs of the language. They change the game state: cards
move, piles shuffle, players get eliminated, scores change.

### 6.1 Moving cards: `deal` vs `move`

The most common actions move cards between piles. There are three verbs with
the same shape:

```
<verb> <how many> from <pile> <status> to <pile>
```

The **verb decides who chooses the cards**:

- **`deal` — the pile chooses.** Cards come off the top, automatically.
  `deal 2 from Deck` = "draw the top 2". The player never picks.
- **`move` / `exchange` — the player chooses.** The player picks the cards
  themselves. `move 1 from Hand` = "pick one card from your hand".

The `<status>` word (`face up`, `face down`, `private`) is required but not
yet used by the engine — pick whichever reads best.

The `<how many>` part works like this:

| You write | What happens with `deal` | What happens with `move` |
|---|---|---|
| `3` | the top 3 cards move | the player picks exactly 3 |
| `any` | the player is asked *how many* | the player picks any number |
| `>= 2 and <= 4` | asked for a number 2–4 | the player picks 2–4 |
| *(nothing)* | everything moves | everything moves |

Examples:

```
deal 3 from Deck private to Hand of current      // draw 3, no questions
deal any from Deck private to Hand of current    // "how many?" then draw that many
move 1 from Hand private to Discard              // discard one card, you choose
move >= 2 and <= 5 from Hand private to Discard  // discard 2–5, you choose
```

If you name a specific card position, no one chooses anything — the position
already decided:

```
move top(Hand) face up to Table      // the top card, automatically
move 1 from top(Deck) private to Hand
move 1 from Hand[2] private to Discard
```

A `where` clause filters which cards are on offer, but the player still
chooses: `move 1 from Hand where Rank is "Ace" private to Discard` asks the
player to pick one Ace.

### 6.2 Other actions

```
shuffle Deck                    // mix a pile (in place)

cycle to next                   // next player's turn
cycle to P:Bob                  // a specific player's turn

end turn                        // pass the turn
end stage                       // leave the current stage
end Play                        // leave the named stage
end game with winner P:P1       // declare the winner and end the game

set P:P1 out of game            // eliminate
set current out of Play         // leave one stage

Pot is 5                        // write a memory
reset Pot                       // zero a memory
bid any on Pot of table         // ask the player for a number, store it
```

---

## 7. Asking the player

Four ways to ask the current player something.

### 7.1 `optional` — yes or no

```
optional {
    deal 1 from Deck private to Hand of current
}
```

The player is asked "do you want to…?" — accept runs the block, decline
skips it. (Declining records nothing, so the same optional can be asked again
next round; bound the stage with a round count.)

### 7.2 `choose` — one of several options

```
choose {
    move top(Hand) face down to Discard
    or
    deal 1 from Deck private to Hand
}
```

The player picks one option; each option can itself contain several rules:

```
choose {
    deal 1 from Deck private to Hand of P:P1
    if (Deck empty) {
        deal 1 from Deck private to Hand of P:P2
    }
    or
    score 1 to P:P1
}
```

### 7.3 `if` — the game decides

```
if (Hand empty) {
    deal 5 from Deck private to Hand of current
}
```

There is no `else` — use two complementary `if`s or a `conditional`.

### 7.4 `conditional` — first match wins

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

The cases are checked in order; the first true one runs, `case else` catches
everything else.

### 7.5 `trigger` — run immediately

```
trigger {
    shuffle Deck
}
```

A trigger runs each time the flow reaches it: once for a top-level trigger,
every round for a trigger inside a stage. (There is no separate "on enter"
— use a one-round stage or `if (stageroundcounter == 0)` for first-pass-only
behaviour.)

---

## 8. Reading the state: expressions

Conditions and actions read the game state with **expressions**. The main
families:

### 8.1 Numbers

```
42                          // a literal
(1 + 2)                     // arithmetic: + - * / mod
size(cards Hand of current) // how many cards
sum of Hand of current using BJ      // total point value
stageroundcounter           // how many times the current stage has looped
(&I:Pot of table)           // read a memory
```

### 8.2 Text

```
"Ace"                       // a literal — capitalised
Rank of top(Hand)           // an attribute of a card
(&S:Name of P:P1)           // read a text memory
```

### 8.3 Yes/no questions

```
Hand empty                      // true when a pile has no cards
Hand not empty
"Spades" in Hand                // any card has this attribute value
P:P1 out of game                // is this player eliminated?
current out of Play             // is the current player out of this stage?
(A and B)                       // combine with and / or / not
not Hand empty                  // note: write `not X`, not `not (X)`
```

### 8.4 Players

```
current                 // the player whose turn it is
next                    // the next player who can act
previous                // the previous player who can act
P:Alice                 // a specific player
owner of top(Hand)      // who owns the top card
```

### 8.5 Card sets and filters

Card sets are the "from" and "to" of moves, and the things you ask questions
about:

```
Hand                            // a bare pile name
Hand of P:Alice                 // a specific player's pile
top(Hand)                       // the top card
bottom(Hand)                    // the bottom card
Hand[2]                         // the 3rd card (counting from 0)
Hand where Rank is "Ace"        // only the Aces
Pair in Hand                    // the cards matching the Pair combo
```

Filters combine attributes with `and`/`or` and can compare, count, and order:

```
Hand where Rank is "Ace"
Hand where (same Rank and distinct Suit)
Hand where size == 5
Hand where Rank higher than "Ten" using RankOrd
```

---

## 9. Scoring and winning

### 9.1 Scores

```
score 10 to P:P1                  // add points
score sum of Hand of current using BJ to current
score 5 to Pot of P:P1            // write into a player's memory instead
```

### 9.2 Winners

```
winner is P:P1                    // P1 wins; everyone else is out
winner is highest score           // the highest score wins (ties draw)
winner is lowest score
winner is highest position        // earliest in turn order
winner is highest Pot             // best memory value wins
```

### 9.3 The winner set

Every game ends with a **winner set**: the players still in the game. Whether
you declared a winner (`winner is …`, `end game with winner …`) or the game
simply ran out of rounds, the rule is the same — declared winners eliminate
everyone else, so the survivors are the winners. The engine reports the
winner set when the game ends (the TUI trace, the trace file, and
`cgdsl-play` all show it); nobody left in the game means **no winner**.

---

## 10. A complete game, annotated

Here is the Blackjack game from `test_games/blackjack.cgdsl`, with every part
explained. Three players play against a dealer (played by the table).

```
// ============ SETUP ============

player P1, P2, P3                    // three players
turnorder (P:P1, P:P2, P:P3)         // P1 starts

location Hand on all                 // each player's own hand
location DealerHand on table         // the dealer's cards, on the table
location Deck on table               // the shared deck

// A full 52-card deck:
card on Deck:
  Rank(Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King)
    for Suit(Diamonds, Clubs, Hearts, Spades)

// Blackjack values: Ace = 11, faces = 10:
points BJ on Rank(
  Ace: 11, Two: 2, Three: 3, Four: 4, Five: 5, Six: 6,
  Seven: 7, Eight: 8, Nine: 9, Ten: 10,
  Jack: 10, Queen: 10, King: 10
)

shuffle Deck                         // randomise before dealing

// ============ PLAY ============

// One deal round: two cards to each player, one to the dealer.
stage Deal for current 1 times {
  deal 2 from Deck private to Hand of P:P1
  deal 2 from Deck private to Hand of P:P2
  deal 2 from Deck private to Hand of P:P3
  deal 1 from Deck private to DealerHand
}

// Up to four full rounds of hit-or-stand. `cycle to next` passes the turn;
// a busted player is skipped automatically, and the round ends by itself
// when nobody is left to play.
stage Play for current 12 times {
  optional {                         // "hit?" — accept or decline
    deal 1 from Deck private to Hand of current
    if (sum of Hand of current using BJ > 21) {
      set current out of game        // bust!
    }
  }
  cycle to next
}

// The dealer (the table) draws while below 17. Bounded to 10 attempts.
stage Dealer for current 10 times {
  if (sum of DealerHand using BJ < 17) {
    deal 1 from Deck private to DealerHand
  }
}

// ============ SCORING ============

// Players whose hand beats the dealer score their hand's value.
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

// The highest score wins. If everyone busted, nobody is in the game and
// nobody wins.
stage End for current 1 times {
  winner is highest score
}
```

Run it with `just tui crates/engine/test_games/blackjack.cgdsl` — you will
see the game play itself, pausing for you to hit or stand.

---

## 11. Common patterns

### 11.1 The standard turn loop

```
stage Play for current 10 times {
    optional {
        // the current player's action
    }
    cycle to next
}
```

### 11.2 Deal N cards to every player

```
deal 3 from Deck private to Hand of all
```

`of all` deals to each player in turn, automatically.

### 11.3 Draw until a condition

```
stage Dealer for current 10 times {
    if (sum of DealerHand using BJ < 17) {
        deal 1 from Deck private to DealerHand
    }
}
```

The `if` stops the loop once the condition is met; the round count is a
safety cap.

### 11.4 Replenish an empty hand

```
if (Hand empty) {
    deal 5 from Deck private to Hand of current
}
```

### 11.5 A betting round

```
stage Bet for current 3 times {
    bid any on Pot of table          // "how much?" — a number prompt
    cycle to next
}
```

Each player bids a number into the shared `Pot` memory; read it back with
`(&I:Pot of table)`.

---

## 12. Quick reference

### Setup

```
player P1, P2
team Red with (P:P1, P:P2)
turnorder (P:P2, P:P1)
location Hand on all
location Deck, Discard on table
card on Deck: Suit(Hearts, Spades) for Rank(Ace, King)
precedence Ord on Rank(A, K, Q, J)
points Values on Rank(A:11, K:10, Q:10, J:10)
combo Pair where (size == 2 and same Rank)
memory MyVar on table
memory Pot 100 on table
shuffle Deck
```

### Stages

```
stage Name for current N times { ... }
stage Name for current until <bool> { ... }
stage Name for current until end { ... }
stage Name for all N times { ... }
```

### Actions

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

### Scoring

```
score 10 to P:P1
score sum of Hand using BJ to current
score 5 to M of current
winner is P:P1
winner is highest score
winner is lowest position
```

### Expressions

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

---

## 13. What's not possible yet

| Construct | Status | Notes |
|-----------|--------|-------|
| `unless` | ❌ Not in the language | Use `if (not <expr>)` — and write `not X`, never `not (X)` |
| `for <players>` in a stage | ⚠️ No effect | All players are always in every stage; use `set … out of <stage>` to scope |
| Simultaneous stages | ❌ | `stage X for all …` runs exactly like `for current` |
| `flip <cardset> to <status>` | ⏳ No-op by design | Reserved for the card-crypto work |
| Tokens / `place` | ❌ | No token data model |
| `demand` | ❌ | Semantics undefined |
| Card visibility / hidden information | ❌ | No per-player privacy yet (the crypto work) |
| Dice / random numbers | ❌ | Simulate with shuffled decks |
| Asking for a number anywhere | ⚠️ | Works in `bid … on <memory>`, `deal any`, `deal >= M and <= N`; not in `score any` |

The authoritative per-construct status table is
[`dsl-completeness.md`](./dsl-completeness.md); known divergences with repros
live in [`engine-vs-design.md`](./engine-vs-design.md).
