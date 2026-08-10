---
type: agent_wiki_node
module: crates::engine
scope: [all — mechanics capability matrix]
topics: [mechanics, capability-matrix, handover, gaps]
last_validated: 2026-08-10
---

# Card Game Mechanics — Capability Matrix

> **Purpose:** for each card-game mechanic, which of the three components
> (DSL language, parser/front_end, engine) supports it? Reading the three
> columns tells you where the roadblock is:
>
> - ✅ **DSL** — the language has a construct for it
> - ✅ **Parser** — the construct parses, lowers to IR, and survives to the
>   engine boundary (`front_end`: grammar, AST, IR builder)
> - ✅ **Engine** — the interpreter/actions/queries execute it
>
> ⚠️ = partial (see the note), ❌ = not supported. If DSL and parser are ✅
> but engine is ❌ → engine work. If DSL is ✅ but parser ❌ → parser/IR
> work. If all three are ❌/⚠️ → a language-design decision comes first.
>
> Per-construct implementation details: [`dsl-completeness.md`](./dsl-completeness.md).
> The "wanted" semantics: [`dsl-semantics.md`](./dsl-semantics.md).
> Bugs & divergences: [`engine-vs-design.md`](./engine-vs-design.md).

---

## 1. Cards & the Deck

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| Arbitrary card attributes | any | ✅ | ✅ | ✅ | `card on Loc: Key(values) for Key(values)` — any keys/values |
| Multiple decks | Canasta, double-deck Poker | ✅ | ✅ | ✅ | One `card on DeckA:` block per deck |
| Partial / custom decks | Skat (32), Piquet | ✅ | ✅ | ✅ | No requirement of 52 cards |
| Jokers / extra cards | Joker, Euchre | ✅ | ✅ | ✅ | Just another value in a key |
| Card ordering | War, Euchre | ✅ | ✅ | ✅ | `precedence Name on Key(...)` |
| Point values per card | Blackjack, Cribbage | ✅ | ✅ | ✅ | `points PM on ...` + `sum/max/min … using PM` |
| Dual-value cards (Ace 1/11) | Blackjack | ❌ | ❌ | ❌ | **Language design**: no conditional valuation construct exists at all |
| Face-up / down / private | Memory, Old Maid | ✅ | ✅ | ❌ | **Engine work (crypto-deferred)**: grammar + parser handle `flip`/status; engine no-ops (D-6) |
| Tokens / markers | Cribbage board | ✅ | ✅ | ❌ | **Engine work**: `token`/`place` parse; no data model (D-6) |
| Dice / arbitrary RNG | party variants | ❌ | ❌ | ❌ | **Language design**: no RNG int expression; simulate via shuffled decks |
| Duplicate attribute sets allowed | any | ✅ | ✅ | ✅ | Two identical Ten-Diamonds are distinct cards (intentional) |

## 2. Setup

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| Fixed player count | any | ✅ | ✅ | ✅ | `player P1, P2, P3` |
| Deal fixed counts | most | ✅ | ✅ | ✅ | `deal 2 from Deck private to Hand of P:P1` |
| Round-robin deal to all | Poker, Hearts | ✅ | ✅ | ✅ | `deal 1 … to Hand of all` (sequential fan-out) |
| Deal to a chosen player | gift mechanics | ✅ | ✅ | ✅ | `to Hand of any` → `ChoosePlayer` |
| Take from a chosen player | Go Fish ask | ✅ | ✅ | ✅ | `deal Hand where Rank is "X" of any …` — source `any` prompts `ChoosePlayer` (engine: `SourcePlayerAny`, 2026-08-10) |
| Turn order fixed/random | any | ✅ | ✅ | ✅ | `turnorder all` / `turnorder all random` |
| Teams | Bridge, Doppelkopf | ✅ | ✅ | ✅ | `team T1 with all`; `location X on T:T1` = one instance per member (P-7, fixed 2026-08-10) |
| First player by highest card | Spades, Hearts | ⚠️ | ✅ | ✅ | `owner of max of <cardset> using PM` works for one pile; **cross-player** comparison not expressible — DSL gap |
| Removing cards pre-deal | Euchre kitty | ✅ | ✅ | ✅ | Any table pile works |

