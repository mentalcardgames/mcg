---
type: agent_wiki_node
module: crates::engine
scope: [engine::controller, engine::interpreter, engine::game_data]
topics: [threading, send-sync, memory, lifetimes, resources]
associated_files:
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/controller/trace_logger.rs
  - crates/engine/src/interpreter/mod.rs
  - crates/engine/src/game_data.rs
  - crates/engine/Cargo.toml
last_validated: 2026-07-28
---

# Concurrency, Memory & Thread Safety

The **production** engine logic is single-threaded and fully synchronous: there is no `tokio`,
no `async`, no `spawn`. *Observability* plumbing, however, does use `std::sync::Arc<Mutex<…>>` in
exactly two places — the trace writer (`crates/engine/src/controller/trace_logger.rs:10`) and the
step counter (`crates/engine/src/controller/mod.rs:67`) — purely to share the value between the
main loop closure and the composed trace sender; no threads are spawned. `run_game` additionally
wraps the run in `std::panic::catch_unwind(AssertUnwindSafe(...))` when a trace log is open (see
[`observability.md`](./observability.md)). This page documents the threading model, the auto-trait
`Send`/`Sync` status of every public type, the resource lifecycle, and the dependency hygiene. For
*how* the single thread loops, see [`lifecycle.md`](./lifecycle.md).

---

## 1. Threading Model

The **production** engine logic is single-threaded and fully synchronous: no `tokio`, `async`, or
`spawn`. The only `std::sync` usage sits in the trace-logging plumbing (post-Stage-5 additions):

- `crates/engine/src/controller/trace_logger.rs:10` — `TraceLogger` stores
  `Arc<Mutex<BufWriter<File>>>` so the composed trace-sender closure (handed to `Interpreter`) can
  write log lines back into the same writer the controller holds.
- `crates/engine/src/controller/mod.rs:67` — `run_game` allocates `Arc<Mutex<usize>>` as a step
  counter shared between the `run` loop (`controller/mod.rs:154`) and the composed trace sender
  closure (`controller/mod.rs:71-84`).
- `crates/engine/src/controller/mod.rs:98-117` — when a trace log is open, `run_game` wraps
  `controller.run()` in `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`, logs any
  panic to the trace file, then `std::panic::resume_unwind`s to re-panic in the caller. No thread
  is spawned; this is purely panic-capture.

The main loop (`crates::engine::controller::Controller::run`,
`crates/engine/src/controller/mod.rs:151-169`) is a plain `loop { … }` that calls
`self.interpreter.step()` directly. The only traditional "concurrency-relevant" construct on the
public contract is the `Send + Sync` bound on the `Player` input closure
(`crates/engine/src/controller/mod.rs:23`), which exists so a *host* application can move an
`crates::engine::controller::InputSource` across threads — but the engine itself never spawns
threads.

---

## 2. `Send` / `Sync` Characteristics

None of the engine types explicitly implement (or derive) `Send`/`Sync`; their auto-trait status
follows from their fields:

| Type | Auto `Send`? | Auto `Sync`? | Rationale |
|---|---|---|---|
| `crates::engine::game_data::GameData` | yes | yes | All fields are `Vec`/`HashMap` of `String`/`usize`/`i32`/`bool`/`u32`; no `Rc`/`RefCell`/raw. |
| `crates::engine::interpreter::Interpreter` | yes | **no** | Now carries `trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>` (`crates/engine/src/interpreter/mod.rs:31`). A `Box<dyn Fn + Send>` is `Send` but **not `Sync`** (the trait object is `Fn + Send`, not `Fn + Send + Sync`). Was `Send + Sync` pre-Stage-5. |
| `crates::engine::controller::Controller` | yes | **no** | Unchanged in conclusion, now for two reasons: `event_sender: Option<Box<dyn Fn(&GameData) + Send>>` (`crates/engine/src/controller/mod.rs:140`) is `Send` not `Sync`, AND `step_count: Arc<Mutex<usize>>` (`controller/mod.rs:145`) is `Send + Sync` but cannot rescue `Sync`. Two threads cannot share `&Controller` safely. |
| `crates::engine::controller::InputSource` | yes | yes | `Player(Box<dyn Fn(InputType) -> Input + Send + Sync>)` explicitly `Send + Sync` (`crates/engine/src/controller/mod.rs:23`); `TestFile(PathBuf)` is `Send + Sync`. |
| `crates::engine::query::Evaluator` | yes | yes | Zero-sized; no interior mutability. |
| `crates::engine::interpreter::StepResult` | yes | yes | Plain enum (`String` is `Send+Sync`). |
| `crates::engine::interpreter::Input`, `crates::engine::interpreter::InputType`, `crates::engine::debug::DebugLevel` | yes | yes | Plain data. |
| `crates::engine::interpreter::TraceEntry`, `crates::engine::interpreter::TraceEvent` | yes | yes | Plain enums of `String`/`u32`/`usize`/`bool`/`Vec<String>` (`crates/engine/src/interpreter/trace.rs:1-46`). |

