---
type: agent_wiki_node
module: crates::engine
scope: [all — crate-level overview]
topics: [overview, architecture, module-map, domain-concepts, entry-point]
last_validated: 2026-08-10
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

The engine is a deterministic finite-state-machine interpreter that runs `.cgdsl`
card-game definitions. It parses a game description, builds an IR graph, and steps
through it — executing actions, evaluating conditions, and requesting player input
when needed.

This crate is part of a larger workspace. Other crates you may need:
- `front_end` — the `.cgdsl` parser, AST types, IR lowering, and graph visualizer (`fsm_to_dot` → `.dot`/`.svg`)
- `cgdsl` — VS Code extension with syntax highlighting and LSP integration
- `lsp_server` — language server (diagnostics, completions, semantic highlighting)
- `mcg-cli` — CLI tool for WebSocket/HTTP/Iroh game interaction
- `qr_comm` — research-phase QR communication protocol

## Quick Architecture

```
  .cgdsl file ──→ [Parser] ──→ [IR Builder] ──→ [Interpreter]
                                                     │
  GameData ←── [Action] ←──────── step() ────────────┘
     │            │
     └── [Query/Evaluator] ──→ reads state for conditions/scoring
```

**Six layers:**
1. **Controller** (`controller/`) — loops `step()` + `validate_player_input()`. Exposes `run_game()`.
2. **Interpreter** (`interpreter/`) — FSM step with payload dispatch, quantifier preprocessor, trace emission.
3. **Quantifier** (`quantifier.rs`) — rewrites `all`/`any`/range edges into synthetic edges or input prompts.
4. **Action** (`action.rs`) — mutates `GameData` (setup rules, moves, scoring, etc.).
5. **GameData** (`game_data.rs`) — the live game state (players, cards, locations, memories, turn order).
6. **Query** (`query/`) — evaluates expressions (bool, int, string, player, cardset) against `GameData`.

## Suggested Reading Path

**New here?** Start with [`quickstart.md`](./quickstart.md) — run your first game in two
minutes (CLI flags, logging, debugging), then follow the path below.

**First time here:**
1. [`dsl-semantics.md`](./dsl-semantics.md) — what every `.cgdsl` construct *means* to the engine
2. [`dsl-completeness.md`](./dsl-completeness.md) — per-construct implementation status (the single status authority)
3. [`lifecycle.md`](./lifecycle.md) — when things happen: setup → stage → play → terminate

**Modifying engine code:**
4. [`invariants.md`](./invariants.md) — 23 guardrails. **Read before touching anything.**
5. [`data-structures.md`](./data-structures.md) — field-level layout of every struct and enum
6. [`contributing.md`](./contributing.md) — development cheatsheet: which files to touch when adding features

**Writing tests:**
7. [`testing.md`](./testing.md) — test layers, fixture conventions, commands

**Integrating from outside:**
8. [`interfaces.md`](./interfaces.md) — public API, data flow, threading, worked examples (the host contract hub)

**Reference:**
9. [`developer-notes.md`](./developer-notes.md) — design decisions (memory ownership, scoring)
10. [`engine-vs-design.md`](./engine-vs-design.md) — divergences from the intended DSL design, with repros
11. [`error-handling.md`](./error-handling.md) — panic sites, recoverable errors, silent no-ops
12. [`observability.md`](./observability.md) — trace hooks, debug output, `MCG_TRACE_LOG`

**Handoff set (2026-08):**
13. [`mechanics-matrix.md`](./mechanics-matrix.md) — which card-game mechanics the system supports (capability matrix + gap summary)
14. [`NEXT_STEPS.md`](./NEXT_STEPS.md) — future-work project seeds (bachelor/master)

## Module Map

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
├── test_games/                    — 73 .cgdsl fixture files (incl. the five demo games)
└── tests/                         — integration tests (one file per feature area)
    ├── quantifier_test.rs
    ├── action_test.rs
    ├── scoring_test.rs
    ├── setup_test.rs
    ├── turn_test.rs
    ├── out_test.rs
    ├── memory_test.rs
    ├── flow_test.rs
    ├── optional_test.rs
    ├── shuffle_test.rs
    ├── demo_games_test.rs         — the five handoff demo games driven end-to-end
    ├── behavior_test.rs           — deterministic (non-shuffled) behavioral fixtures with exact expected outcomes
    └── random_play_test.rs        — monkey tests: random inputs, 40 seeds per game
```

## Key Concepts

- **IR (Intermediate Representation):** The parser lowers `.cgdsl` into a graph of states and edges.
  Each edge carries a `Payload` (Action, Choice, Optional, Condition, EndCondition, Trigger, StageRoundCounter).
  States are identified by `StateID` (u32). The interpreter advances through the graph one edge at a time.

- **GameData:** The live game state — players, teams, turn order, locations, cards, memories, scores,
  stage counters, and the stage stack. All mutations go through `action.rs`. All reads go through `query/`.

- **Input:** When the interpreter hits a `Choice`, `Optional`, `ChoosePlayer`, or `ChooseCards` payload,
  it returns `StepResult::NeedsInput(InputType)` and waits. The controller delivers player input as
  `Input { player_id, kind: InputKind }`. Inputs from non-current players are rejected.

- **Quantifier Preprocessor:** Edges carrying `all`/`any`/range quantifiers are intercepted before
  dispatch. `all` fans out to one synthetic edge per player. `any` issues a ChoosePlayer/ChooseCards
  prompt. `>= M and <= N` validates the selection and re-prompts on failure.

- **Memory:** Per-player key-value store (`HashMap<String, MemoryValue>`) with owner-prefixed keys
  (`"P1_M"`, `"Table_pot"`). See [`developer-notes.md`](./developer-notes.md) §1.1 for the ownership model.

- **Scoring:** `score N to P1` adds to `Player::score`. `winner is P1` eliminates all other players.
  `winner is highest score` compares scores across all in-game players and eliminates non-matching.
  See [`dsl-semantics.md`](./dsl-semantics.md) §4.

## Build and Test

```
cargo test -p cgdsl-engine              # 545 tests (440 lib + 5 cgdsl-play + 9 TUI + 91 integration, +1 ignored)
cargo test -p cgdsl-engine --test <name> # single integration test file
cargo clippy -p cgdsl-engine --all-targets -- -D warnings
cargo fmt -p cgdsl-engine -- --check
just tui [GAME]                          # interactive testing
```
