---
type: agent_wiki_node
module: crates::engine
scope: [all — design divergences and known bugs]
topics: [divergences, known-bugs, design-gaps, demo-games]
last_validated: 2026-08-11
---

# Engine vs. DSL Design — Divergence Report

> **Purpose:** where the engine's behaviour deviates from the *intended* `.cgdsl`
> design (as documented in `dsl-semantics.md`, `dsl-completeness.md`, the
> original design material under `docs/cardgamedsl/`, and the grammar in
> `crates/front_end/src/grammar.pest`). Each entry has a severity, a repro, and a
> suggested direction.
>
> Everything **not** listed here behaves as documented in
> `dsl-semantics.md` / the authoring guide. This page tracks only the gaps —
> engine-side (D-n), parser-side (P-n), and the parser-dependent work deferred
> by design (§5).

---

## 1. Open divergences (engine-side)

### D-2 — `until` / stage-exit semantics check at entry only
- **Severity:** medium (game-design impact).
- **Behaviour:** the end condition is evaluated at stage *entry*; the body always
  runs once the condition is false. Effects produced mid-body (e.g. "a player
  emptied their hand") are only observed on the next entry — up to one full
  rotation late, and a player may be re-asked after an irreversible event.
- **Repro:** `crazy_eights.cgdsl` / `go_fish.cgdsl` — the "hand empty" exit
  lags; the 30/24-turn caps guarantee termination.
- **Wanted:** a mid-body exit mechanism (`end stage when <bool>`?) or a
  condition re-check after each body action. **Deferred: parser work (see §5).**

### D-3 — `optional` decline runs nothing; refusal cannot be recorded
- **Severity:** medium.
- **Behaviour:** `optional { ... }` decline simply skips to the next edge. There
  is no else-branch, so "stand" (blackjack), "draw instead" (crazy eights), etc.
  must be expressed as *separate* optionals or inverse `if`s — the demo games
  re-ask players each round instead of tracking their choice.
- **Wanted:** `optional { ... } else { ... }` (grammar + IR + engine), or a
  declarative per-player "acted this round" flag. **Deferred: parser work (see §5).**

### D-6 — card status *behaviour* (face up/down/private) is unimplemented
- **Severity:** medium (feature gap).
- **Status:** the data model slot exists (`GameData::card_statuses`, parallel to
  `cards`, default `FaceUp`); `FlipAction` is still a no-op and
  `MoveType::Place`/tokens remain stubs.
- **Wanted:** implement together with card encryption — flipping a card is
  (de)encrypting its face; privacy is the foundation for P2P play.

### D-7 — `demand` semantics are undefined (silent no-op)
- **Severity:** low (no spec to violate, but games cannot be written).
- **Behaviour:** `DemandAction`, `DemandMemoryAction` parse and lower, then do
  nothing. (`bid` is defined — see `dsl-semantics.md` §3.7.)
- **Wanted:** a written semantic spec first, then an implementation.

### D-9 — `GameSuccessful` / `GameFail` ≡ `Game`
- **Severity:** low.
- **Behaviour:** `out of game successful` / `out of game fail` and the
  `OutOfPlayer` bool treat both exactly like `out of game`. There is no notion
  of a success/fail outcome.
- **Wanted:** a game-outcome flag, or remove the keywords from the grammar.

### D-16 — combo *read-side* evaluation over-approximates
- **Severity:** medium (game-design impact for Rummy-style games).
- **Behaviour:** lay-down moves (`move <combo> in <pile> …`) prompt the player
  and **validate** the chosen set against the combo's filter (re-prompting on
  mismatch; 0 cards = skip), which makes classic constraints work —
  `combo Set where (same Rank and size >= 3)` correctly **rejects** a two-Ace
  selection. But the *read-side* (`size(cards Set in Hand)`) still counts *any*
  duplicated rank (pairs included) and applies `size` to the whole pile — the
  filter semantics themselves are unchanged.
- **Wanted (read-side):** per-group filters (e.g. a `same Rank size >= 3`
  atom, or `adjacent` restricted to chains of exactly N).
- Verified by `tests/behavior_test.rs::combo_laydown_prompts_and_validates`
  and `combo_until_stage_loops_until_hand_cleared`.

---

## 2. Parser / lowering divergences (front_end-side)

- **P-1 (`for X` is dropped).** The `stage ... for <player>` clause is
  parsed into the AST but never lowered (`build_seq_stage` ignores `stage.player`).
  `for current` ≡ `for all` ≡ `for P:P2`; all players are marked in-stage
  (`ensure_stage_entered`). Fix: carry the participant collection into the IR
  payload and gate stage entry on it.
- **P-2 (SimStage ≡ SeqStage).** `build_sim_stage` is an identical copy of the
  sequential builder (explicit TODO). No simultaneous execution exists.
- **P-4 / P-5 (memory write rules have no `of <owner>`).** The read rules
  accept `of <owner>`; the write rules (`M is X`, `reset M`) do not. The engine
  **bridges** the gap (declared-owner → current-player resolution, see
  `dsl-semantics.md` §3.6), but the grammar could make the owner mandatory and
  remove the bridge.
- **P-6 (`create` keyword unused).** `kw_create` exists but no rule uses it.
- **P-8 (PEG parens quirks).** `not (X)`, `(X)` and `case (A > B)`/`until (A > B)`
  with complex operands do not parse (see `dsl-completeness.md` §8). These are
  grammar-shape issues that silently steer authors away from valid programs.

---

## 3. Demo game index (handoff)

