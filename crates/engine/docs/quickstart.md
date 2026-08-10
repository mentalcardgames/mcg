---
type: agent_wiki_node
module: crates::engine
scope: [all — crate-level overview]
topics: [quickstart, getting-started, cli, logging, debugging, first-run]
associated_files:
  - crates/engine/src/bin/cgdsl-play.rs
  - crates/engine/src/bin/engine-tui/main.rs
  - crates/engine/src/controller/trace_logger.rs
  - crates/engine/src/controller/mod.rs
last_validated: 2026-08-11
---

# Quick Start

The fastest path from "nothing installed" to "a card game is running" — about two minutes.
This page is the on-ramp; each section points deeper into the wiki.

## 1. Prerequisites

- Rust stable (plus nothing else — the engine has no WASM, server, or LSP requirements).
- Optional: [`just`](https://github.com/casey/just) for the convenience recipes (`just tui`, …).

## 2. Run a game

**Interactive play** (you answer the prompts in the terminal):

```sh
cargo run -p cgdsl-engine --bin cgdsl-play -- crates/engine/test_games/go_fish.cgdsl
```

Other demo games: `blackjack.cgdsl`, `five_card_draw.cgdsl`, `crazy_eights.cgdsl`, `war.cgdsl`.

**Scripted replay** (answers read from a file, one per line):

```sh
cargo run -p cgdsl-engine --bin cgdsl-play -- crates/engine/test_games/ordering_test.cgdsl \
    crates/engine/test_games/ordering_test.txt
```

**Terminal UI** (the ratatui harness — arrow keys + Enter, `p` switches player perspective,
`q` quits):

```sh
just tui                                        # defaults to ordering_test.cgdsl
just tui crates/engine/test_games/blackjack.cgdsl   # any .cgdsl fixture
```

## 3. CLI flags (`cgdsl-play`)

```text
Usage: cgdsl-play [OPTIONS] <game.cgdsl> [input.txt]

  --log <path>            write the MCG trace log to <path>
  --debug-level <L>       after the run, print the full GameData dump
                          at level low | medium | high
  -h, --help              show help (also lists exit codes)

Exit codes:  0 completed · 1 usage · 2 read error · 3 parse error · 4 engine error
```

Example — run a game, write a trace file, and dump the final state at full verbosity:

```sh
cargo run -p cgdsl-engine --bin cgdsl-play -- --log trace.log --debug-level high \
    crates/engine/test_games/go_fish.cgdsl
```

## 4. Logging & debugging

- **Trace file:** set `MCG_TRACE_LOG=path` (any path; `off`/`none` disables) or pass
  `--log <path>`. The file gets one line per FSM transition
  (`[Step NNN] [from->to] <event>`), a stamped header (engine version, UTC timestamp, game name),
  and a step-counted footer naming the winner set (`=== GameOver after N steps — winners: … ===`). Details and the resolution order:
  [`observability.md`](./observability.md) §3.
- **State dumps:** `--debug-level low|medium|high` prints `format_game_data` at the end of the
  run; `Low` = players/current stage/card counts, `Medium` adds scores/teams/memories,
  `High` = everything (every card, all memories, combos, point maps). See `observability.md` §4.
- **Errors:** every failure is a typed `EngineError` (never a raw string); panics from engine
  invariants can be converted to errors with `capture_panics(true)`. See
  [`error-handling.md`](./error-handling.md).

## 5. Using the engine as a library

```rust
use cgdsl_engine::{run_game_with, GameData, InputSource, RunOptions};
use front_end::validation::parse_document;

let game = parse_document(&std::fs::read_to_string("game.cgdsl")?)?;
let ir = game.to_lowered_graph();
let state = run_game_with(
    ir,
    GameData::new(),
    InputSource::TestFile("inputs.txt".into()),   // or InputSource::Player(closure)
    RunOptions::new()
        .with_log_path("trace.log".into())        // optional trace file
        .with_game_name("game.cgdsl")             // optional header tag
        .capture_panics(true),                    // panics -> Err(EngineError::InternalPanic)
)?;
```

For the full host contract (event/trace callbacks, manual Mode-B driving, the
`InputType` ⇄ `Input` turn contract): [`interfaces.md`](./interfaces.md) §4 and §7.

## 6. Tests & tooling

```sh
cargo test -p cgdsl-engine                 # full suite (551 tests)
cargo test -p cgdsl-engine --test flow_test  # one integration file
cargo clippy -p cgdsl-engine --all-targets --no-deps -- -D warnings
cargo fmt -p cgdsl-engine -- --check
```

## 7. Where to go next

- [`README.md`](./README.md) — the wiki hub with the full reading path.
- [`testing.md`](./testing.md) — test layers, fixture conventions.
- [`cgdsl-authoring-guide.md`](./cgdsl-authoring-guide.md) — writing your own `.cgdsl` games.
- [`contributing.md`](./contributing.md) — development cheatsheet.