## 3. Player state & resources

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| Scores | most | ✅ | ✅ | ✅ | `score N to <players>` |
| Per-player counters / flags | Uno direction | ✅ | ✅ | ✅ | `M is 1` targets the declared slot, else the current player (D-14 fixed 2026-08-10) |
| Per-player set-table writes | Go Fish books | ✅ | ✅ | ✅ | `score N to M of <players>` → `{player}_M` |
| Chips / money | Poker | ✅ | ✅ | ✅ | Memory arithmetic + the **numeric input prompt** (`bid any on Pot of table`, 2026-08-10) |
| Bankruptcy / zero chips | Poker | ✅ | ✅ | ⚠️ | Expressible via chip-memory conditions; no "not playable" state exists (see `plans/bankrupt.md`) — engine gap |
| Hand limits | shedding games | ✅ | ✅ | ✅ | Guard conditions on `size(cards Hand of current)` |
| Elimination from game | Blackjack, Rummy | ✅ | ✅ | ✅ | `set … out of game`; `winner is X` eliminates the rest |
| Elimination from a stage | Hearts | ✅ | ✅ | ✅ | `set … out of stage` / `out of Play` |
| Hidden/private information | Poker, Memory | ❌ | ❌ | ❌ | **All three**: no per-player visibility concept; privacy is the crypto work (D-6, NEXT_STEPS §3) |
| Player-identity validation | any | ✅ | ✅ | ✅ | Only the current player's inputs accepted (I-23) |

## 4. Turn structure

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| Sequential turns | most | ✅ | ✅ | ✅ | `cycle to next` |
| Turn order reversal | Uno, Mao | ✅ | ✅ | ✅ | `cycle to previous` skips ineligible players and wraps to self (D-12 fixed 2026-08-10) |
| Skip a player's turn | Uno skip card | ✅ | ✅ | ✅ | `cycle to next` twice |
| Choose who acts next | target cards | ✅ | ✅ | ✅ | `cycle to P:P2` / `Hand of any` |
| Turn / round caps | any bounded game | ✅ | ✅ | ✅ | `for current N times` |
| Until-loops | War, shedding | ✅ | ✅ | ✅ | `until <bool>` / `until <bool> or N times` |
| Infinite stage (exit via action) | many | ✅ | ✅ | ✅ | `until end` + `end stage` |
| Simultaneous play | Snap, party games | ✅ | ⚠️ | ⚠️ | **Parser/IR work**: `sim_stage` parses but lowers identically to sequential (P-2) |
| Several actions per turn | most | ✅ | ✅ | ✅ | Multiple statements in a stage body |
| Only current player acts | most | ✅ | ✅ | ✅ | Enforced by input validation (I-23); eliminated players are never prompted (I-24, 2026-08-10) |

