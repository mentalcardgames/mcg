---
type: agent_wiki_node
module: crates::engine
scope: [all — testing strategy and conventions]
topics: [testing, fixtures, coverage, conventions, tdd, harness]
associated_files:
  - crates/engine/Cargo.toml
  - crates/engine/src/lib.rs
  - crates/engine/src/game_data_tests.rs
  - crates/engine/src/quantifier_tests.rs
  - crates/engine/src/controller/tests.rs
  - crates/engine/src/interpreter/tests.rs
  - crates/engine/src/debug/tests.rs
  - crates/engine/tests/quantifier_test.rs
  - crates/engine/test_games/
last_validated: 2026-08-13
---

# Testing Strategy

> **Scope:** how to write, organize, and extend tests for `crates::engine`. For *what* the
> engine does, see [`README.md`](./README.md); for guardrails tests must respect, see
> [`invariants.md`](./invariants.md).

The engine is a **single-threaded, fully synchronous, deterministic** FSM interpreter
(see [`interfaces.md`](./interfaces.md) §6). There is no I/O concealed inside the library
(except the optional trace log opened by [`controller/mod.rs`](../src/controller/mod.rs)),
no `tokio`, no threads. This makes the **entire crate testable with plain `cargo test`** —
no mock servers, no time control, noDeterministic-runtime tricks. Every test in the suite
runs in microseconds.

---

## 1. Philosophy

1. **Determinism is the test substrate.** A `GameData` plus an `Ir<LoweredPayLoad>` plus a
   fixed input sequence yields exactly one terminal state. Tests exploit this by
   constructing one of three things and asserting on the result:

| Layer               | Construct directly                        | Assert on                                                           |
| ------------------- | ----------------------------------------- | ------------------------------------------------------------------- |
| Unit Tests          | AST/IR enums, `GameData::new()`           | return value of `Evaluator::eval_*`, `quantifier::*`, `GameData::*` |
| Interpreter-level   | a small hand-built `Ir` and `Interpreter` | `StepResult` variants, `current_state`, side-effects on `game_data` |
| Fixture integration | a `.cgdsl` file under `test_games/`       | terminal `GameData`, emitted `TraceEntry`s, snapshots               |

3. **Prefer fixtures over hand-built IR for engine arms.** Hand-building `Edge`s and
   `Payload`s is verbose and couples tests to the IR representation. For any behavior that
   crosses an interpreter `step()` boundary (actions, conditions, stages, quantifier
   dispatch), write a `.cgdsl` fixture and drive it through `run_game`. Reserve
   hand-built-IR tests for the interpreter's own dispatch logic (Choice/Optional prompt
   shapes, edge-count errors, etc.) — see `src/interpreter/tests.rs` for the pattern.

4. **Pin invariants, not implementations.** Every guardrail I-1..I-25 in
   [`invariants.md`](./invariants.md) must have at least one regression test. When
   refactoring, the test should fail loudly before the invariant is silently violated.

5. **TDD for new engine arms.** When adding a new `ActionRule` / `SetUpRule` / `Payload`
   variant or a new `Evaluator` method, write the failing test first. See §6.

---

## 2. Test Layout & Discovery

`Cargo.toml` declares **no `[[test]]` targets**; everything relies on Cargo's default
discovery rules:

```
crates/engine/
├── src/
│   ├── game_data.rs            ┐
│   ├── game_data_tests.rs      │  wired via `#[cfg(test)] #[path = "game_data_tests.rs"] mod tests;`
│   ├── quantifier.rs           │  at the bottom of the parent .rs file.
│   ├── quantifier_tests.rs     ┘
│   ├── controller/
│   │   ├── mod.rs
│   │   └── tests.rs             ┐ wired via `#[cfg(test)] #[path = "tests.rs"] mod tests;`
│   ├── interpreter/             │ inside the parent mod.rs.
│   │   ├── mod.rs               │
│   │   └── tests.rs             ┘
│   └── debug/
│       ├── mod.rs
│       └── tests.rs
└── tests/                       ← integration tests (separate compilation units)
    ├── common/mod.rs            │  shared harness (load_game, default_input, test_file,
    │                            │  total_cards, players_in_game, player_location,
    │                            │  table_location, move_traces, CurrentTracker,
    │                            │  accept_everything) — pulled in with `mod common;`
    ├── actions_test.rs          │  one file per feature area, auto-discovered by Cargo:
    ├── behavior_test.rs         │  deterministic rule-outcome fixtures
    ├── demo_games_test.rs       │  the five demo games driven end-to-end
    ├── elimination_test.rs      │  I-24 skip/auto-end, cycle-skip, winner-set traces
    ├── flow_test.rs             │  if/optional/choose/stage flow
    ├── hygiene_test.rs          │  fixture-hygiene guard (no orphaned .cgdsl)
    ├── memory_test.rs           │  memory initial values + numeric `bid` prompt
    ├── quantifier_test.rs       │
    ├── random_play_test.rs      │  random-input monkey tests
    ├── scoring_test.rs          │
    ├── setup_test.rs            │  setup rules incl. setup quantifiers
    ├── teams_test.rs            │  team-owned state
    └── verb_semantics_test.rs   ┘
