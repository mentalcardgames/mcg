---
type: agent_wiki_node
module: crates::engine
scope: [all — future work]
topics: [future-work, project-ideas, handoff]
last_validated: 2026-08-11
---

# Next Steps — Project Seeds

> One-paragraph seeds for future bachelor's/master's projects, grouped by area.
> See `engine-vs-design.md` for the numbered divergence references (D-n, P-n)
> and `dsl-completeness.md` for the construct status table.

## Engine correctness (small-to-medium projects)

- **Card status / tokens (D-6).** The `card_statuses` slot exists in
  `GameData`; implement `FlipAction` (de/encrypt a card's face) and
  `Place`/tokens with card cryptography. Prerequisite for face-down mechanics
  and privacy-aware play.
- **Demand semantics (D-7).** `bid` has defined semantics (the numeric input
  prompt); `demand` still needs a written spec before it can be implemented.
  Good project for a thesis that starts from game-theory requirements.
- **Remove the two internal `GameData` panics** (`add_location`,
  `next_player`) by making those methods fallible. Both are unreachable from
  well-formed DSL input today, but the panics remain.
- **Combo read-side filters (D-16).** Lay-down moves validate correctly;
  the read-side (`size(cards Set in Hand)`) still over-approximates. Pin
  straight/flush detection patterns with fixtures, then add per-group filter
  atoms.

## DSL / parser work (front_end)

- **`optional { ... } else { ... }` (D-3).** The engine's `Optional` arm
  already dispatches on `input.idx()`; the IR needs a third edge (body /
  else-body / exit).
- **`end stage when <bool>` mid-body exit (D-2).** The engine's `EndCondition`
  arm already implements the exit; the IR builder only emits `EndCondition` at
  stage *entry* today.
- **`for <players>` participation (P-1) / SimStage (P-2).** Carry the stage
  participant collection into the IR and gate `ensure_stage_entered` on it;
  SimStage additionally needs per-player sub-FSMs — the largest DSL-semantics
  project.
- **Close the grammar gaps.** Mandatory `of <owner>` on memory writes (P-4/P-5
  — currently bridged engine-side); remove the unused `create` keyword (P-6).
- **Parse-level ergonomics.** Fix the PEG parens quirks (P-8) and add a
  strict-mode validator that warns on constructs that are silently dropped
  (status keywords, `for` clause).
- **Numeric prompts in pure int slots.** `InputType::Number` exists (behind
  `bid … on <memory> of <owner>` and the `deal` count prompt); `any` in
  *arbitrary* int expressions (`score any to …`, `M is any`) still needs
  grammar work. "Pick exactly one card" is already `move 1 from …`.
- **Parser stack scaling.** `pest`'s recursion cost grows with the number of
  flow components — roughly a 0.8 MiB base plus ~10-20 KiB per component in
  debug builds — so games with many options (Go Fish's 13-option asks)
  overflow the 1 MiB OS-default main-thread stack during parsing. The engine
  binaries work around it by running the driver on a 16 MiB thread
  (`cgdsl-play`/`engine-tui` `DRIVER_STACK_BYTES`); the real fix is a
  parser-side stack reduction or an explicit stack policy.

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
  invariants I-1..I-25 hold after every `step()`).
- **Coverage + trace-diff tooling.** Wire `cargo-llvm-cov`; add a golden-trace
  harness that diffs `TraceEntry` streams across engine changes (protects the
  TUI and future UI consumers).

## Suggested order of work

1. `optional else` + `end stage when` (D-3, D-2) — unblocks faithful game rules.
2. Card status/tokens (D-6) — unblocks face-down mechanics.
3. Combo read-side filters (D-16) — unblocks exact Rummy-style hand ranking.
4. P2P crypto + network wiring — the flagship masters projects, built on 1-3.
