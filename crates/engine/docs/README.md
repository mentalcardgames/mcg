---
type: agent_wiki_node
module: crates::engine
scope: [all — crate-level overview]
topics: [overview, architecture, responsibilities, pipeline, module-map, entry-point]
last_validated: 2026-08-11
---

# CGDSL Engine — Overview

> **SCOPE — read first.** Every file in this directory (`crates/engine/docs/`)
> documents **only the `cgdsl-engine` crate** (the `.cgdsl` interpreter). This
> is the complete and only documentation set maintained for it.
>
> The repository root `docs/` and `plans/` directories belong to **other
> projects** (the `native_mcg` poker product, QR protocol, VS Code extension,
> …) and are **not** maintained or kept in sync here — do not treat them as
> engine documentation. When in doubt: if it lives under `crates/engine/docs/`,
> it describes this crate; otherwise it does not.

The engine is the **referee** for card games written in the CGDSL language.
You hand it a `.cgdsl` file — a complete description of a game — and it sets
up the table, runs the rounds, asks the players for their decisions, scores
the game, and reports who won.

It is a deterministic, single-threaded interpreter: given the same game file
and the same player answers, it always produces the same game. There is no
network, no graphics, and no hidden randomness beyond what the game file
asks for (`shuffle`, random turn order).

---

## Core responsibilities

1. **Interpret game definitions.** Read a `.cgdsl` file and turn it into an
   executable *game plan* — a graph of steps that says exactly what happens
   in what order. (The parsing lives in the `front_end` crate; the engine
   consumes its output.)

2. **Keep the game state.** Maintain the live `GameData`: players, teams,
   turn order, piles and their cards, memories, scores, stage counters and
   the stage stack. All state changes flow through one place
   (`action.rs`); all reads flow through another (`query/`).

3. **Enforce the rules.** Walk the game plan step by step, executing actions
   (deal, move, shuffle, score, eliminate…), evaluating conditions, and
   running stages as loops with their end conditions. A game that eliminates
   players keeps itself moving: out-of-game players are never asked for
   input and their instructions are skipped, and a stage (or the whole game)
   ends by itself when no players are left to act.

4. **Ask the players.** Whenever the game needs a decision — a yes/no
   prompt, a choice between options, picking a player, picking cards, or
   entering a number — the engine pauses and returns a structured request
   (`InputType`). The host (a terminal, a UI, a server) answers with an
   `Input`; only the current player's answers are accepted.

5. **Keep it honest.** Inputs are validated (ranges, counts, player
   identity), and errors are recoverable: a failing rule stops the run with
   a typed `EngineError` instead of crashing the process.

6. **Report what happened.** Every step can be observed through trace
   events, and at the end the engine reports the **winner set** — the
   players still in the game (nobody, if all were eliminated).

## How it happens — the pipeline

```
 .cgdsl file          ──parser──▶   game plan (IR)   ──interpreter──▶   GameData
 (your rules)         (front_end)   (a graph of steps)                  (live state)
```

1. **Parse** — `front_end` reads the file and checks it against the
   language grammar. A file that does not follow the syntax is rejected
   here, before anything runs.

2. **Lower** — `front_end` turns the rules into an *IR*: a directed graph
   whose nodes are states and whose edges are the things that happen —
   actions, conditions, choices, stage end-checks, round counters.

3. **Run** — the interpreter (`interpreter/`) starts at the graph's entry
   and takes one edge at a time: execute the action, evaluate the
   condition, enter the stage, count the round. Quantifier sites (`all`,
   `any`, ranges, exact counts) are expanded on the fly into per-player
   fan-outs or player prompts. When a step needs a decision, the run stops
   and returns `NeedsInput`; the host answers and the run resumes.

4. **State** — every mutation lands in `GameData` (players, piles, cards,
   memories, scores, turn order, stages). Cards are plain attribute bags
   referenced by id; piles are ordered lists of card ids; memories are an
   owner-prefixed key-value store.

5. **Finish** — when the graph has no more steps, the run returns the final
   `GameData`; the winner set is the players still in the game.

Because the interpreter is a plain synchronous loop, hosts can drive it two
ways: hand it an input closure and let it run to completion (Mode A), or own
the interpreter and call `step()` yourself, rendering between steps (Mode B
— recommended for UIs). See [`interfaces.md`](./interfaces.md).

---

## The wiki

### Suggested Reading Path

**New here?** Start with [`quickstart.md`](./quickstart.md) — run your first game in two
minutes (CLI flags, logging, debugging), then follow the path below.

**First time here:**
1. [`cgdsl-authoring-guide.md`](./cgdsl-authoring-guide.md) — how to write a game, from the ground up
2. [`dsl-semantics.md`](./dsl-semantics.md) — what every `.cgdsl` construct *means* to the engine
3. [`dsl-completeness.md`](./dsl-completeness.md) — per-construct implementation status (the single status authority)
4. [`lifecycle.md`](./lifecycle.md) — when things happen: setup → stage → play → terminate

**Modifying engine code:**
5. [`invariants.md`](./invariants.md) — 25 guardrails. **Read before touching anything.**
6. [`data-structures.md`](./data-structures.md) — field-level layout of every struct and enum
7. [`contributing.md`](./contributing.md) — development cheatsheet: which files to touch when adding features

**Writing tests:**
8. [`testing.md`](./testing.md) — test layers, fixture conventions, commands

