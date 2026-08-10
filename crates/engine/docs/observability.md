---
type: agent_wiki_node
module: crates::engine
scope: [engine::debug, engine::controller, engine::interpreter, engine::action]
topics: [observability, telemetry, debugging, events, trace, diagnostics]
associated_files:
  - crates/engine/src/debug/mod.rs
  - crates/engine/src/debug/low.rs
  - crates/engine/src/debug/medium.rs
  - crates/engine/src/debug/high.rs
  - crates/engine/src/debug/save.rs
  - crates/engine/src/controller/mod.rs
  - crates/engine/src/controller/trace_logger.rs
  - crates/engine/src/interpreter/trace.rs
  - crates/engine/src/action.rs
last_validated: 2026-08-10
---

# Observability & Diagnostics

The engine has **no `tracing`/`log` integration** (verified: zero imports of either in
`crates/engine/src`). Telemetry is provided by four mechanisms, three legacy plus one new:

1. A reactive `event_sender` callback (per-loop-iteration `&GameData` snapshot) — §1.
2. A new per-FSM-transition `trace_sender` callback emitting `TraceEntry` values — §2.
3. The `MCG_TRACE_LOG` trace file produced by `TraceLogger` (opt-in — see §3).
4. The structured `crates::engine::debug` formatter/dumps (`DebugLevel` Low/Medium/High) — §4.

plus a single ad-hoc `eprintln!` (§5). For the error channels themselves, see
[`error-handling.md`](./error-handling.md).

---

## 1. Reactive Event Callback (`event_sender`)

```rust
// crates/engine/src/controller/mod.rs:35
event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
```

`crates::engine::controller::run_game` accepts it as its 4th argument
(`crates/engine/src/controller/mod.rs:31-37`). It is invoked by
`crates::engine::controller::Controller::emit_event`
(`crates/engine/src/controller/mod.rs:290-294`) at the **top of every loop iteration**
(`mod.rs:153`) *and* once more just before returning `GameOver` (`mod.rs:163`). The callback
receives `&GameData`, so a host can render or snapshot after every single transition without
polling. This is the recommended coarse-grained observability seam for production hosts that want
per-loop state snapshots — note this is per-loop-iteration, *not* per-FSM-transition; the
fine-grained per-transition seam is the `trace_sender` of §2.

> The callback receives a shared `&GameData` — it must not mutate through it. Hosts that need a
> snapshot must `clone()`. The bound is `Fn(&GameData) + Send` but **not** `Sync` — see
> [`interfaces.md`](./interfaces.md) §6.2 for multi-thread host implications.

---

## 2. Per-Transition Trace Sender (`trace_sender`) — post-Stage-5

```rust
// crates/engine/src/controller/mod.rs:36
trace_sender: Option<Box<dyn Fn(TraceEntry) + Send>>,
```

`run_game`'s **5th** argument. Unlike `event_sender` (which is per-loop-iteration and gets the
`&GameData`), `trace_sender` is invoked **once per FSM transition** (i.e. once per `step()` that
returns `Ok`/`NeedsInput`/`GameOver`) with a `crates::engine::interpreter::TraceEntry`. It is the
recommended **structured logging seam**: a host that wants per-edge detail (action subtype, choice
index, condition results, quantifier branch taken) implements one closure and gets every transition
without instrumenting the interpreter's interior.

### 2.1 `TraceEntry`

```rust
// crates/engine/src/interpreter/trace.rs:1-8
pub enum TraceEntry {
    Step { from: u32, to: u32, event: TraceEvent },
}
```

`from`/`to` are **raw** `StateID` integers (via `StateID::raw()`), so they include the synthetic
ids the quantifier subsystem allocates from `u32::MAX - 1` downward (see invariant I-16 in
[`invariants.md`](./invariants.md)).

### 2.2 `TraceEvent`

```rust
// crates/engine/src/interpreter/trace.rs:10-52
pub enum TraceEvent {
    Action { subtype: String, detail: String, raw_detail: String },
    Choice { chosen_idx: usize, options: Vec<String> },
    OptionalAccept,
    OptionalDecline,
    Condition { expr: String, raw_expr: String, result: bool, negated: bool, took_else: bool },
    EndCondition { expr: String, raw_expr: String, result: bool, stage: String, exited: bool },
    StageRoundCounter { stage: String, new_count: u32 },
    EndStage { stage: String },
    Trigger,
    Quantifier { kind: String, detail: String },
}
```

`detail`/`expr` carry **DSL-level text** (e.g. `deal 1 from Deck private to
Hand of P:P1`, `(sum of Hand of current using BJ > 21)`) — the AST `Display`
implementations; `raw_detail`/`raw_expr` carry the `Debug` representations.
`TraceEvent::pretty()` renders the simplified form (this is what `Display`
uses, so the trace file is readable); `TraceEvent::raw()` renders the `Debug`
form — the TUI toggles between them with `r`.