## 5. Actions & moves

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| Draw a fixed count | most | ✅ | ✅ | ✅ | `deal 2 from Deck …` |
| Draw a computed count | memory-driven games | ✅ | ✅ | ✅ | `deal (&I:M of current) from Deck …` (quantities live since D-8) |
| Draw-until-condition | Blackjack dealer | ✅ | ✅ | ✅ | Bounded `if` loops |
| Play a card to the table | most | ✅ | ✅ | ✅ | `move top(Hand) private to Discard` |
| Play to a specific player's pile | Go Fish | ✅ | ✅ | ✅ | `to Hand of P:P1` |
| Play a *chosen* card | Crazy Eights | ✅ | ✅ | ✅ | `deal any from Hand of current …` |
| Discard N with a size range | Poker | ✅ | ✅ | ✅ | `deal >= 1 and <= 3 from …` (validated, re-prompts) |
| Move cards matching a rule | Rummy melds | ✅ | ✅ | ✅ | `where`-filters and combo groups |
| Lay down sets / books | Rummy, Go Fish | ✅ | ✅ | ✅ | `combo Set where (same Rank and size >= 3)` + validated lay-down move (D-16 note: 2+2 splits pass) |
| Draw from top / bottom / Nth | most | ✅ | ✅ | ✅ | `top(Loc)` / `bottom(Loc)` / `Loc[N]` |
| Exchange cards | Draw Poker | ✅ | ✅ | ✅ | `exchange` |
| Shuffle a pile (incl. partial) | any | ✅ | ✅ | ✅ | `shuffle Deck` — partial sets shuffled in place (F-5) |
| Flip face up/down | Memory, Solitaire | ✅ | ✅ | ❌ | **Engine work (crypto)**: parses, no-op (D-6) |
| Pass a card to a chosen player | Hearts passing | ✅ | ✅ | ✅ | `deal 1 … to Hand of any` |
| Take cards from another player | Go Fish | ✅ | ✅ | ✅ | `deal (Hand where Rank is "X" of next) …`; `of any` for a chosen target (2026-08-10) |
| Steal / capture | Slapjack variants | ✅ | ✅ | ✅ | Moves with filters |
| Return cards to the deck | most | ✅ | ✅ | ✅ | Any move into the deck location |
| Side pile / kitty | Canasta, Euchre | ✅ | ✅ | ✅ | Any location works |
| Sort / reorder a hand | UI nicety | ❌ | ❌ | ❌ | **Language design**: no sort construct |
| Bid / auction | Bridge, Poker | ✅ | ✅ | ✅ | `bid <qty> on <memory> of <owner>` — the numeric input prompt (2026-08-10); plain `bid` errors; `demand` still stubs (D-7) |
| Antes / blinds / pot | Poker | ✅ | ✅ | ✅ | Table memory + arithmetic + the numeric prompt |
| Place tokens | Cribbage board | ✅ | ✅ | ❌ | **Engine work**: tokens not modeled (D-6) |

## 6. Conditions & branching

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| If | most | ✅ | ✅ | ✅ | `if (<bool>) { … }` |
| Optional (accept/decline) | Blackjack, Crazy Eights | ⚠️ | ⚠️ | ⚠️ | **DSL/grammar work**: no else-branch; decline runs nothing (D-3) |
| Multiple-choice | Go Fish rank ask | ✅ | ✅ | ✅ | `choose { … or … }` — unlimited `or`-groups, multi-statement |
| Chained conditionals | many | ✅ | ✅ | ✅ | `conditional { case … case else … }` |
| Automatic triggers | setup steps | ✅ | ✅ | ✅ | `trigger { … }` |
| Int/string/player/cardset comparisons | most | ✅ | ✅ | ✅ | `== != < > <= >=` (ints); equality on the rest |
| Emptiness / membership | many | ✅ | ✅ | ✅ | `X empty`, `X not empty`, `"Ace" in Hand` |
| Multi-condition booleans | many | ⚠️ | ⚠️ | ✅ | **Parser/grammar work**: parenthesised `(A and B)` only; `not (X)` rejected (P-8) — write `not X` |
| Per-player aggregation in conditions | "all players have ≥3 cards" | ⚠️ | ⚠️ | ⚠️ | Per-player memory aggregates work (F-13); general iteration over players in one predicate is not expressible — DSL design |

## 7. Scoring & winning

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| Add points to players | most | ✅ | ✅ | ✅ | `score <int> to <players>` |
| Score from card values | Blackjack, Cribbage | ✅ | ✅ | ✅ | `score sum of Hand of current using PM to current` |
| Write scores to memories | multi-round games | ✅ | ✅ | ✅ | `score N to M of <players>` |
| Winner by highest/lowest score | most | ✅ | ✅ | ✅ | `winner is highest/lowest score` |
| Winner by memory extrema | Go Fish books | ✅ | ✅ | ✅ | `winner is highest M` → `{player}_M` |
| Winner by position | party games | ⚠️ | ✅ | ⚠️ | **DSL/engine semantics**: turn-order index interpretation may not match intent (D-10) |
| Multiple winners / ties | most | ✅ | ✅ | ✅ | All matching extrema remain |
| Last-man-standing | elimination games | ✅ | ✅ | ✅ | `winner is X` / `out of game` + end condition |
| End by deck/hand empty, N rounds | most | ✅ | ✅ | ✅ | `until Deck empty`, `for current N times` |
| End with declared winner | scripted ends | ✅ | ✅ | ✅ | `end game with winner <players>` jumps to the goal |
| Trick / round tallying | Hearts, Euchre | ✅ | ✅ | ✅ | Per-player memories + `ScoreMemory` |
| Hand-ranking combos (pairs, flushes, straights) | Poker, Rummy | ⚠️ | ✅ | ⚠️ | **DSL filter design + engine**: `same/adjacent` exist but group-size constraints over-approximate (D-16) |
| Hand value for showdown | Poker | ⚠️ | ⚠️ | ⚠️ | Additive sums + combo bonuses only; exact poker ranking not expressible |
| Sudden-death / replay on tie | tournaments | ⚠️ | ⚠️ | ⚠️ | Re-deal stages expressible; no automatic tie-breaking |

