---
type: agent_wiki_node
module: crates::engine
scope: [all — design divergences and known bugs]
topics: [divergences, known-bugs, design-gaps, demo-games, audit]
last_validated: 2026-08-09
---

# Engine vs. DSL Design — Divergence Report

> **Purpose:** where the engine's behaviour deviates from the *intended* `.cgdsl`
> design (as documented in `dsl-semantics.md`, `dsl-completeness.md`, the original
> design material under `docs/cardgamedsl/`, and the grammar in
> `crates/front_end/src/grammar.pest`). Each entry has a severity, a repro, and a
> suggested direction. Entries marked **FIXED** were corrected on 2026-08-09 with
> regression tests.
>
> Note on the "wanted design": the original thesis document is not in this
> repository. The design authority used here is `docs/cardgamedsl/` (parser-side
> design records) + `grammar.pest` + `docs/dsl-semantics.md`. Where the grammar
> itself took liberties (e.g. the `create` keyword is unused), that is flagged as
> parser divergence rather than engine divergence.

---

## 1. Fixed during the 2026-08-09 audit

| ID | Bug | Fix | Regression test |
|---|---|---|---|
| F-1 | `set_memory` incremented `Int` memories by 1 instead of assigning (I-9) | Now assigns the evaluated `MemoryValue` (`game_data.rs:301`) | `game_data_tests` / `memory_test` suite |
| F-2 | `execute_cardset_move` dest guard used `>` not `>=` — `dest_loc_idx == len` panicked with an index error | Guard now `>=` with a clear message (`action.rs:538`) | `action_tests` |
| F-3 | `OwnerOfMemory` looked up `"{memory}_{owner}"` instead of `"{owner}_{memory}"` — always failed or hit the wrong slot | Key order corrected (`query/player.rs:93`) | `player_tests::eval_player_aggregate_owner_of_memory_{min,max}` |
| F-4 | `GroupOwner` with a `where`-filter evaluated the base location against the *current* player, then filtered by owner — `Hand of P:P2 where Rank is "X"` returned nothing whenever current ≠ P2 | Base location resolved against the owner (`query/cardset.rs`, `owner_base_location`) | `go_fish` demo game + `cardset_tests` |
| F-5 | `ShuffleAction` replaced the whole location with the evaluated set — `shuffle top 3 of Deck` discarded the rest of the deck | Selected cards shuffled in place; unselected untouched (`action.rs:192`) | `shuffle_test` |
| F-6 | Three `debug` tests hard-coded Unix `/tmp/` paths — failed on Windows | `std::env::temp_dir()` (`debug/tests.rs`) | suite is green on Windows |
| F-7 | `blackjack_runs_end_to_end` hung forever (input closure returned a wrong `player_id`; the controller's validation re-prompted infinitely, I-15/I-23) | Closure tracks the current player via `event_sender` | `flow_test` |

## 2. Open divergences (engine-side)

### D-1 — `cycle to next` panics when no eligible *other* player exists
- **Severity:** high (crash instead of recoverable error).
- **Cause:** `GameData::resolve_turn` never considers the current player (I-13).
  With exactly one eligible player left, `cycle to next` → `eval_player(Next)` →
  `"No next player available"` → `panic!` at `action.rs:309`.
- **Repro:** run `blackjack.cgdsl` with a script that busts two of three players
  (the old fixture did this; the demo game now guards with
  `if (size(playersin) >= 2)`).
- **Wanted:** a recoverable `StepResult::Error` (or a no-op cycle), never a panic;
  games should not need guards.
- **Direction:** move the cycle resolution into the interpreter (which can return
  `StepResult::Error`) or make `next_player` a fallible `GameData` method and
  surface the error through `action::execute`'s return.

### D-2 — `until` / stage-exit semantics check at entry only
- **Severity:** medium (game-design impact).
- **Behaviour:** the end condition is evaluated at stage *entry*; the body always
  runs once the condition is false. Effects produced mid-body (e.g. "a player
  emptied their hand") are only observed on the next entry — up to one full
  rotation late, and a player may be re-asked after an irreversible event.
- **Repro:** `crazy_eights.cgdsl` / `go_fish.cgdsl` — the "hand empty" exit
  lags; the 30/24-turn caps guarantee termination.
- **Wanted:** a mid-body exit mechanism (`end stage when <bool>`?) or
  condition re-check after each body action.

### D-3 — `optional` decline runs nothing; refusal cannot be recorded
- **Severity:** medium.
- **Behaviour:** `optional { ... }` decline simply skips to the next edge. There
  is no else-branch, so "stand" (blackjack), "draw instead" (crazy eights), etc.
  must be expressed as *separate* optionals or inverse `if`s — the demo games
  re-ask players each round instead of tracking their choice.
- **Wanted:** `optional { ... } else { ... }` (grammar + IR + engine), or a
  declarative per-player "acted this round" flag.

### D-4 — collection-memory aggregation is unimplemented (4 `todo!()` panics)
- **Severity:** medium.
- **Sites:** `IntCollection::AggregateMemory` (`query/int.rs:170`),
  `TeamCollection::AggregateMemory` (`query/int.rs:257`),
  `StringCollection::AggregateMemory` (`query/string.rs:55`),
  and `PlayerCollection::Aggregate` (`query/player.rs:240`, reachable via
  `end game with winner(for all ...)` / `OutOfPlayer` with quantifiers).
- **Repro:** any DSL using `sum((&IC:M of all))`, `size(playersin)`-style
  aggregates over multi-owner memories, or a quantifier in `out of`.
- **Wanted:** multi-owner iteration in the evaluator (iterate the owner
  collection, read each prefixed slot, aggregate).

### D-5 — combo per-card matching of `same` / `distinct` is wrong
- **Severity:** medium.
- **Cause:** `card_matches_filter` implements `Same` as "some card in the whole
  game with the same key-value is *this card*" (always true) and `Distinct` as
  "another card shares the value" (inverted). `where same Rank` on a *group*
  (via `apply_filter`) is correct; only the combo/`not combo` per-card path is
  affected. `Size` in per-card matching always compares against 1.
- **Repro:** `combo Pair where same Rank` matches every card that has a Rank.
- **Wanted:** group-context-aware per-card matching (pass the group, or rework
  combo evaluation to run over the group like `apply_filter`).

### D-6 — status (face up/down/private) is parsed and ignored
- **Severity:** medium (feature gap).
- **Cause:** no card-status field in the data model; `FlipAction` is a silent
  no-op; `MoveType::Place` and tokens likewise.
- **Wanted:** a per-card status map in `GameData`, `FlipAction` execution, and
  status-aware rendering (privacy is the foundation for P2P play).

### D-7 — bidding / demand semantics are undefined (silent no-ops)
- **Severity:** low (no spec to violate, but games cannot be written).
- **Behaviour:** `BidAction`, `BidMemoryAction`, `DemandAction`,
  `DemandMemoryAction` parse and lower, then do nothing.
- **Wanted:** a written semantic spec first (what does a bid do to game state?),
  then an implementation.

### D-8 — `resolve_quantity` evaluates against an empty `GameData`
- **Severity:** low.
- **Behaviour:** `deal (<runtime int expr>) from X ...` and range quantities
  evaluate their int exprs against `GameData::new()`; memory/stage-backed exprs
  fail and silently fall back to `1` (or "accept any count" for ranges).
- **Wanted:** evaluate quantities against the live state (the
  `validate_int_range` re-prompt path already handles live ranges at resume).

### D-9 — `GameSuccessful` / `GameFail` ≡ `Game`
- **Severity:** low.
- **Behaviour:** `out of game successful` / `out of game fail` and the
  `OutOfPlayer` bool treat both exactly like `out of game`. There is no notion
  of a success/fail outcome.
- **Wanted:** a game-outcome flag, or remove the keywords from the grammar.

### D-10 — `winner is highest position` uses turn-order index
- **Severity:** low (ambiguous design).
- **Behaviour:** position = 0-based index in `turn_order`; players not in
  `turn_order` score `usize::MAX` (so `lowest position` can be won by a player
  who is *not* in the turn order). Documented in `developer-notes.md` §1.3.
- **Wanted:** clarify the intended meaning (table position? turn position?) and
  pin it with a test.

### D-11 — empty filter results resolve to location 0 (I-14)
- **Severity:** low.
- **Behaviour:** `X where <no match>` returns `(0, [])`; using such a set as a
  move destination sends cards to the first location.
- **Wanted:** a `Result`-level "empty set" marker; never silently use location 0.

### D-12 — `Previous` ignores in-game/stage eligibility
- **Severity:** low.
- **Behaviour:** `previous` returns the previous turn-order entry even if that
  player is out; `next` skips ineligible players (asymmetric).
- **Wanted:** consistent eligibility semantics for both directions.

### D-13 — `WinnerWith` memory extrema clamps negatives, misses non-Ints
- **Severity:** low.
- **Behaviour:** memory-based winner extrema treat missing/negative/non-Int
  memories as 0.
- **Wanted:** explicit behaviour for missing memories (error vs. skip).

## 3. Parser / lowering divergences (front_end-side)

- **P-1 (`for X` is dropped, B-1).** The `stage ... for <player>` clause is
  parsed into the AST but never lowered (`build_seq_stage` ignores `stage.player`).
  `for current` ≡ `for all` ≡ `for P:P2`; all players are marked in-stage
  (`ensure_stage_entered`). Fix: carry the participant collection into the IR
  payload and gate stage entry on it.
- **P-2 (SimStage ≡ SeqStage, B-3).** `build_sim_stage` is an identical copy
  (`ir.rs:654`, explicit TODO). No simultaneous execution exists.
- **P-3 (setup-`Any` rejected, I-20).** `location X on any`, `turnorder any`,
  etc. error with "quantifier 'any' is not supported in setup rules". `All`
  works. Wanted: either implement setup-`Any` (prompt before setup) or document
  `Any` as play-phase-only in the language reference.
- **P-4 (bare memory refs parse but error).** `&I:M` without `of <owner>` parses;
  the engine rejects it ("memory access requires an explicit owner"). The
  grammar should make the owner mandatory (it already does in the `create`
  rules — only the read rules have the optional form).
- **P-5 (`SetMemory`/`ResetMemory` lack `of owner`).** The write rules have no
  owner clause; the engine bridges by prefixing the *current player* — which is
  wrong for "set P2's memory" and a silent trap when `current` is `None`
  (panic). Fix in the grammar (`M is X of P:P2`), then drop the bridge.
- **P-6 (`create` keyword unused).** `kw_create` exists but no rule uses it.
- **P-7 (team-owned locations/memories parse but error).** `location X on T:T1`
  and `memory M on T:T1` are rejected at runtime ("team-owned locations are not
  in the data model"). Either implement team ownership or reject in validation.
- **P-8 (PEG parens quirks).** `not (X)`, `(X)` and `case (A > B)`/`until (A > B)`
  with complex operands do not parse (see `dsl-completeness.md` §8). These are
  grammar-shape issues that silently steer authors away from valid programs.

## 4. Demo game index (handoff)

| Game | File | Interactivity | Engine features exercised | Known simplifications |
|---|---|---|---|---|
| Blackjack | `test_games/blackjack.cgdsl` | optionals per turn | optionals, points, `sum`, `size(playersin)`, guarded `cycle`, `winner is highest score` | Ace = 11 only; standing re-asks each round (D-3); dealer is a tableau, not a player |
| War | `test_games/war.cgdsl` | none (automatic) | `until (A or B)` exit, point-map comparison, `if` chains, moves, scoring | ties discard both cards (no war redeal) |
| Crazy Eights | `test_games/crazy_eights.cgdsl` | choose-card + choose-player per turn | `deal any` (ChooseCards), `Hand of any` (ChoosePlayer), `until (A or B) or N times`, lowest-score winner | no match constraint on plays; draw may be gifted to any player (house rule) |
| Five-Card Draw | `test_games/five_card_draw.cgdsl` | choose-card per draw round | `Hand of all` fan-out, `deal any` discard, `where same Rank/Suit` filters, score bonuses | draw-1 variant; additive scoring (no straights/full houses) |
| Go Fish | `test_games/go_fish.cgdsl` | 13-way `choose` per turn | `choose` with 13 options, `where Rank is "X" of next` (owner-aware filter), draw-on-miss, memory-free scoring | ask the *next* player only; no books; hand-size scoring |

All five run end-to-end under `tests/demo_games_test.rs` (structural assertions:
card conservation, completion, winner existence). TUI: `just tui crates/engine/test_games/<name>.cgdsl`.

## 5. Audit trail

- Audit performed 2026-08-09: **406 unit + 57 integration tests green**;
  `cargo clippy -p cgdsl-engine --all-targets --no-deps -- -D warnings` clean;
  `cargo fmt -p cgdsl-engine -- --check` clean. The engine package previously
  did not meet the clippy bar (12 pre-existing lints fixed the same day,
  incl. `engine-tui`).
- **Workspace-level clippy caveat:** `cargo clippy --workspace --all-targets
  -- -D warnings` additionally fails on *pre-existing* lints in
  `front_end/build.rs` (outside `crates/engine`, not touched by this audit).
- Doc-drift corrected the same day: I-9 semantics, `execute_cardset_move` guard,
  I-18 synthetic-key naming (`Table_`-prefixed), test counts, `rand` dependency,
  `mcg-cli` location (it is a `native_mcg` binary, not a workspace crate),
  `docs/README.md` module map, and the `cgdsl-authoring-guide.md` blackjack
  walkthrough (which previously taught the unguarded `cycle to next` pattern).