**Interior mutability:** none in the production state machine. The engine uses `&mut GameData`
passed down the call stack (`crates::engine::action::execute(payload, &mut game_data)`,
`crates::engine::query::Evaluator` takes `&GameData`). There are no `RefCell`/`Cell`/`RwLock` in
the production path. The two `Mutex` uses noted in §1 (`BufWriter<File>` and `usize` step counter)
are observation-only and never guard engine state. The shared mutability pattern through
callbacks (`event_sender`, `trace_sender`) is read-only with respect to the engine: `event_sender`
receives `&GameData`; `trace_sender` receives `TraceEntry` by value. Hosts that need to snapshot
must `clone()`.

**Implication for hosts:** because `event_sender` is `Send` but not `Sync`, a host that wants to
emit events from multiple worker threads must wrap the engine in its own `Mutex<Controller>` or run
the engine on a single dedicated thread and communicate via channels.

---

## 3. Resource Management

- **Memory:** `crates::engine::game_data::GameData` is a flat aggregate of owned
  `Vec`/`HashMap`. The only large allocation point is the terminal `clone()` in
  `crates::engine::controller::Controller::run`
  (`crates/engine/src/controller/mod.rs:164`) — for a game
  with many cards this is O(total state). There is no arena, slab, or recycling; card ids are never
  reused (only appended), so `cards.len()` grows monotonically. The quantifier overlay
  (`crates::engine::interpreter::Interpreter::pending_overlay`,
  `crates/engine/src/interpreter/mod.rs:36`) only ever holds synthetic-state replacement edges
  during a single quantifier edge's resolution; it is bounded by the fan-out cap
  (`crate::quantifier::FANOUT_CAP = 64`, `crates/engine/src/quantifier.rs:38`) and never leaks past
  the overlay-dispatch completion.
- **File descriptors:** the test-input `std::fs::File`
  (`crates/engine/src/controller/mod.rs:205`) is
  opened lazily on the first `NeedsInput` and its `BufReader` is consumed and dropped within
  `crates::engine::controller::Controller::read_test_file`'s loading block
  (`crates/engine/src/controller/mod.rs:204-218`). No FD leaks across a run.
  `crates::engine::debug::save_game_data` (`crates/engine/src/debug/save.rs:8-24`) opens a file in
  `append(true).create(true)` mode per call and drops it on return. When trace logging is enabled,
  `TraceLogger::open` (`crates/engine/src/controller/trace_logger.rs:14-20`) creates the log file
  once per `run_game` invocation and drops it on return from `run_game`.
- **Network sockets:** none. The engine has no networking; per the workspace's P2P architecture
  intent, each player runs their own backend and this crate is transport-agnostic.
- **Drop order:** `crates::engine::controller::Controller` owns
  `crates::engine::interpreter::Interpreter` owns `crates::engine::game_data::GameData`; standard
  Rust drop order suffices. No `Drop` impls exist in the crate.

---

## 4. Dependencies Inventory

`crates/engine/Cargo.toml` declares:

**In use — production library target:**
- `front_end` — IR, AST, and lowering types.
- `serde` + `serde_json` — used by `alloc_synth` for `StateID` construction.

**In use — `engine-tui` binary:**
- `ratatui` + `crossterm` — terminal UI.
- `crossbeam-channel` — threaded input loop.

**`cgdsl-play` binary:**
- No extra dependencies (auto-discovered, no `[[bin]]` entry needed).

**Not in `Cargo.toml`:** no unused dependencies remain.
Error handling is stringly-typed (`Result<_, String>`).
`rand` is imported from `front_end`'s dependency tree, not directly.

> Note: `crates/engine/src/bin/cgdsl-play.rs` is auto-discovered by cargo. Only `engine-tui`
> has an explicit `[[bin]]` entry in `Cargo.toml`.

last_validated: 2026-07-28
