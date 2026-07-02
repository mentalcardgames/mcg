---
type: agent_wiki_node
module: crates::engine
scope: [engine::debug, engine::controller, engine::action]
topics: [observability, telemetry, debugging, events, diagnostics]
associated_files:
  - crates/engine/src/debug.rs
  - crates/engine/src/controller.rs
  - crates/engine/src/action.rs
last_validated: 2026-07-02
---

# Observability & Diagnostics

The engine has **no `tracing`/`log` integration** (verified: zero imports of either in
`crates/engine/src`). Telemetry is provided by three mechanisms: a reactive event callback, the
structured `crates/engine/src/debug.rs` dumps, and a single ad-hoc `eprintln!`. For the error
channels themselves, see [`error-handling.md`](./error-handling.md).

---

## 1. Reactive Event Callback

`crates::engine::controller::run_game` accepts an optional
`event_sender: Option<Box<dyn Fn(&GameData) + Send>>` (`crates/engine/src/controller.rs:52`). It is
invoked by `crates::engine::controller::Controller::emit_event`
(`crates/engine/src/controller.rs:161-165`) at the **top of every loop iteration** *and* once more
just before returning `GameOver` (`crates/engine/src/controller.rs:64,73`). The callback receives
`&GameData`, so a host can render or snapshot after every single transition without polling. This
is the recommended observability seam for production hosts.

> The callback receives a shared `&GameData` — it must not mutate through it. Hosts that need a
> snapshot must `clone()`. The bound is `Fn(&GameData) + Send` but **not** `Sync` — see
> [`concurrency.md`](./concurrency.md) §2 for multi-thread host implications.

---

## 2. `crates/engine/src/debug.rs` — Structured Human-Readable Dumps

Three verbosity levels selectable via `crates::engine::debug::DebugLevel`
(`crates/engine/src/debug.rs:6-30`):

```rust
pub enum DebugLevel { Low, Medium, High }
impl DebugLevel {
    pub fn from_marker(s: &str) -> Option<Self>;   // parses "<!--LOW-->" etc., case-insensitive
    pub fn marker(&self) -> &'static str;          // inverse
}
pub fn format_game_data(data: &GameData, level: DebugLevel) -> String;
pub fn print_game_data(data: &GameData, level: DebugLevel);
pub fn save_game_data(data: &GameData, path: &Path) -> io::Result<()>;
```

| Level | Includes |
|---|---|
| `crates::engine::debug::DebugLevel::Low` (`crates/engine/src/debug.rs:40-71`) | Player names, current player, current stage, turn-order indices, card count per location. |
| `crates::engine::debug::DebugLevel::Medium` (`crates/engine/src/debug.rs:73-157`) | All of Low + per-player scores, teams, all memories, per-location card names (first 5 if >5). |
| `crates::engine::debug::DebugLevel::High` (`crates/engine/src/debug.rs:159-249`) | Full dump: every player with `in_stage` map, teams with raw indices, all locations with raw card-id vecs, every card's full attribute map, stage stack, all stage counters, all memories (typed), combos, precedences, point maps. |

`crates::engine::debug::save_game_data` (`crates/engine/src/debug.rs:255-271`) **appends** to
`path` (creating it if absent) and **persists the `DebugLevel`** as an HTML comment marker on the
file's first line: on subsequent calls it reads the existing first line via
`crates::engine::debug::DebugLevel::from_marker` and reuses that level (defaulting to `Medium` if
absent/unparseable). This round-trip (`DebugLevel::marker` ↔ `DebugLevel::from_marker`, tested at
`crates/engine/src/debug.rs:278-315`) lets a log file self-describe its verbosity.

---

## 3. Ad-hoc stderr

The only direct stdlib logging in the crate is
`eprintln!("ShuffleAction failed: {e}")` (`crates/engine/src/action.rs:176`). There are no
`dbg!`/`println!` calls in the library target (the `println!`s in
`crates/engine/src/bin/cgdsl-play.rs` are CLI output, not engine telemetry).

---

## 4. Agent Guidance for Adding Telemetry

Prefer extending `crates/engine/src/debug.rs`'s level-gated formatter or emitting from the
`event_sender` callback rather than sprinkling `println!`/`eprintln!` through
`crates/engine/src/action.rs`/`crates/engine/src/query.rs`/`crates/engine/src/interpreter.rs`. If
structured logging is desired, `tracing` would need to be added to `crates/engine/Cargo.toml` (it
is not currently a dependency) and spans established around
`crates::engine::interpreter::Interpreter::step` /
`crates::engine::action::execute` /
`crates::engine::query::Evaluator::eval_*`; do not assume spans already exist.