| Game | File | Interactivity | Engine features exercised | Known simplifications |
|---|---|---|---|---|
| Blackjack | `test_games/blackjack.cgdsl` | optionals per turn | optionals, points, `sum`, `size(playersin)`, `cycle to next` (unguarded — self-wrap + skip mode), `winner is highest score` | Ace = 11 only; standing re-asks each round (D-3); dealer is a tableau, not a player |
| War | `test_games/war.cgdsl` | none (automatic) | `until (A or B)` exit, point-map comparison, `if` chains, moves, scoring | ties discard both cards (no war redeal) |
| Crazy Eights | `test_games/crazy_eights.cgdsl` | choose-card + choose-player per turn | `move 1` (pick-one play), `Hand of any` (ChoosePlayer), `deal N` draws, `until (A or B) or N times`, lowest-score winner | no match constraint on plays; draw may be gifted to any player (house rule) |
| Five-Card Draw | `test_games/five_card_draw.cgdsl` | choose-card per draw round | `Hand of all` fan-out, `move 1` discard, `where same Rank/Suit` filters, score bonuses | draw-1 variant; additive scoring (no straights/full houses) |
| Go Fish | `test_games/go_fish.cgdsl` | 13-way `choose` per turn | `choose` with 13 options, `where Rank is "X" of next` (owner-aware filter), draw-on-miss, memory-free scoring | ask the *next* player only; no books; hand-size scoring |

All five run end-to-end under `tests/demo_games_test.rs` (structural assertions:
card conservation, completion, winner existence). TUI: `just tui crates/engine/test_games/<name>.cgdsl`.

---

## 4. Deferred — parser-dependent fixes (not implemented, by design)

These fixes require touching `front_end` (grammar, parser, or IR builder).
They are deliberately **not** implemented in this handoff; the engine-side
prerequisites are noted so a later project can pick them up:

- **`optional { ... } else { ... }` (D-3).** Required work:
  1. `grammar.pest`: `optional_rule` gains an optional `kw_else ~ "{" ~ flow_component+ ~ "}"` tail.
  2. `parser.rs`: `OptionalRule` AST carries an `else_flows` field.
  3. `ir.rs build_optional_rule`: emit a third edge (body / else-body / exit); the engine's
     `Optional` payload arm already dispatches on `input.idx()` (accept=0, decline=1), so the
     engine change is confined to the IR shape (decline must lead to the else-body, whose exit
     merges with the accept path).
  4. Engine: no `Payload` change needed; only tests for the new edge layout.
- **`end stage when <bool>` mid-body exit (D-2).** Required work:
  1. Grammar: a new flow rule (e.g. `end_stage_when = { kw_end ~ kw_stage ~ kw_when ~ "(" ~ bool_expr ~ ")" }`).
  2. IR: emit a `Payload::EndCondition { negated: true, stage }` edge to the stage's exit — the
     engine's `EndCondition` arm (incl. `leave_stage`) already implements exactly this; the IR
     builder only ever emits `EndCondition` at stage *entry* today.
  3. Engine: no change required.
- **`for <players>` stage participation (P-1) / SimStage (P-2).** Required work:
  1. IR: carry `stage.player`/`stage.players` into a new payload or a stage-region marker.
  2. Engine: gate `ensure_stage_entered` on the participant collection (the `in_stage` map
     already models participation); SimStage additionally needs per-player sub-FSMs in the IR.
  3. Validation: reject `for` clauses referencing unknown players at parse time.
- **`any` / ranges in pure int slots (`score any to …`, `M is any`).**
  Engine-side `InputType::Number` exists (behind `bid … on <memory> of <owner>`
  and the `deal` count prompt); the grammar surface for prompting in *arbitrary*
  int expressions does not (`quantity`-slot `any` means choose-*cards* /
  choose-*count*). "Pick exactly one card" is already expressible as
  `move 1 from …`.

---

## 5. Audit trail

A short history of the engine's evolution, for orientation only (current
behaviour is documented in `dsl-semantics.md`):

- **Initial handoff:** parser + IR builder (`front_end`) with a working
  interpreter; `action::execute` was panicky, collection-memory aggregation was
  `todo!()`, and several quantifier paths were unimplemented.
- **2026-08-09 audit:** the panic table was removed (`action::execute` became
  fallible, `EngineError`); combo evaluation became group-wise; empty
  `where`-sets stopped resolving to location 0; `choose` split on `or`;
  `not <combo> in X empty` bound correctly; `owner of <memory>` key order
  fixed; the five demo games were hardened.
- **2026-08-10 (ergonomics pass):** ineligible-player skip + stage auto-end
  (I-24); self-wrapping `next`/`previous`/`cycle`; memory initial values,
  typed init, full reset, evaluated collection writes, declared-owner write
  resolution; winner-extrema fixes; team-owned locations/memories (one per
team, team-keyed);
  the numeric input prompt (`InputType::Number`) behind `bid … on <memory>`.
- **2026-08-10 (verb semantics):** the verb carries the quantity semantics —
  `deal` = automatic from the top (with a count prompt for `any`/ranges),
  `move`/`exchange` = the player picks (exact-N prompt for literals),
  positional sources always automatic. The `>= 1 and <= 1` idiom disappeared
  from the demo games.
- **2026-08-10 (winner set):** `GameData::winner_names()` (winners = players
  still in game), `end game with winner X` now eliminates everyone else, and
  the winner set is logged by the TUI, the trace file, and `cgdsl-play`.
- **Test counts:** `cargo test -p cgdsl-engine` — 527 tests green (439 lib +
  5 cgdsl-play + 9 engine-tui + 74 integration); `clippy -p cgdsl-engine
  --all-targets --no-deps -D warnings` and `fmt --check` clean. Workspace
  caveat: `cargo clippy --workspace` still fails on pre-existing `front_end`
  library lints and a `code_gen` lint outside this crate.
