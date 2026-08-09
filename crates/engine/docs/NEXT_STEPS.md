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

- **Graceful error handling instead of panics.** Replace the remaining panic
  sites (D-1 `cycle to next`, `SetMemory`/`ResetMemory` without a current
  player, `CreateLocation`/`CreateCardOnLocation` setup failures) with
  recoverable `StepResult::Error`s by moving fallible resolution into the
  interpreter. Small, well-scoped, high value.
- **Collection-memory aggregation.** Implement the four `todo!()` evaluator
  arms (D-4) so `size`/`sum`/`at` work over multi-owner memories — unlocks
  team and cross-player bookkeeping.
- **Combo semantics.** Fix per-card `same`/`distinct` combo matching (D-5) and
  pin combo behaviour with fixtures.
- **Card status / tokens.** Add a card-status map to `GameData`, implement
  `FlipAction` and `Place`/tokens (D-6) — prerequisite for face-down
  mechanics and privacy-aware play.
- **Bidding/demand.** Write the semantic spec first, then implement
  `BidAction`/`DemandAction` (D-7). Good project for a thesis that starts from
  game-theory requirements.

## DSL / parser work (front_end)

- **Close the grammar gaps.** Mandatory `of <owner>` on memory reads and
  `SetMemory`/`ResetMemory` (P-4, P-5); `optional { ... } else { ... }`
  (D-3); `end stage when <bool>` mid-body exit (D-2); remove the unused
  `create` keyword or make it meaningful (P-6).
- **Implement `for <players>` and SimStage.** Carry the stage participant
  collection into the IR (P-1) and build true parallel sub-FSMs for `sim`
  stages (P-2) — the largest DSL-semantics project.
- **Parse-level ergonomics.** Fix the PEG parens quirks (P-8) and add a
  strict-mode validator that warns on constructs that are silently dropped
  (status keywords, `for` clause).

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