## 8. Player interaction & input

| Mechanic | Example games | DSL | Parser | Engine | Notes / what's missing |
|---|---|---|---|---|---|
| Yes/no prompt | most | ✅ | ✅ | ✅ | `optional` |
| Choose among options | most | ✅ | ✅ | ✅ | `choose` — unlimited options; TUI list navigation |
| Choose a player | target cards, gifts | ✅ | ✅ | ✅ | `Hand of any` → `ChoosePlayer` |
| Choose one card | many | ✅ | ✅ | ✅ | `deal any from …` |
| Choose a range-sized subset | Poker discard | ✅ | ✅ | ✅ | `deal >= 1 and <= 3 from …` (re-prompts) |
| Chained choices on one move | Go Fish ask | ✅ | ✅ | ✅ | `deal any from Hand of any …` — sequential prompts (player, then cards) since 2026-08-10 |
| Validated combo lay-down | Rummy, Go Fish books | ✅ | ✅ | ✅ | Combo-source move (0 = skip, re-prompt on mismatch) |
| **Enter a number** (bid amount, ante) | Poker, Cribbage | ✅ | ✅ | ✅ | **`bid <qty> on <memory> of <owner>`** (2026-08-10): `any`/range → `InputType::Number` prompt; literal → direct write (F-26). Grammar-level `any` in *pure* int slots (`score any …`) remains front_end work (§9) |
| Hidden simultaneous choices | Poker, RPS | ❌ | ❌ | ❌ | Needs SimStage (P-2) + per-player input routing |

---

## 9. Gap summary — mechanic → roadblock component

| Missing mechanic | Roadblock | Where it lives |
|---|---|---|
| ~~Numeric input prompt~~ | **Done 2026-08-10** — `bid <qty> on <memory> of <owner>` (`InputType::Number`); `any` in pure int slots still needs grammar | §8; F-26 |
| Face-up/down/private state | **Engine** (crypto-deferred, D-6) — DSL+parser already ✅ | §1/§5 |
| Tokens | **Engine** (data model, D-6) — DSL+parser already ✅ | §1/§5 |
| Simultaneous stages + input | **Parser/IR** (P-2) | §4/§8 |
| Per-group combo filters | **DSL filter design** + **engine** filter semantics (D-16 read-side) | §7 |
| Bid/demand semantics | ~~Bid~~ **done 2026-08-10** (F-26); `demand` still needs a spec (D-7) | §5 |
| Hidden info enforcement | **All three** (crypto) | §3; NEXT_STEPS §3 |
| Dual-value cards | **Language design** (no construct) | §1 |
| ~~Turn-order reversal eligibility~~ | **Done 2026-08-10** (D-12, F-24) | §4 |
| `not (X)` / multi-condition ergonomics | **Parser/grammar** (P-8) | §6 |
| `for <players>` participation | **Parser/IR** (P-1) | §4 |
| ~~Elimination-game turn flow~~ | **Done 2026-08-10** — self-wrapping `next`/`cycle`, ineligible-player skip, stage auto-end (F-16/F-17, I-24) | §4 |
| Replay determinism | **Engine** (seeded RNG) | NEXT_STEPS §4 |

Suggested order of work: see [`NEXT_STEPS.md`](./NEXT_STEPS.md) — the numeric
input prompt and per-group combo filters are the two highest-value additions
for opening up new game families (numeric input now exists engine-side;
per-group combo filters remain the top DSL-filter gap).