**Integrating from outside:**
9. [`interfaces.md`](./interfaces.md) — public API, data flow, threading, worked examples (the host contract hub)

**Reference:**
10. [`developer-notes.md`](./developer-notes.md) — design decisions (memory ownership, scoring)
11. [`engine-vs-design.md`](./engine-vs-design.md) — divergences from the intended DSL design, with repros
12. [`error-handling.md`](./error-handling.md) — panic sites, recoverable errors, silent no-ops
13. [`observability.md`](./observability.md) — trace hooks, debug output, `MCG_TRACE_LOG`
14. [`mechanics-matrix.md`](./mechanics-matrix.md) — which card-game mechanics the system supports (capability matrix + gap summary)
15. [`NEXT_STEPS.md`](./NEXT_STEPS.md) — future-work project seeds (bachelor/master)

### Module Map

```
crates/engine/src/
├── lib.rs                     — crate root; re-exports public API
├── action.rs                  — execute() dispatch; all game state mutations
├── action_tests.rs
├── controller/
│   ├── mod.rs                 — run_game(), Controller, InputSource, validation
│   ├── tests.rs               — input parsing, integration tests
│   └── trace_logger.rs        — MCG_TRACE_LOG file logger
├── error.rs                   — EngineError enum (thiserror); single error type for the crate
├── game_data.rs               — GameData, Player, Location, MemoryValue, etc.
├── game_data_tests.rs
├── interpreter/
│   ├── mod.rs                     — Interpreter::step(), provide_input()
│   ├── tests.rs                   — Choice/Optional/Condition dispatch tests
│   ├── quant_driver.rs            — quantifier preprocessor: step/resume/fan-out
│   ├── quant_driver_tests.rs
│   ├── ir_ext.rs                  — IrExt trait (edge_labels, rule_signature)
│   ├── ir_ext_tests.rs
│   ├── trace.rs                   — TraceEntry, TraceEvent, Display impls
│   ├── trace_tests.rs
│   ├── trace_tracing.rs           — optional `tracing` bridge (feature-gated)
│   ├── types.rs                   — Input, InputKind, InputType, StepResult
│   └── types_tests.rs
├── quantifier.rs                  — scan_edge, alloc_synth, substitute, fan-out
├── quantifier_tests.rs
├── query/
│   ├── mod.rs                     — Evaluator struct + memory key resolution
│   ├── bool.rs / bool_tests.rs    — eval_bool, eval_end_condition
│   ├── int.rs / int_tests.rs      — eval_int, resolve_quantity
│   ├── string.rs / string_tests.rs— eval_string, expand_types
│   ├── player.rs / player_tests.rs— eval_player, eval_team, resolve_players
│   └── cardset.rs / cardset_tests.rs — eval_cardset, eval_card_position
├── debug/
│   ├── mod.rs                     — DebugLevel, format_game_data, print/save
│   └── tests.rs
├── bin/
│   ├── engine-tui/                — ratatui terminal UI for interactive testing
│   │   ├── main.rs                — thin driver: channel plumbing, render loop, shutdown
│   │   ├── keys.rs                — key handling (quit, navigation, input answers)
│   │   ├── trace.rs               — trace relay types (detail filters)
│   │   └── ui/                    — state, layout, game_state, trace_log, input, controls
│   └── cgdsl-play.rs              — CLI game driver (interactive + file replay; `--log`, `--debug-level` flags)
├── test_games/                    — 64 .cgdsl fixture files (incl. the five demo games) + .txt replay pairs
└── tests/                         — integration tests (one file per feature area, auto-discovered)
    ├── common/mod.rs              — shared harness (load_game, default_input, CurrentTracker, …)
    ├── actions_test.rs            — action arms + regressions (I-5, I-13, D-5, D-11, D-14)
    ├── setup_test.rs              — setup rules incl. setup quantifiers (I-20)
    ├── scoring_test.rs            — cross-boundary scoring: tie/memory/position winners, aggregates
    ├── flow_test.rs               — if/optional/choose/stage flow
    ├── quantifier_test.rs         — all quantifier sites
    ├── elimination_test.rs        — ineligible-player skip, auto-end (I-24), winner-set traces (I-25)
    ├── teams_test.rs              — team-owned locations/memories
    ├── memory_test.rs             — memory initial values + numeric `bid` prompt
    ├── verb_semantics_test.rs     — `deal`/`move` quantity semantics
    ├── behavior_test.rs           — deterministic (non-shuffled) behavioral fixtures with exact expected outcomes
    ├── demo_games_test.rs         — the five handoff demo games driven end-to-end
    ├── random_play_test.rs        — monkey tests: random inputs, 40 seeds per game
    └── hygiene_test.rs            — guard: no orphaned .cgdsl fixtures
```

## Build and Test

```
cargo test -p cgdsl-engine              # 551 tests (463 lib + 5 cgdsl-play + 9 TUI + 74 integration)
cargo test -p cgdsl-engine --test <name> # single integration test file
cargo clippy -p cgdsl-engine --all-targets --no-deps -- -D warnings
cargo fmt -p cgdsl-engine -- --check
just test-engine / test-engine-lib / test-engine-bins / test-engine-area <area>   # class selection
just coverage-engine                      # line coverage via cargo-llvm-cov
just tui [GAME]                          # interactive testing
```
