---
type: agent_wiki_node
module: crates::engine
scope: [engine::controller, engine::interpreter, engine::game_data]
topics: [threading, send-sync, memory, lifetimes, resources]
associated_files:
  - crates/engine/src/controller.rs
  - crates/engine/src/interpreter.rs
  - crates/engine/src/game_data.rs
  - crates/engine/Cargo.toml
last_validated: 2026-07-02
---

# Concurrency, Memory & Thread Safety

The engine is deliberately **single-threaded and fully synchronous**. There is no `tokio`, no
`async`, no `spawn`, no `Arc`/`Mutex`/`RwLock` in `crates/engine/src`. This page documents the
threading model, the auto-trait `Send`/`Sync` status of every public type, the resource
lifecycle, and the dependency hygiene. For *how* the single thread loops, see
[`lifecycle.md`](./lifecycle.md).

---

## 1. Threading Model

The engine is **single-threaded and fully synchronous**. There is no `tokio`, `async`, `spawn`, or
`Arc`/`Mutex` in `crates/engine/src`. The main loop
(`crates::engine::controller::Controller::run`, `crates/engine/src/controller.rs:62-79`) is a plain
`loop { … }` that calls `self.interpreter.step()` directly. The only "concurrency-relevant"
construct is the `Send + Sync` bound on the `Player` input closure
(`crates/engine/src/controller.rs:16`), which exists so a *host* application can move an
`crates::engine::controller::InputSource` across threads — but the engine itself never spawns
threads.

---

## 2. `Send` / `Sync` Characteristics

None of the engine types explicitly implement (or derive) `Send`/`Sync`; their auto-trait status
follows from their fields:

| Type | Auto `Send`? | Auto `Sync`? | Rationale |
|---|---|---|---|
| `crates::engine::game_data::GameData` | yes | yes | All fields are `Vec`/`HashMap` of `String`/`usize`/`i32`/`bool`/`u32`; no `Rc`/`RefCell`/raw. |
| `crates::engine::interpreter::Interpreter` | yes | yes | `front_end::ir::Ir<front_end::ir::LoweredPayLoad>` (serde types), `GameData`, `Vec<Input>`, `front_end::ir::StateID(u32)`. |
| `crates::engine::controller::Controller` | yes | **no** | `event_sender: Option<Box<dyn Fn(&GameData) + Send>>` — the bound is `Send` but **not `Sync`** (`crates/engine/src/controller.rs:52`). Two threads cannot share `&Controller` safely. |
| `crates::engine::controller::InputSource` | yes | yes | `Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)` explicitly `Send + Sync`; `TestFile(PathBuf)` is `Send + Sync`. |
| `crates::engine::query::Evaluator` | yes | yes | Zero-sized; no interior mutability. |
| `crates::engine::interpreter::StepResult` | yes | yes | Plain enum (`String` is `Send+Sync`). |
| `crates::engine::interpreter::Input`, `crates::engine::interpreter::InputType`, `crates::engine::debug::DebugLevel` | yes | yes | Plain data. |

**Interior mutability:** none. The engine uses `&mut GameData` passed down the call stack
(`crates::engine::action::execute(payload, &mut game_data)`,
`crates::engine::query::Evaluator` takes `&GameData`). There are no
`RefCell`/`Cell`/`Mutex`/`RwLock` anywhere in the crate. The only shared mutability pattern is the
`event_sender` callback, which receives `&GameData` (shared ref) — it must not attempt to mutate
through the provided reference; hosts that need to snapshot must `clone()`.

**Implication for hosts:** because `event_sender` is `Send` but not `Sync`, a host that wants to
emit events from multiple worker threads must wrap the engine in its own `Mutex<Controller>` or run
the engine on a single dedicated thread and communicate via channels.

---

## 3. Resource Management

- **Memory:** `crates::engine::game_data::GameData` is a flat aggregate of owned
  `Vec`/`HashMap`. The only large allocation point is the terminal `clone()` in
  `crates::engine::controller::Controller::run` (`crates/engine/src/controller.rs:74`) — for a game
  with many cards this is O(total state). There is no arena, slab, or recycling; card ids are never
  reused (only appended), so `cards.len()` grows monotonically.
- **File descriptors:** the test-input `std::fs::File` (`crates/engine/src/controller.rs:118`) is
  opened lazily on the first `NeedsInput` and its `BufReader` is consumed and dropped within
  `crates::engine::controller::Controller::read_test_file`'s loading block
  (`crates/engine/src/controller.rs:117-129`). No FD leaks across a run.
  `crates::engine::debug::save_game_data` (`crates/engine/src/debug.rs:255-271`) opens a file in
  `append(true).create(true)` mode per call and drops it on return.
- **Network sockets:** none. The engine has no networking; per the workspace's P2P architecture
  intent, each player runs their own backend and this crate is transport-agnostic.
- **Drop order:** `crates::engine::controller::Controller` owns
  `crates::engine::interpreter::Interpreter` owns `crates::engine::game_data::GameData`; standard
  Rust drop order suffices. No `Drop` impls exist in the crate.

---

## 4. Unused Dependencies (Agent Note)

`crates/engine/Cargo.toml` declares `indexmap`, `dashmap`, `thiserror`, and `anyhow`, but **none
are imported anywhere in `crates/engine/src`** (verified: only `std::collections::HashMap` is used,
in `crates/engine/src/game_data.rs`, `crates/engine/src/query.rs`, and
`crates/engine/src/action.rs`). Error handling is stringly-typed (`Result<_, String>`,
`crates::engine::interpreter::StepResult::Error(String)`) — `thiserror`/`anyhow` are not exercised.
Agents should not assume these crates are available to engine code without re-adding a real import;
conversely, removing them from `Cargo.toml` is safe as of this writing. See
[`error-handling.md`](./error-handling.md) for the error model in use.
