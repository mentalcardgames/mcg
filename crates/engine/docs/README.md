---
type: agent_wiki_node
module: crates::engine
scope: [all — crate-level overview]
topics: [overview, architecture, module-map, domain-concepts, entry-point]
last_validated: 2026-07-28
---

# CGDSL Engine — Overview

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

**First time here:**
1. [`dsl-semantics.md`](./dsl-semantics.md) — what every `.cgdsl` construct means to the engine
2. [`lifecycle.md`](./lifecycle.md) — when things happen: setup → stage → play → terminate

**Modifying engine code:**
3. [`invariants.md`](./invariants.md) — 23 guardrails. **Read before touching anything.**
4. [`data-structures.md`](./data-structures.md) — field-level layout of every struct and enum
5. [`contributing.md`](./contributing.md) — development cheatsheet: which files to touch when adding features

**Writing tests:**
6. [`testing.md`](./testing.md) — test layers, fixture conventions, commands

**Integrating from outside:**
7. [`interfaces.md`](./interfaces.md) — public API surface, data flow
8. [`api-usage.md`](./api-usage.md) — `run_game()` examples

**Reference:**
9. [`developer-notes.md`](./developer-notes.md) — design decisions, completeness audit, known bugs
10. [`error-handling.md`](./error-handling.md) — panic sites, recoverable errors, silent no-ops
11. [`concurrency.md`](./concurrency.md) — threading model, `Send`/`Sync`
12. [`observability.md`](./observability.md) — trace hooks, debug output, `MCG_TRACE_LOG`

**Handoff set (2026-08):**
13. [`dsl-completeness.md`](./dsl-completeness.md) — per-construct status: grammar → IR → engine
14. [`engine-vs-design.md`](./engine-vs-design.md) — divergence/bug catalog with repros + demo-game index
15. [`NEXT_STEPS.md`](./NEXT_STEPS.md) — future-work project seeds (bachelor/master)

## Module Map

```
crates/engine/src/
├── lib.rs                         — crate root; re-exports public API
├── action.rs                      — execute() dispatch; all game state mutations
├── action_tests.rs
├── controller/
│   ├── mod.rs                     — run_game(), Controller, InputSource, validation
│   ├── tests.rs                   — input parsing, integration tests
│   └── trace_logger.rs            — MCG_TRACE_LOG file logger
├── game_data.rs                   — GameData, Player, Location, MemoryValue, etc.
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
│   │   ├── main.rs, trace.rs
│   │   └── ui/                    — state, layout, game_state, trace_log, input, controls
│   └── cgdsl-play.rs              — CLI game driver (interactive + file replay)
├── test_games/                    — 63 .cgdsl fixture files (incl. the five demo games)
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
    └── demo_games_test.rs         — the five handoff demo games driven end-to-end
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
cargo test -p cgdsl-engine              # 463 tests (406 unit + 57 integration)
cargo test -p cgdsl-engine --test <name> # single integration test file
cargo clippy -p cgdsl-engine --all-targets -- -D warnings
cargo fmt -p cgdsl-engine -- --check
just tui [GAME]                          # interactive testing
```