```

**Naming conventions:**

| Where the code lives | Test file name | Wired how |
|---|---|---|
| Top-level module `foo.rs` (e.g. `game_data.rs`, `quantifier.rs`) | `foo_tests.rs` next to it | `#[cfg(test)] #[path = "foo_tests.rs"] mod tests;` at the bottom of `foo.rs` |
| Submodule `foo/mod.rs` (e.g. `controller/`, `interpreter/`, `debug/`) | `foo/tests.rs` | `#[cfg(test)] #[path = "tests.rs"] mod tests;` somewhere in `foo/mod.rs` |
| Cross-crate / end-to-end | `crates/engine/tests/<area>_test.rs` | auto-discovered by Cargo; one `#[test]` per integration scenario |
| Shared integration harness | `crates/engine/tests/common/mod.rs` | every integration file opens with `mod common;` plus an explicit `use common::{...};` (files in subdirectories of `tests/` are never compiled as test targets themselves) |

**Why the `#[path]` indirection?** It keeps the test file a sibling of the code under
test but keeps `mod.rs` short. It also lets tests reach `pub(super)` items via `use
super::*;` without exposing a `pub` test module.

**Test functions** use plain `#[test]` (no `rstest`, no parametrization crate). Where a
behavior has many input combinations, write a generator helper and a small loop inside one
test (see `alloc_synth_yields_valid_increasing_stateids` in
`src/quantifier_tests.rs:76`).

---

## 3. The Three Test Layers

### 3.1 Pure unit tests — `Evaluator`, `quantifier`, `GameData`

These are the cheapest and fastest. Construct AST enums directly, call a function, assert
on the `Result`. No `Interpreter`, no `run_game`, no I/O.

```rust
// src/quantifier_tests.rs — illustrative
use crate::query::Evaluator;
use front_end::ast::{IntExpr, IntCompare};

#[test]
fn eval_int_literal_returns_value() {
    let gd = crate::game_data::GameData::new();
    assert_eq!(
        Evaluator::eval_int(&IntExpr::Literal { int: 42 }, &gd),
        Ok(42)
    );
}
```

Reach for this layer when:

- Testing a pure function over a `GameData` you can build by hand
  (`add_player`, `add_location`, `add_memory`, …).
- Testing enum classification or pure rewrites (`scan_edge`, `substitute_*`,
  `setup_contains_any`).
- Testing `GameData` mutator semantics
  (`set_player_out` flips `in_game`; `leave_stage` pops the stack).

Do **not** use this layer for `action::execute_*` — those want a real interpreter
context (and a fixture is usually shorter).

### 3.2 Interpreter-level tests — hand-built `Ir`

For the dispatcher itself (`Interpreter::step`), construct a minimal `Ir`, an
`Interpreter`, and drive `step()`. This is the pattern in
`src/interpreter/tests.rs:222` (`step_choice_emits_rich_options_in_needs_input`).

A reusable `state_id(n)` helper appears at the top of that file:

```rust
fn state_id(n: u32) -> StateID { unsafe { std::mem::transmute(n) } }
```

