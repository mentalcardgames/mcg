---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [contributing, development, workflow, cheatsheet]
last_validated: 2026-08-10
---

# Contributing — Development Cheatsheet

Where to touch when adding or modifying engine functionality.

## Adding a new Action Rule

4 files to change, in order:

| # | File | What to do |
|---|------|-----------|
| 1 | `crates/front_end/src/ast.rs` | Add the variant to `ActionRule` (or `SetUpRule`, `ScoringRule`, etc.) |
| 2 | `crates/front_end/src/parser.rs` | Add a parser function and grammar rule (`grammar.pest`) |
| 3 | `crates/engine/src/action.rs` | Add the execution arm in `execute_action_rule()` — mutate `GameData` here |
| 4 | `crates/engine/src/interpreter/ir_ext.rs` | Add a `rule_signature` subtype string for trace emission |
| 5 | `crates/engine/src/action_tests.rs` | Write a unit test (build AST directly, call `execute_action_rule`, assert on `GameData`) |
| 6 | `crates/engine/test_games/` + `tests/` | Write a `.cgdsl` fixture + integration test |

## Adding a new Query Evaluator method

| # | File | What to do |
|---|------|-----------|
| 1 | `crates/engine/src/query/<file>.rs` | Add `impl Evaluator { pub fn eval_*(...) }` |
| 2 | `crates/engine/src/query/<file>_tests.rs` | Unit tests (call `Evaluator::eval_*(...)` with hand-built AST + `GameData`) |

## Adding a new DSL construct (parser → engine)

| # | File | What to do |
|---|------|-----------|
| 1 | `crates/front_end/src/grammar.pest` | Add the grammar rule |
| 2 | `crates/front_end/src/parser.rs` | Add the parser function |
| 3 | `crates/front_end/src/ast.rs` | Add the AST variant |
| 4 | `crates/front_end/src/ir.rs` | Add lowering logic (if control flow) or let it pass through as `Payload::Action` (if action) |
| 5 | `crates/engine/src/action.rs` | Add the execution arm |
| 6 | `crates/engine/docs/dsl-semantics.md` | Document the new construct's semantics |

## Changing the public API

| # | File | What to do |
|---|------|-----------|
| 1 | The source file (e.g., `types.rs`) | Add/modify the type or function |
| 2 | `crates/engine/src/lib.rs` | Re-export if it should be public |
| 3 | `crates/engine/docs/interfaces.md` | Update the public API inventory |
| 4 | `crates/engine/docs/interfaces.md` | Update the public API inventory (§1) and the worked examples (§7) if the integration pattern changed |

## Testing conventions

- **Unit tests** live alongside source files (e.g., `action_tests.rs`, `query/int_tests.rs`) — wired via `#[cfg(test)] #[path = "..."] mod tests;`
- **Fixture tests** live in `crates/engine/tests/` — one file per feature area
- **Fixtures** live in `crates/engine/test_games/` — minimal `.cgdsl` files, 2-3 players, named `<area>_<variant>.cgdsl`
- **Run:** `cargo test -p cgdsl-engine` (full suite), `cargo test -p cgdsl-engine --test <name>` (single file)

## Guardrails

- **Read `invariants.md` before modifying any engine code.** 23 documented invariants.
- **Read `dsl-semantics.md` before changing DSL semantics.**
- **Check `developer-notes.md`** for design decisions and known gaps.
- **Memory ownership:** memory keys are prefixed with the owner name (`"P1_M"`, `"Table_pot"`). Use `Evaluator::resolve_memory_key()` in query code, never access `memories` directly.
- **Error model:** enum-typed — every fallible path returns `Result<_, EngineError>` (see `error.rs`
  and [`error-handling.md`](./error-handling.md)). The `Display` messages are stable strings that
  tests assert via `to_string()`; add new variants to `crates/engine/src/error.rs` rather than
  inventing ad-hoc string errors.

## Build commands

```
cargo test -p cgdsl-engine                    # 546 tests
cargo clippy -p cgdsl-engine --all-targets -- -D warnings
cargo fmt -p cgdsl-engine -- --check
just tui [GAME]                                # interactive TUI testing
```
