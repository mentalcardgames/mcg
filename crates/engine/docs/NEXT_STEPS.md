---
type: agent_wiki_node
module: crates::engine
scope: [all — future work]
topics: [future-work, project-ideas, handoff]
last_validated: 2026-08-09
---

# Next Steps — Project Seeds

> One-paragraph seeds for future bachelor's/master's projects, grouped by area.
> See `engine-vs-design.md` for the numbered divergence/bug references (D-n, P-n)
> and `dsl-completeness.md` for the construct status table.

## Engine correctness (small-to-medium projects)

- **~~Graceful error handling instead of panics~~** — DONE 2026-08-09: `action::execute`
  and `Interpreter::execute_edge` are fallible; every DSL-reachable panic site
  (incl. `cycle to next`, `SetMemory`/`ResetMemory`) now returns
  `StepResult::Error`. Remaining work: make `end turn` with nobody eligible a
  recoverable error too (currently silently sets `current_player = None`), and
  remove the two internal `GameData` panics (`add_location`, `next_player`) by
  making those methods fallible.
  > **Update 2026-08-10:** `end turn`/`cycle to next` with nobody eligible is
  > now a **no-op** + stage auto-end (F-16, I-24), so the strand is gone; the
  > two internal `GameData` panics remain (unreachable from DSL).
- **~~Collection-memory aggregation~~** — DONE 2026-08-09 (the four `todo!()` arms
  plus the silent-empty `PlayerCollection` memory reads). Follow-up: `SetMemory`
  still inserts typed empty defaults for collection types — evaluate the actual
  collections once the grammar supports them.
  > **Update 2026-08-10:** collection `SetMemory` writes and setup-time
  collection memory declarations are fully evaluated (F-19).
- **~~Combo semantics~~** — DONE 2026-08-09 (group-wise evaluation). Follow-up:
  pin straight/flush detection patterns (combos + filters) with fixtures.
- **Card status / tokens (D-6).** The `card_statuses` slot now exists in
  `GameData`; implement `FlipAction` (de/encrypt a card's face) and
  `Place`/tokens with card cryptography. Prerequisite for face-down mechanics
  and privacy-aware play.
- **Demand semantics (D-7).** `bid` gained the numeric-input semantics
  (2026-08-10, F-26); `demand` still needs a written spec before it can be
  implemented. Good project for a thesis that starts from game-theory
  requirements.

## DSL / parser work (front_end)

- **Close the grammar gaps.** Mandatory `of <owner>` on memory reads and
  `SetMemory`/`ResetMemory` (P-4, P-5 — bridged engine-side 2026-08-10, F-21);
  `optional { ... } else { ... }` (D-3); `end stage when <bool>` mid-body exit
  (D-2); remove the unused `create` keyword or make it meaningful (P-6).
- **Implement `for <players>` and SimStage.** Carry the stage participant
  collection into the IR (P-1) and build true parallel sub-FSMs for `sim`
  stages (P-2) — the largest DSL-semantics project.
- **Parse-level ergonomics.** Fix the PEG parens quirks (P-8) and add a
  strict-mode validator that warns on constructs that are silently dropped
  (status keywords, `for` clause).
- **Numeric prompts in pure int slots.** `InputType::Number` exists
  (2026-08-10, F-26) behind `bid <qty> on <memory> of <owner>` and the
  `deal any`/`deal >= M and <= N` count prompt (F-28); `any` in *arbitrary*
  int expressions (`score any to …`, `M is any`) still needs grammar work.
  "Pick exactly one card" is now simply `move 1 from …` (F-28).
- **Parser stack scaling (found 2026-08-10).** `pest`'s recursion cost grows
  with the number of flow components — roughly a 0.8 MiB base plus
  ~10-20 KiB per component in debug builds — so games with many options
  (Go Fish's 13-option asks) overflow the 1 MiB OS-default main-thread stack
  during parsing. The engine binaries work around it by running the driver
  on a 16 MiB thread (`cgdsl-play`/`engine-tui` `DRIVER_STACK_BYTES`); the
  real fix is a parser-side stack reduction or an explicit stack policy.

## The P2P game (large projects, masters-level)

- **Mental-card-game cryptography.** Implement ZKP-based shuffling/drawing
  (mental poker) for the `.cgdsl` engine — card dealing is already a single
  engine action (`deal`), so the crypto can sit behind one seam. Companion to
  `docs/FUTURE_WORK_AND_MERGE.md` §3.
- **Wire the engine into a network layer.** The live server (`native_mcg`)
  still runs its own poker engine; the workspace intent is P2P play where each
  player runs a backend. Project: replace/augment `native_mcg`'s game module
  with `cgdsl-engine` behind a WebSocket/iroh transport, mapping `InputType`
  prompts to network messages.
- **Node discovery & trust.** Lobby/DHT discovery (iroh), signed messages,
  emoji-hash key verification (see `docs/FUTURE_WORK_AND_MERGE.md` §2, §4).
- **External UI on Mode B.** Build the GUI the engine docs anticipate
  (`interfaces.md` §4): drive `Interpreter::step` from a UI crate, render
  `GameData` + `TraceEntry`s, and answer `InputType` prompts — the frontend
  (`frontend/`) already renders cards for the poker engine.

## Testing / tooling

- **Injectable engine RNG.** `ShuffleAction`/`CreateTurnorderRandom` use
  `rand::thread_rng()`; a seeded RNG (via `run_game` argument or
  `GameData` field) would make games fully reproducible — enabling golden
  replays and deterministic random-input tests (currently only the input
  sequence is seed-reproducible; see `tests/random_play_test.rs`).
- **Property-based DSL tests.** `front_end` already ships proptest
  generators; extend them to the engine (arbitrary `GameData` + IR, assert
  invariants I-1..I-23 hold after every `step()`).
- **Coverage + trace-diff tooling.** Wire `cargo-llvm-cov`; add a golden-trace
  harness that diffs `TraceEntry` streams across engine changes (protects the
  TUI and future UI consumers).

## Suggested order of work

1. Graceful error handling (D-1 + `SetMemory`/`ResetMemory` panics) — smallest,
   unblocks safe game authoring.
2. `optional else` + `end stage when` (D-3, D-2) — unblocks faithful game rules.
3. Card status/tokens (D-6) — unblocks face-down mechanics.
4. P2P crypto + network wiring — the flagship masters projects, built on 1-3.
