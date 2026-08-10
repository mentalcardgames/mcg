---
type: agent_wiki_node
module: crates::engine
scope: [all — design divergences and known bugs]
topics: [divergences, known-bugs, design-gaps, demo-games, audit]
last_validated: 2026-08-10
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

## 1. Fixed during the 2026-08 audit

| ID | Bug | Fix | Regression test |
|---|---|---|---|
| F-1 | `set_memory` incremented `Int` memories by 1 instead of assigning (I-9) | Now assigns the evaluated `MemoryValue` (`game_data.rs:301`) | `game_data_tests` / `memory_test` suite |
| F-2 | `execute_cardset_move` dest guard used `>` not `>=` — `dest_loc_idx == len` panicked with an index error | Guard now `>=` with a clear message (`action.rs:538`) | `action_tests` |
| F-3 | `OwnerOfMemory` looked up `"{memory}_{owner}"` instead of `"{owner}_{memory}"` — always failed or hit the wrong slot | Key order corrected (`query/player.rs:93`) | `player_tests::eval_player_aggregate_owner_of_memory_{min,max}` |
| F-4 | `GroupOwner` with a `where`-filter evaluated the base location against the *current* player, then filtered by owner — `Hand of P:P2 where Rank is "X"` returned nothing whenever current ≠ P2 | Base location resolved against the owner (`query/cardset.rs`, `owner_base_location`) | `go_fish` demo game + `cardset_tests` |
| F-5 | `ShuffleAction` replaced the whole location with the evaluated set — `shuffle top 3 of Deck` discarded the rest of the pile | Selected cards shuffled in place; unselected untouched (`action.rs:192`) | `shuffle_test` |
| F-6 | Three `debug` tests hard-coded Unix `/tmp/` paths — failed on Windows | `std::env::temp_dir()` (`debug/tests.rs`) | suite is green on Windows |
| F-7 | `blackjack_runs_end_to_end` hung forever (input closure returned a wrong `player_id`; the controller's validation re-prompted infinitely, I-15/I-23) | Closure tracks the current player via `event_sender` | `flow_test` |
| F-8 | **Panic table removed — `action::execute` is fallible.** `cycle to next` with no eligible *other* player (D-1), `SetMemory`/`ResetMemory` without a current player, `CycleAction` eval/player/turn-order failures, `CreateLocation`/`CreateMemory` owner resolution, `CreateCardOnLocation`, `CreatePointMap`, `Score`/`ScoreMemory` int eval, and all `execute_cardset_move` failure modes now return `StepResult::Error` instead of panicking. `Interpreter::execute_edge` returns `Result<(), EngineError>`; `ShuffleAction` eval failures are errors (were `eprintln!` + continue). | `action.rs` (all arms), `interpreter/mod.rs` | `action_tests` (7 former `#[should_panic]` pins converted) + `errors_cycle_no_next.cgdsl` + `errors_set_memory_no_current.cgdsl` |
| F-9 | **`resolve_players` / `resolve_player_collection` are fallible.** Eval failures and unknown literal player names return `Err`; the `Aggregate` arm (previously `todo!()`) and the `AggregateMemory`/`Memory` arms (previously silent `vec![]`) are implemented. | `query/player.rs` | `player_tests` (converted panic pins, new aggregation tests) |
| F-10 | **Combo per-card matching of `same`/`distinct`/`size` was wrong** (D-5): `Same` matched every card with the key, `Distinct` was inverted, `Size` always compared 1. Combos are now evaluated group-wise (like `where`); the broken per-card matcher is deleted. | `query/cardset.rs` | `fix_combo_same_rank.cgdsl` (pair = 2, not 3) |
| F-11 | **Empty `where`-filtered sets resolved to location 0** (D-11): a move destination like `Second where Rank is "Ghost"` with no matches sent cards to the first location. `eval_group` now reports the base location of the groupable; `execute_cardset_move` no-ops on an empty source. | `query/cardset.rs`, `action.rs` | `fix_empty_where_dest.cgdsl` |
| F-12 | **`resolve_quantity` evaluated against an empty `GameData`** (D-8): runtime-backed quantities silently fell back to 1 (or "accept any" for ranges). It now evaluates against the live state and propagates errors. | `query/int.rs` | `int_tests` (rewritten runtime tests) |
| F-13 | **Collection-memory aggregation unimplemented** (D-4): the four `todo!()` arms (`IntCollection`/`TeamCollection`/`StringCollection` `AggregateMemory`, `PlayerCollection::Aggregate`) now aggregate the slot across every owner of `multi`. | `query/int.rs`, `query/string.rs`, `query/player.rs` | `int_tests`, `string_tests`, `player_tests` |
| F-14 | **`choose` did not split on `or`** (parser bug, `front_end`): `choose { A B or C D }` produced four single-component options instead of two options of `[A, B]`/`[C, D]`. The AST now carries `options: Vec<Vec<FlowComponent>>`, the parser groups on `or`, the IR builder chains each option's sequence, the formatter renders groups, and the arbitrary generator produces non-empty groups. | `front_end`: `grammar.pest` (kw_or), `parser.rs`, `ast.rs`, `arbitrary.rs`, `ir.rs`, `fmt_ast.rs` | `front_end` `choice_rule_splits_options_on_or` / `choice_rule_single_option_no_or`; `go_fish.cgdsl` now behaves as authored (13 arms of deal+draw) |
| F-15 | **`not <combo> in X empty` bound `not` to the combo** (parser ambiguity, `front_end`): `not Book in Hand of current empty` parsed as `CardSetEmpty(NotCombo(Book, Hand))` — "the cards not matching Book are empty" — almost never true, so guards like `if (not Book in Hand of current empty)` silently never fired. Root cause: `bool_expr` tried `card_set_empty`/`card_set_not_empty` before `bool_expr_unary`, and a combo group may itself start with `not`. Fix: `bool_expr_unary` moved before the cardset-empty rules, so a leading `not` binds to the boolean (`Unary(Not, CardSetEmpty(Combo-in-X))`); `Book in Hand of current not empty` remains `CardSetNotEmpty`. Also renamed the trace label `else=` → `body=` (it reports whether the if-*body* edge was taken). | `front_end`: `grammar.pest` (bool_expr rule order) | `front_end` `not_combo_empty_parses_as_boolean_negation` / `combo_not_empty_parses_as_card_set_not_empty`; `go_fish.cgdsl` book guard |

## 1b. Partially fixed

- **Card status (D-6, data model only).** `GameData` now carries
  `card_statuses: Vec<CardStatus>` (parallel to `cards`, default `FaceUp`,
  accessors `card_status` / `set_card_status`). **Behaviour is intentionally
  deferred**: `FlipAction` remains a no-op with a comment stating it should be
  implemented together with card encryption — flipping a card is
  (de)encrypting its face. The engine never reads or writes the slot today.

## 2. Open divergences (engine-side)

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
- **Status:** the data model slot exists (§1b); `FlipAction` is still a no-op and
  `MoveType::Place`/tokens remain stubs.
- **Wanted:** implement together with card encryption — flipping a card is
  (de)encrypting its face; privacy is the foundation for P2P play.

### D-7 — bidding / demand semantics are undefined (silent no-ops)
- **Severity:** low (no spec to violate, but games cannot be written).
- **Behaviour:** `BidAction`, `BidMemoryAction`, `DemandAction`,
  `DemandMemoryAction` parse and lower, then do nothing.
- **Wanted:** a written semantic spec first (what does a bid do to game state?),
  then an implementation.

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

### D-14 — `SetMemory`/`ResetMemory` owner bridging (grammar gap)
- **Severity:** medium.
- **Behaviour:** the write rules have no `of <owner>` clause; the engine keys
  to the *current player* and errors when there is none (recoverable since F-8).
  "Set P2's memory" is inexpressible; a write aimed at a non-current player is a
  silent trap.
- **Wanted:** grammar support (`M is X of P:P2`), then drop the bridge.
  **Deferred: parser work (see §5).**

### D-15 — location-0 fallback remains for `CardSet::Memory` (I-14)
- **Severity:** low.
- **Behaviour:** a *memory-backed* cardset whose first card cannot be found in
  any location still returns `(0, card_ids)`; the `where`-set case is fixed
  (F-11) but the memory case keeps the sentinel.
- **Wanted:** an explicit "empty set" marker at the `eval_cardset` boundary.

### D-16 — combo *read-side* evaluation over-approximates; moves now prompt + validate
- **Severity:** medium (game-design impact for Rummy-style games).
- **Lay-down moves (fixed 2026-08-09):** a move whose source is a combo group
  (`move Set in Hand of current private to Table`) is now a **validated
  prompt** (quantifier site `ComboSource`): the player chooses cards from the
  *whole* pile; the engine validates the choice against the combo's filter
  and re-prompts on mismatch. This makes the classic constraints work:
  `combo Set where (same Rank and size >= 3)` correctly **rejects** a two-Ace
  selection (the `size` filter is now applied to the *player's selection*).
  The prompt accepts **0 cards as a valid no-op ("skip")**, so a prompt that
  over-fires (read-side, below) can always be dismissed — and games can offer
  "lay down or pass" freely (Go Fish's book mechanic uses this). For "lay
  down everything", pair the move with a stage loop:
```
stage Laydown for current until Set in Hand empty 
	{ 
	move Set in Hand of
	current private to Table 
	}
```
   — `until <combo> in <pile> empty` is a valid end condition (a combo group is a cardset; `card_set_empty` applies).
- **Read-side remains over-approximating:** `size(cards Set in Hand)` still
  counts *any* duplicated rank (pairs included) and applies `size` to the
  whole pile — the filter semantics themselves are unchanged.
- **Wanted (read-side):** per-group filters (e.g. a `same Rank size >= 3`
  atom, or `adjacent` restricted to chains of exactly N).
- Verified by `tests/behavior_test.rs::combo_laydown_prompts_and_validates`
  (invalid selection rejected, then re-prompted) and
  `combo_until_stage_loops_until_hand_cleared` (`test_games/behavior_combo_{laydown,until}.cgdsl`).

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
- **P-5 (`SetMemory`/`ResetMemory` lack `of owner`).** See D-14.
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

## 5. Deferred — parser-dependent fixes (not implemented, by design)

These three fixes require touching `front_end` (grammar, parser, or IR builder).
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

## 6. Audit trail

- Audit performed 2026-08-09 (second pass): **414 unit + 63 integration tests green**
  (the panic-table removal converted 10 `#[should_panic]` pins into `Err`-assertion tests
  and added 4 new fixtures); `cargo clippy -p cgdsl-engine --all-targets --no-deps -- -D warnings`
  clean; `cargo fmt -p cgdsl-engine -- --check` clean.
- **Third pass (same day):** `choose` splits on `or` (F-14, front_end fix) — `front_end`
  now has 20 passing tests including two dedicated `choice_rule` regression tests and the
  format↔parse proptests.
- **Fourth pass (same day):** behavioral fixtures (`tests/behavior_test.rs`, 6 tests over
  5 deterministic non-shuffled fixtures) verify exact rule outcomes. They caught a real
  DSL-authoring bug in `go_fish.cgdsl`: each ask option dealt *before* checking emptiness,
  so a successful ask also drew a card (draw-on-hit). Fixed by inverting the order
  (check first, deal second). Total: 414 unit + 74 integration tests.
- **Workspace-level clippy caveat (updated):** `cargo clippy --workspace --all-targets
  -- -D warnings` still fails on *pre-existing* debt in the `front_end` **library**
  (~200 `redundant field names` / doc-comment style lints across `parser.rs`, `ast.rs`,
  `symbols.rs`, …). The two `front_end/build.rs` collapsible-`if` lints that also blocked
  the check were fixed the same day (mechanical). The engine crate itself meets the bar.
- Doc-drift corrected the same day: I-9 semantics, `execute_cardset_move` guard,
  I-18 synthetic-key naming (`Table_`-prefixed), test counts, `rand` dependency,
  `mcg-cli` location (it is a `native_mcg` binary, not a workspace crate),
  `docs/README.md` module map, and the `cgdsl-authoring-guide.md` blackjack
  walkthrough (which previously taught the unguarded `cycle to next` pattern).