Each variant is emitted by exactly one arm of `Interpreter::step`
(`crates/engine/src/interpreter/mod.rs:64-364`) or by the quantifier driver
(`crates/engine/src/interpreter/quant_driver.rs:363-375`):
- `Action` — `Payload::Action` arm (`interpreter/mod.rs:154-171`) and the overlay-dispatch branch
  (`interpreter/mod.rs:91-100`); `subtype`/`detail` come from `rule_signature`
  (`ir_ext.rs:28-96`).
- `Choice` — `Payload::Choice` arm (`interpreter/mod.rs:173-194`).
- `OptionalAccept`/`OptionalDecline` — `Payload::Optional` arm (`interpreter/mod.rs:195-228`).
- `Condition` — `Payload::Condition` arm (`interpreter/mod.rs:229-265`).
- `EndCondition` — `Payload::EndCondition` arm (`interpreter/mod.rs:266-314`).
- `StageRoundCounter` — `Payload::StageRoundCounter` arm (`interpreter/mod.rs:315-331`).
- `EndStage` — `Payload::EndStage` arm (`interpreter/mod.rs:332-345`).
- `Trigger` — `Payload::Trigger` arm (`interpreter/mod.rs:346-356`).
- `Quantifier` — quantifier initial prompt, resume, and fan-out arms
  (`interpreter/quant_driver.rs:128,174,202,230,268,326`).

### 2.3 Display

Both `TraceEntry` and `TraceEvent` implement `std::fmt::Display`
(`crates/engine/src/interpreter/trace.rs:48-102`) — `TraceEntry` formats as
`[{from}->{to}] {event}`, so a one-line trace readout is `format!("{}", entry)`. This is what the
file logger (§3) writes per step.

---

## 3. The Trace File: `MCG_TRACE_LOG` / `with_log_path`

`crates::engine::controller::run_game_with` resolves a log path at startup via the private
`trace_logger::resolve_log_path` (`crates/engine/src/controller/trace_logger.rs`). Precedence:

```text
RunOptions::with_log_path(p)  → p (wins over everything)
MCG_TRACE_LOG env var:
  "" / "off" / "none" (case-insensitive) → disabled
  any other string                      → used verbatim as the path
neither set                           → disabled (no file is ever created
                                         in the working directory by default)
```

The `cgdsl-play` binary exposes this as `cgdsl-play --log <path>` (its trace header is also
tagged with the game-file name); interactive users can just set `MCG_TRACE_LOG=path`.

If a path was resolved and `TraceLogger::open` succeeds, `run_game_with` writes the following
structured lines:

- **Header** (written before the run loop):
  ```
  === MCG Trace Log ===
  Version: <CARGO_PKG_VERSION> (cgdsl-engine)
  Started: <YYYY-MM-DD HH:MM:SS UTC>
  Game: <game_name>            ← only when RunOptions::with_game_name was set
  Entry: <{:?} of ir.entry.raw()>
  Goal: <{:?} of ir.goal.raw()>
  Input source: <"interactive" or the test file path>
  ====================
  ```
  The timestamp is UTC and dependency-free (`trace_logger.rs::format_timestamp`, Hinnant's
  `civil_from_days`); the version line is `env!("CARGO_PKG_VERSION")`.
- **Per-step lines** (one per `TraceEntry`):
  ```
  [Step NNN] <TraceEntry Display>
  ```
  where `NNN` is the controller's `step_count` (loop iterations), shared with the composed
  sender closure.
- **Footer**:
  - `=== GameOver after N steps ===` on success, or `=== Error: <e> (after N steps) ===` on
    `Err` — `N` is the total loop-iteration count.
- **Panic line**: `=== Panic: <msg> (after N steps) ===` written before the panic is converted
  (`capture_panics(true)`) or re-raised; see §3.2 and [`error-handling.md`](./error-handling.md) §2.

`TraceLogger` itself stores `Arc<Mutex<BufWriter<File>>>` (`trace_logger.rs:10`) so both the
composed sender closure (handed to `Interpreter`) and the post-run footer writes go through one
writer; it is `Clone` (cheap `Arc` clone).

### 3.1 Caller `trace_sender` + file logger composition