(`StateID` is a `#[repr(transparent)]` wrapper; the `transmute` is a test-only shortcut
that avoids depending on `front_end`'s constructor visibility.)

Use this layer to assert:

- `StepResult` variant selection (`NeedsInput(InputType::Choice { … })` shape)
- `GameOver` vs. `Error("No outgoing edges and not at goal state")` ([I-4](./invariants.md))
- `Condition` vs `EndCondition` inverted edge indexing ([I-3](./invariants.md))
- 2-edge-count violation errors
- `Item::idx()`, `provide_input` LIFO ordering ([I-7](./invariants.md))

### 3.3 Fixture integration tests — `tests/` + `test_games/`

The default for any cross-arm behavior. The shared loader lives in
`tests/common/mod.rs` (every integration file pulls it in with
`mod common;` + `use common::load_game;`):

```rust
// tests/common/mod.rs — do not copy this into a new test file; use `mod common;`
pub fn load_game(name: &str) -> Ir<LoweredPayLoad> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("test_games").join(name);
    let src =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let game = parse_document(&src).unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));
    game.to_lowered_graph()
}
```

Then drive the IR via `run_game` with one of two input sources (§4).

```rust
let gd = run_game(
    load_game("quantifier_deal_all.cgdsl"),
    GameData::new(),
    InputSource::Player(Box::new(|_it: InputType| Input::Choice { idx: 0 })),
    None,  // event_sender
    None,  // trace_sender
).expect("game should complete");
```

Use this layer to assert end-to-end post-conditions: card counts per location, turn
order contents, stage stack, memory values, no synthetic memory slots leaked.

---

## 4. Input Sources

`InputSource` (`crates/engine/src/controller/mod.rs:22`) is the only I/O seam. Two flavors:

### 4.1 `InputSource::TestFile(PathBuf)` — golden replay

One input per line in `test_games/<name>.txt`. Accepted line formats
(`crates/engine/src/controller/mod.rs:203`):

| Line | Yields |
|---|---|
| `y` / `yes` | `Input { player_id: "P1", kind: InputKind::OptionalAccept }` |
| `n` / `no`  | `Input { player_id: "P1", kind: InputKind::OptionalDecline }` |
| `<N>`       | `Input { player_id: "P1", kind: InputKind::Choice { idx: N-1 } }` (1-based) |
| `p <N>`     | `Input { player_id: "P1", kind: InputKind::ChoosePlayer { idx: N-1 } }` (1-based) |
| `c <csv>`   | `Input { player_id: "P1", kind: InputKind::ChooseCards { selected: [..] } }` (1-based, comma-separated) |
| `Name:y` / `Name:<N>` / etc. | Same as above, with `player_id: "Name"` |

Blank lines and lines starting with `#` are skipped. Use TestFile when you want a
fixture-input pair that documents the replay verbatim and is easy to diff on failure.
Example: `test_games/ordering_test.cgdsl` + `ordering_test.txt`, exercised by
`src/controller/tests.rs:105`.

*Lines without a `Name:` prefix default to player `"P1"`. Use the prefix in
multi-player test scenarios where different players submit inputs in sequence.*

### 4.2 `InputSource::Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)` — programmatic

A closure that receives the full `InputType` (Choice/Optional/ChoosePlayer/ChooseCards —
see `crates/engine/src/interpreter/types.rs:57`) and returns an `Input`. Use a closure
when the test must branch on prompt type:

```rust
InputSource::Player(Box::new(|it: InputType| match it {
    InputType::ChooseCards { .. } => Input::ChooseCards { selected: vec![0] },
    InputType::ChoosePlayer { .. } => Input::ChoosePlayer { idx: 1 },
    _ => Input::Choice { idx: 0 },
}))
```

To assert *how often* a prompt fires (e.g. "ChooseCards re-prompts once then accepts"),
capture a counter in an `Arc<Mutex<usize>>` moved into the closure (see
`quantifier_range_rejects_zero_then_moves_two`, `tests/quantifier_test.rs:148`).

**Note**: the controller's `Player`-path validator re-prompts on out-of-range `Choice`
indices (I-15). A buggy closure that always returns an invalid index will spin forever;
assert prompt counts to catch this.

---

## 5. Trace and Snapshot Assertions

### 5.1 Trace capture

`run_game`'s 5th argument (`trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>`) fires
once per FSM transition with a `TraceEntry::Step { from, to, event }`. Capture into an
`Arc<Mutex<Vec<TraceEntry>>>`:

```rust
let trace: Arc<Mutex<Vec<TraceEntry>>> = Arc::new(Mutex::new(Vec::new()));
let trace_clone = trace.clone();
let gd = run_game(ir, GameData::new(), input, None,
    Some(Box::new(move |e: TraceEntry| { trace_clone.lock().unwrap().push(e); }))).unwrap();
```

Filter by `TraceEvent` variant. The `move_traces` helper in
`tests/quantifier_test.rs:52` is a direct copy-paste template:

```rust
fn move_traces(trace: &[TraceEntry]) -> usize {
    use front_end::ast::{ActionRule, GameRule};
    trace.iter().filter(|e| matches!(e,
        TraceEntry::Step { event: TraceEvent::Action { rule }, .. }
        if matches!(rule, GameRule::Action { action: ActionRule::Move { .. } })
    )).count()
}
```

`TraceEvent` carries typed AST payloads (`rule`/`expr`), so variant filters match
on the node shape rather than on a subtype string; `TraceEvent::summary()` renders a compact
structured one-liner when you need a readable line.

The complete `TraceEvent` enum lives in `crates/engine/src/interpreter/trace.rs`:
`Action`, `Choice`, `OptionalAccept`, `OptionalDecline`, `Condition`, `EndCondition`,
`StageRoundCounter`, `EndStage`, `Trigger`, `Quantifier`.

### 5.2 Snapshot capture

`run_game`'s 4th argument (`event_sender: Option<Box<dyn Fn(&GameData) + Send>>`) fires at
the top of every loop iteration and once more just before `GameOver` return. Capture
clones for diffing:

```rust
let snapshots: Arc<RwLock<Vec<GameData>>> = Arc::new(RwLock::new(Vec::new()));
let sc = snapshots.clone();
let gd = run_game(ir, GameData::new(), input,
    Some(Box::new(move |gd: &GameData| { sc.write().unwrap().push(gd.clone()); })), None).unwrap();
```

See `run_ordering_game_snapshots`, `src/controller/tests.rs:127`. Combine with
`format_game_data(&snap, DebugLevel::Medium)` (see [`observability.md`](./observability.md))
for human-diffable strings.

---

## 6. Driving the Interpreter Manually (no `run_game` ownership loss)

`run_game` takes ownership of `ir`. When a test needs to inspect `ir` *after* the run
(e.g. to assert the quantifier preprocessor didn't mutate it), drive the `Interpreter`
directly:

```rust
// tests/quantifier_test.rs:303 — quantifier_ir_not_mutated_and_memory_cleaned
let ir = load_game("quantifier_deal_any.cgdsl");
let ir_before = format!("{:?}", ir);
let mut interp = Interpreter::new(ir, GameData::new(), None);
loop {
    match interp.step() {
        StepResult::Ok => continue,
        StepResult::NeedsInput(it) => { interp.provide_input(resolve(it)); }
        StepResult::GameOver => break,
        StepResult::Error(e) => panic!("step error: {e}"),
    }
}
assert_eq!(ir_before, format!("{:?}", interp.ir), "self.ir must be unchanged");
```

This is also the pattern for testing the `quantifier` overlay's synthetic-state dispatch
branches (B) and synthetic-memory cleanup (A) in `Interpreter::step` directly.

---

## 7. Fixture Conventions

All fixtures live in `crates/engine/test_games/`. Existing examples (study before
authoring new ones):

| Fixture | Tests | Demonstrates |
|---|---|---|
| `ordering_test.cgdsl` + `.txt` | `src/controller/tests.rs` | Minimal end-to-end golden replay |
| `quantifier_deal_all.cgdsl` | `tests/quantifier_test.rs` | `Quantifier::All` dest-player fan-out |
| `quantifier_deal_any.cgdsl` | `tests/quantifier_test.rs` | `Quantifier::Any` card choice |
| `quantifier_range.cgdsl` | `tests/quantifier_test.rs` | `IntRange` quantity, re-prompt on 0 |
| `quantifier_dest_any.cgdsl` | `tests/quantifier_test.rs` | `AggregatePlayerCollection::Quantifier::Any` dest |
| `quantifier_all_then_any.cgdsl` | `tests/quantifier_test.rs` | Stack of two quantifier sites |
| `setup_location_all.cgdsl` / `_literal.cgdsl` / `_any.cgdsl` | `tests/setup_test.rs` | `SetUpRule` location quantifier handling (all / literal / any-prompt, I-20) |
| `setup_turnorder_all.cgdsl` / `setup_teams_all.cgdsl` | `tests/setup_test.rs` | Setup `All` resolution |
| `auto_end_cycle_no_eligible.cgdsl` / `auto_end_set_memory_skipped.cgdsl` | `tests/actions_test.rs` | I-13 relaxed: `cycle to next` with nobody eligible / skipped `SetMemory` auto-end the stage (named `auto_end_*`, not `errors_*` — they assert success) |
| `turn_switch.cgdsl` | `src/controller/tests.rs`, `tests/actions_test.rs` | Stage entry + turn advance; I-5 counter-once regression |
| `skip_ineligible.cgdsl` | `tests/elimination_test.rs` | Ineligible-player skip + stage auto-end (I-24): eliminated players never prompted, post-elimination instructions skipped, empty winner set |
| `memory_initial_value.cgdsl` | `tests/memory_test.rs` | Memory declarations honour their initial value; typed init (I-10) |
| `bid_prompt.cgdsl` + `.txt` | `tests/memory_test.rs` | Numeric input prompt (`InputType::Number`): `bid any`/range, out-of-bounds re-ask |
| `team_locations.cgdsl` | `tests/teams_test.rs` | Team-owned locations/memories = one shared instance per team |
| `team_pile_reads.cgdsl` | `tests/teams_test.rs` | Team pile addressing: bare name, `X of T:Red`, `owner of`, `&I:M of T:Red` |
| `verb_deal_count.cgdsl` + `.txt` | `tests/verb_semantics_test.rs` | `deal >= M and <= N` / `deal any` prompt for the **count** |
| `verb_deal_range_automatic.cgdsl` | `tests/verb_semantics_test.rs` | Degenerate `>= 2 and <= 2` range deals automatically |
| `verb_deal_count_to_all.cgdsl` + `.txt` | `tests/verb_semantics_test.rs` | Deal-count prompt chains with the dest-`all` fan-out |
| `verb_move_exact_n.cgdsl` + `.txt` | `tests/verb_semantics_test.rs` | `move N` = pick exactly N; wrong count re-prompts |
| `verb_move_exact_n_short_pile.cgdsl` + `.txt` | `tests/verb_semantics_test.rs` | Exact-N prompt clamps to the available cards |
| `verb_positional_automatic.cgdsl` | `tests/verb_semantics_test.rs` | Positional sources (`top(X)`, `X[N]`) never prompt |
| `cycle_skips_out_of_game.cgdsl` | `tests/elimination_test.rs` | Out-of-game-but-in-stage players are skipped by `cycle to next`/`previous` and the `next` expression (I-13 regression) |
| `winner_set_remaining.cgdsl` | `tests/elimination_test.rs` | Winner set = players left in game when no winner statement exists |
| `winner_set_declared.cgdsl` | `tests/elimination_test.rs` | `end game with winner X` eliminates the rest; the survivor is the winner set |
| `behavior_combo_laydown.cgdsl` / `_book.cgdsl` / `_until.cgdsl` | `tests/behavior_test.rs` | Combo lay-down validation, four-of-a-kind books, lay-down-all stage loop (D-16) |
| `location_resolution.cgdsl`, `test.cgdsl` | TUI fixtures | Interactive play (load via `just tui <name>`; allow-listed in `tests/hygiene_test.rs`) |

**Authoring conventions:**

- **Name** the fixture `<area>_<variant>.cgdsl`, snake_case. Add a `.txt` only if the
  golden input sequence is short enough to read.
- Keep fixtures **minimal**: three players (`P1`, `P2`, `P3`), one stage, one or two
  locations (`Stock` / `Hand` / `Discard`), enough cards to exercise the path and no more.
- Put a one-line `# comment` at the top of `.cgdsl` fixtures describing what it exercises.
- Reuse established names (`Stock`, `Hand`, `P1`) so test helpers like
  `player_location`/`table_location` (`tests/quantifier_test.rs:30,44`) work without
  modification.
- If a fixture should error (e.g. `setup_location_any_errors.cgdsl`), name it with the
  `_errors`/`_fails` suffix and assert `result.is_err()` plus the error substring.

---

## 8. Coverage Conventions

Conventions the suite aims for (not yet fully met — see §10):

1. Every `pub fn` in `crates/engine/src/` has at least one direct test, or is exercised
   end-to-end by a named fixture test.
2. Every variant of `ActionRule`, `SetUpRule`, `Payload`, `MoveType`, `OutOf`,
   `EndType`, `ScoringRule` has at least one fixture test that reaches it.
3. Every `Evaluator::eval_*` method (`eval_bool`, `eval_int`, `eval_string`,
   `eval_player`, `eval_team`, `eval_cardset`, `eval_card_position`, `eval_end_condition`,
   `eval_compare`, `eval_int_compare`, `resolve_players`, `resolve_player_collection`,
   `resolve_owner_to_name(s)`, `resolve_quantity`, `expand_types`,
   `check_attr_value_in_cardset`) has unit tests covering happy-path and each documented
   error string from [`error-handling.md`](./error-handling.md) §1.
4. Every invariant I-1..I-25 in [`invariants.md`](./invariants.md) has a regression test
   that fails if the invariant is silently violated.
5. Every `.expect`/`panic!`/`unwrap`/`todo!` site listed in [`error-handling.md`](./error-handling.md)
   §2 has a `#[should_panic(expected = "…")]` test pinning the panic message — unless the
   site is a known bug scheduled for a fix (in which case file it in
   [`engine-vs-design.md`](./engine-vs-design.md) and write the regression test around the *corrected*
   behavior).
6. Every `TraceEvent::Display` arm in `src/interpreter/trace.rs` has a test pinning the
   rendered string format (so trace-log consumers downstream of `cgdsl-engine` don't
   silently break on `Display` rewording).

### 8.1 Documenting behavioral bugs

When a test reveals behavior that disagrees with intent (e.g. the WTO mismatch in I-9, or
the `>` vs `>=` off-by-one in `execute_cardset_move`), the workflow is:

1. Add the bug to [`engine-vs-design.md`](./engine-vs-design.md) with a B-n id if not already present.
2. **Do not** write a "pin current behavior" test — that cements the bug. Instead, write
   the regression test around the *corrected* behavior; it will stay red until the fix lands.
3. Land the fix and the now-passing regression test in the same commit.

---

## 9. Extension Guide — Add a Test for a New Engine Arm

Concrete example: you've added an `ActionRule::StashAction { card_set }`.

1. **Write the fixture first** (`test_games/stash_action_basic.cgdsl`):

   ```
   # Exercise StashAction: move top card of Stock to a side pile, face down.
   game StashTest:
     setup:
       create players P1, P2, P3
       create location Stock owner Table
       create location Side owner Table
       create card { Rank: Ace, Suit: Hearts } on location Stock
     stages:
       Play:
         on enter:
           stash top(Stock) face down to Side
         end stage Play when true
   ```

2. **Write a failing integration test** in `tests/<area>_test.rs` (create the file if new;
   otherwise append):

   ```rust
   use cgdsl_engine::{run_game, GameData, Input, InputSource, InputType};
   use front_end::validation::parse_document;

   fn load_game(name: &str) -> /* …copy helper from §3.3… */ { /* … */ }

   #[test]
   fn stash_action_moves_top_card_to_side() {
       let ir = load_game("stash_action_basic.cgdsl");
       let gd = run_game(ir, GameData::new(),
           InputSource::Player(Box::new(|_| Input::Choice { idx: 0 })),
           None, None).expect("game should complete");
       let stock = gd.locations.iter().find(|l| l.name == "Stock").unwrap();
       assert!(stock.cards.is_empty(), "Stock drained");
       let side = gd.locations.iter().find(|l| l.name == "Side").unwrap();
       assert_eq!(side.cards.len(), 1, "Side received the stashed card");
   }
   ```

3. **Run it red**:

   ```
   cargo test -p cgdsl-engine --test <area>_test
   ```

4. **Implement** the action arm in `src/action.rs` and the trace `rule_signature` subtype
   in `src/interpreter/ir_ext.rs` (add `"Action:StashAction".to_string()`).

5. **Run it green**:

   ```
   cargo test -p cgdsl-engine --test <area>_test
   cargo clippy -p cgdsl-engine --all-targets -- -D warnings
   cargo fmt -p cgdsl-engine -- --check
   ```

6. **If the arm has pure logic**, also add a unit test next to the implementation (e.g. a
   `src/action_tests.rs` wired via `#[cfg(test)] #[path = "action_tests.rs"] mod tests;`
   at the bottom of `src/action.rs`). Build the `ActionRule::StashAction { … }` directly
   and call `action::execute_action_rule(it, &mut gd)` to assert side effects.

7. **If the arm violates or refines an invariant**, update [`invariants.md`](./invariants.md)
   and add a regression test alongside the relevant `I-n` section.

---

## 10. Commands

| Command | Purpose |
|---|---|
| `cargo test -p cgdsl-engine` | Run the entire engine test suite (all layers) |
| `cargo test -p cgdsl-engine --lib` / `--bins` | Unit+dispatch+controller layer / binary smoke tests |
| `cargo test -p cgdsl-engine --test <area>_test` | Run only one integration tests file (e.g. `flow_test`) |
| `cargo test -p cgdsl-engine <test_name>` | Run one named test (substring match) |
| `cargo test -p cgdsl-engine -- --nocapture` | Show `println!` output from tests |
| `cargo clippy -p cgdsl-engine --all-targets --no-deps -- -D warnings` | Lint (per `AGENTS.md`; `--no-deps` skips the pre-existing `front_end`/`code_gen` workspace lints — see `engine-vs-design.md` §5) |
| `cargo fmt -p cgdsl-engine -- --check` | Format check (per `AGENTS.md`) |
| `just test-engine` | Full engine suite (`--all-targets`) |
| `just test-engine-lib` / `just test-engine-bins` | Layer subsets |
| `just test-engine-area <area>` | One integration file (e.g. `just test-engine-area flow`) |
| `just test-engine-ci` | Tests + clippy + fmt, exit code only (the gate) |
| `just coverage-engine` | Line coverage report for the lib target (see below) |
| `just tui [<name>]` | Open the engine TUI on `test_games/<name>.cgdsl` (default `ordering_test`) |

### Coverage

Wired since 2026-08 via `cargo-llvm-cov` (install once:
`cargo install cargo-llvm-cov` + `rustup component add llvm-tools-preview`):

```
just coverage-engine                        # full-suite coverage (all targets, ≈85% lines)
cargo llvm-cov -p cgdsl-engine --lib        # unit-layer only (fast, ≈66% lines)
cargo llvm-cov -p cgdsl-engine --html       # browsable report under target/llvm-cov
```

Baseline (2026-08-11, `--all-targets`): **85.07% lines / 88.75% regions /
84.09% functions**. The largest remaining sinks are the `where`-filter
`Adjacent`/`Higher`/`Lower` success paths plus the combo-laydown integration
in `query/cardset.rs` (~76%) and the no-op/`SetUp` arms of `action.rs`
(~71%) — the documented gap-pin deletion per §8.1 keeps some of those
intentionally uncovered. See §11.

---

## 11. Known Untested Edges

(This section is a holding pen for gaps discovered during refactoring. Append items as
you trip over them; promote to dedicated tests when the area is touched.)

- `src/interpreter/quant_driver.rs` — `take_quant_resume` and every resume
  arm now have direct unit tests (`src/interpreter/quant_driver_tests.rs`),
  pinning the I-19 state-match / I-21 stale-input guards and the dispatch /
  re-prompt / error contract per `PendingKind`. The initial-prompt step arms
  (`step_dest_player_all`, `step_src_cards_any_or_range`, `step_deal_count`,
  `step_combo_source`, …) remain integration-covered via
  `tests/quantifier_test.rs`; under `cargo llvm-cov --all-targets` the file
  sits at ~91% lines.
- `src/controller/trace_logger.rs` — `TraceLogger` has four embedded tests
  (path resolution, open, flush) but the composed
  sender/`catch_unwind` interplay is only exercised indirectly via
  `run_game` when `MCG_TRACE_LOG` is set.
- `src/bin/cgdsl-play.rs` and `src/bin/engine-tui/**` keep embedded smoke tests
  (CLI usage errors, key/input handling — 14 tests); they need private access,
  so they are not promoted to `tests/`.

### Known behavior bugs scheduled for fix (not pin tests)

Per §8.1, these are bugs to fix before writing the regression tests:

- **`optional` decline runs nothing (D-3)** and **stage-exit checks happen at
  entry only (D-2)** — both need parser/IR work and are deferred by design
  (see `engine-vs-design.md` §4).

When fixing, file entries under [`engine-vs-design.md`](./engine-vs-design.md), land the corrected
behavior and its regression test together.

## 12. Behavioral Fixtures — `tests/behavior_test.rs`

Invariant tests (completion, card conservation) prove a game *runs*; behavioral
fixtures prove it *plays the game correctly*. Each fixture mirrors one demo
game's core mechanic with a **deterministic deck** and asserts the **exact**
outcome from the rules:

| Fixture | Mechanic verified | Exact assertions |
|---|---|---|
| `behavior_go_fish_ask.cgdsl` | ask a held rank → transfer, no draw; ask a missing rank → draw exactly 1 | hand sizes, rank counts, deck size |
| `behavior_war.cgdsl` | battle capture + winner declaration | per-round winners, scores, winnings/discard split |
| `behavior_blackjack.cgdsl` | dealing order, dealer draw-until-17, scoring vs dealer, winner | scores, dealer hand, deck exhaustion |
| `behavior_five_card_draw.cgdsl` | hand sum + pair (+10) + flush (+20) bonuses | exact scores 59/40/68, winner |
| `behavior_crazy_eights.cgdsl` | empty-hand win, lowest-score winner | scores, discard/deck counts |

**The determinism trick:** without `shuffle Deck`, card creation order defines
the deck (`expand_types` is rank-major, suit innermost: `Ace-D, Ace-C, Ace-H,
Two-D, ...`), and `deal N` takes the top N cards — so every hand is known. Use
single-suit decks or comma-separated type groups (as in `behavior_war.cgdsl`)
when exact card order matters.

**Track record:** these fixtures caught a real DSL-authoring bug in
`go_fish.cgdsl` — the option body dealt *before* checking emptiness, so a
successful ask also drew a card (draw-on-hit). Fixed by inverting the order
(check first, deal second). Invariant tests could not see this; the game
completed with all 52 cards either way.

---

## 13. Random-Input ("Monkey") Testing — `tests/random_play_test.rs`

The demo games are additionally driven with **fully random player inputs** across
40 seeds per game (`RUNS_PER_GAME`), plus a 10% rate of deliberately
*out-of-range* answers to exercise the controller's re-prompt validation loop
(I-15). The property under test: a well-formed game **never panics, never hangs,
and conserves all 52 cards**, regardless of what the player does.

- `RandomPlayer` (a seeded `StdRng` behind an `Arc<Mutex<..>>`) answers every
  `InputType` uniformly at random within its valid range; answers carry the
  current player's name (tracked via `event_sender`, I-23).
- Infinite re-prompt storms are caught by an input-call cap (`INPUT_CALL_CAP`).
- Panics propagate out of `run_game` and fail the test directly.
- Per-run seeds are derived from one entropy draw and printed in failure
  messages, so a failing *input sequence* is reproducible. The shuffle itself
  uses `rand::thread_rng()` (not injectable), so full replay determinism needs a
  seeded engine RNG (see `NEXT_STEPS.md`).

---

## 14. Cross-References

| Page | When relevant |
|---|---|
| [`README.md`](./README.md) | Hub of the engine wiki; module map |
| [`invariants.md`](./invariants.md) | Read before authoring any engine test — I-1..I-25 are the most common regression targets |
| [`error-handling.md`](./error-handling.md) | Source of `EngineError` variants and their stable `Display` messages to assert against |
| [`lifecycle.md`](./lifecycle.md) | Step sequencing and quantifier pre-dispatch timing — explains *why* traces look the way they do |
| [`observability.md`](./observability.md) | The `event_sender` / `trace_sender` seams tests capture through |
| [`data-structures.md`](./data-structures.md) | Field-level layout for hand-building `GameData` in unit tests |
| [`interfaces.md`](./interfaces.md) | Public API, data flow, threading, and worked examples (golden path + manual driving, §7) reused by §6 |
| [`engine-vs-design.md`](./engine-vs-design.md) | Bugs scheduled for fix; tests should be written around corrected behavior |