When the caller also passes a `trace_sender` (`run_game`'s 5th arg), `run_game` does **not** make
the caller's closure and the file logger race for the same `TraceEntry`. Instead it composes them
into a single `Box<dyn Fn(TraceEntry) + Send>` (`crates/engine/src/controller/mod.rs:71-84`) that
first logs to the file (if open) and then forwards to the caller's closure (if present). The
interpreter only sees the composed sender; it does not know whether one or both backends exist.
Hosts therefore do **not** need to duplicate the file logging themselves — passing a `trace_sender`
for live structured observation plus letting `MCG_TRACE_LOG` write the file is a fully supported
combination.

### 3.2 Panic capture

Panic capture is governed by two conditions (`crates/engine/src/controller/mod.rs`):

- **`RunOptions::capture_panics(true)`** — the run is always wrapped in
  `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| controller.run()))`. On panic, the
  message is logged as `=== Panic: <msg> ===` to the trace file if one is open, and the panic is
  **converted** to `Err(EngineError::InternalPanic { message })` — the host process does not
  abort. This is the recommended setting for embedded hosts (Mode B, servers, the TUI).
- **Legacy default (`capture_panics(false)`)** — the run is wrapped *only* when a trace log is
  open, purely to log the panic before `std::panic::resume_unwind(payload)` re-panics in the
  caller's thread. Net effect: **the panic surfaces to the caller AFTER being logged**. Without a
  trace log, `run_game` calls `controller.run()` directly with no `catch_unwind`; the panic
  propagates exactly as before.

---

## 4. `crates::engine::debug` — Structured Human-Readable Dumps

Three verbosity levels selectable via `crates::engine::debug::DebugLevel`
(`crates/engine/src/debug/mod.rs:13-37`):

```rust
pub enum DebugLevel { Low, Medium, High }
impl DebugLevel {
    pub fn from_marker(s: &str) -> Option<Self>;   // parses "<!--LOW-->" etc., case-insensitive
    pub fn marker(&self) -> &'static str;          // inverse
}
pub fn format_game_data(data: &GameData, level: DebugLevel) -> String;   // mod.rs:38-44
pub fn format_game_data(data: &GameData, level: DebugLevel) -> String;   // mod.rs:38-44
pub fn save_game_data(data: &GameData, path: &Path) -> io::Result<()>;   // save.rs:8-24
```

The level-specific formatters live in dedicated submodules:

| Level | Implementation | Includes |
|---|---|---|
| `crates::engine::debug::DebugLevel::Low` | `crates/engine/src/debug/low.rs` (`format_game_data_low`) | Player names, current player, current stage, turn-order indices, card count per location. |
| `crates::engine::debug::DebugLevel::Medium` | `crates/engine/src/debug/medium.rs` (`format_game_data_medium`) | All of Low + per-player scores, teams, all memories, per-location card names (first 5 if >5). |
| `crates::engine::debug::DebugLevel::High` | `crates/engine/src/debug/high.rs` (`format_game_data_high`) | Full dump: every player with `in_stage` map, teams with raw indices, all locations with raw card-id vecs, every card's full attribute map, stage stack, all stage counters, all memories (typed), combos, precedences, point maps. |

`crates::engine::debug::save_game_data` (`crates/engine/src/debug/save.rs:8-24`) **appends** to
`path` (creating it if absent) and **persists the `DebugLevel`** as an HTML comment marker on the
file's first line: on subsequent calls it reads the existing first line via
`crates::engine::debug::DebugLevel::from_marker` and reuses that level (defaulting to `Medium` if
absent/unparseable). This round-trip (`DebugLevel::marker` ↔ `DebugLevel::from_marker`) is tested
in `crates/engine/src/debug/tests.rs` (`from_marker` cases at lines 8-37, marker round-trip at
41-45) — lets a log file self-describe its verbosity.

---

## 5. Ad-hoc stderr

There are **no** `eprintln!`/`println!`/`dbg!` calls left in the production library: the former
`eprintln!("ShuffleAction failed: {}", e)` was converted to a recoverable `Err` on 2026-08-09
(see `engine-vs-design.md` F-8). The `println!`s in `crates/engine/src/bin/cgdsl-play.rs` are
CLI output, not engine telemetry. One `eprintln!` remains in the `run_game` startup path: when
`MCG_TRACE_LOG` resolves to a path but `TraceLogger::open` fails
(`crates/engine/src/controller/mod.rs:43-49`), a `Warning: failed to open trace log ...` message
goes to stderr and the run continues without file logging — this is intentional grace, not a
panic.

---

## 6. Agent Guidance for Adding Telemetry

Two seams are now sanctioned:

1. **`event_sender`** (per loop iteration, with `&GameData`) — coarse-grained; use for rendering or
   taking state snapshots after every transition.
2. **`trace_sender`** (per FSM transition, with `TraceEntry`) — fine-grained and structured; the
   recommended seam for adding new telemetry. Adding a new `TraceEvent` variant is a one-edit
   change to `crates/engine/src/interpreter/trace.rs` plus emission at the relevant branch of
   `Interpreter::step` (or `quant_driver.rs`).

Prefer extending the `crates::engine::debug` level-gated formatter or hooking one of these two
seams rather than sprinkling `println!`/`eprintln!` through
`crates/engine/src/action.rs`/`crates/engine/src/query/`/`crates/engine/src/interpreter/`. If true
structured logging is desired, `tracing` would need to be added to `crates/engine/Cargo.toml` (it
is not currently a dependency) and spans established around
`crates::engine::interpreter::Interpreter::step` /
`crates::engine::action::execute` /
`crates::engine::query::Evaluator::eval_*`; do not assume spans already exist.
