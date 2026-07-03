---
type: agent_wiki_node
module: crates::engine
scope: [front_end / parser / DSL bugs affecting engine behaviour]
topics: [known-bugs, parser, DSL, front_end, player-collection, simstage, seqstage]
last_validated: 2026-07-03
---

# Known Bugs — Frontend/DSL Bugs Affecting Engine Behaviour

This page documents bugs that live in the `front_end` crate (parser, AST, IR lowering) but whose
effects manifest at engine runtime. Engine-only changes cannot fix them; they require changes to
`crates/front_end`. Each entry describes the DSL syntax involved, what the author probably
intended, what actually happens, and the relevant code locations.

---

## B-1: `for Y` player collection is dropped during IR lowering

**Severity:** Medium — silently wrong, no error, no warning.

### DSL syntax affected

```cgdsl
stage Play for current 2 times { ... }    -- SeqStage, player_expr
stage Play for all 2 times { ... }         -- SimStage, PlayerCollection (Aggregate)
stage Play for (P:P1, P:P2) 2 times { ... }  -- SimStage, PlayerCollection (Literal)
stage Play for others 2 times { ... }      -- SimStage, PlayerCollection (Runtime::Others)
```

Any `for Y` clause where `Y` is anything other than `current`.

### What the author probably intended

- `for current` → only the "current" (active) player participates in this stage.
- `for all` → all players participate simultaneously.
- `for (P:P1, P:P2)` → only P1 and P2 participate.
- `for others` → all players except the current one participate.

### What actually happens

**All players are marked in-stage regardless of `Y`.** The `for Y` collection is parsed into the AST
(`SeqStage.player: PlayerExpr` or `SimStage.players: PlayerCollection`) but is **dropped during IR
lowering** — `build_seq_stage` and `build_sim_stage` (`crates/front_end/src/ir.rs:565` and `:654`)
both produce the same IR shape (EndCondition-at-entry + StageRoundCounter back-edge) without reading
`stage.player` or `stage.players`. The stage name alone is encoded in `Payload::EndCondition` /
`Payload::StageRoundCounter`. The engine's `ensure_stage_entered` (`crates/engine/src/game_data.rs:234`)
has no participant information available and marks **all** players `in_stage[stage] = true`.

The two stage types are also not differentiated at runtime:
- `SeqStage` (sequential, `for current`) and `SimStage` (simultaneous, `for <collection>`) both lower
  to the same sequential IR shape. There is no "simultaneous" execution — all flows run single-threaded
  in the interpreter regardless.

### Why no error is raised

The IR lowering never attempts to use the `player`/`players` field, so the mismatch is silent. The
game runs to completion, but with all players in-stage instead of the intended subset.

### Relevant code

| File | Lines | Role |
|---|---|---|
| `crates/front_end/src/grammar.pest` | 636, 641 | `sim_stage` and `seq_stage` grammar rules |
| `crates/front_end/src/parser.rs` | 76–101 | `seq_stage` and `sim_stage` parser rules |
| `crates/front_end/src/ast.rs` | 1631, 1649 | `SeqStage` (`player: PlayerExpr`) and `SimStage` (`players: PlayerCollection`) |
| `crates/front_end/src/ir.rs` | 565–641 | `build_seq_stage` — **does not read `stage.player`** |
| `crates/front_end/src/ir.rs` | 654–730 | `build_sim_stage` — **does not read `stage.players`**; comment at `:648` confirms TODO |
| `crates/front_end/src/ir.rs` | 598–620, 686–709 | `EndCondition`-at-entry edges only carry `stage: SID`, not participant list |
| `crates/engine/src/game_data.rs` | 234–239 | `ensure_stage_entered` marks all players in-stage (no participant info available) |
| `crates/engine/src/query.rs` | 1607–1664 | `resolve_player_collection` — would handle `PlayerCollection` at runtime but is never called for stage participation |

### Partial mitigation

`end <stage>` / `out of` (`ActionRule::OutAction`, `crates/engine/src/action.rs:186–194`) removes
specific players from `in_stage` after entry. So a game author *can* express "all except X" by
entering all and then using `end Play for P:P1` as the first flow action. However:
- `end <stage>` pops the entire stage stack for that stage (I-11), which may not be the intended
  semantics for nested stages.
- There is no way to express "only player X participates" without also relying on turn-order
  discipline (having other players' turns be no-ops).

### Fix requires

Front_end changes to carry the participant collection into the IR, e.g.:
- Add `players: PlayerCollection` field to `Payload::EndCondition` / `Payload::StageRoundCounter`.
- Emit separate IR edges per player for `SimStage` (the stated TODO at `ir.rs:648`).
- Or: emit an `enter_stage(stage, participants)` action at the stage entry edge instead of relying
  on the interpreter to call `ensure_stage_entered` blindly.

---

## B-2: `PlayerCollection::Aggregate` (all/any quantifier) is not implemented

**Severity:** Low — unreachable in practice due to B-1.

### DSL syntax affected

```cgdsl
stage Play for all 2 times { ... }   -- PlayerCollection::Aggregate { Quantifier::All }
stage Play for any 2 times { ... }   -- PlayerCollection::Aggregate { Quantifier::Any }
```

### What the author probably intended

`for all` means all in-game players participate in the stage simultaneously. `for any` is less clear but
would presumably mean "at least one player participates."

### What actually happens

Even if B-1 were fixed, `resolve_player_collection` (`crates/engine/src/query.rs:1625–1627`) hits:

```rust
PlayerCollection::Aggregate { .. } => {
    todo!("PlayerCollection::Aggregate not yet implemented")
}
```

This code path is **never reached** for stage participation because the `PlayerCollection` is dropped
during IR lowering (B-1). It is reachable only for `end game with winner(for all ...)` or
`OutOfPlayer { players: for all, out_of: ... }` expressions, which would panic at runtime if a game
author wrote them.

### Relevant code

| File | Lines | Role |
|---|---|---|
| `crates/front_end/src/grammar.pest` | 426–429 | `quantifier = { kw_all \| kw_any }` |
| `crates/front_end/src/ast.rs` | 1167–1170 | `AggregatePlayerCollection::Quantifier { Quantifier::All \| Any }` |
| `crates/front_end/src/ast.rs` | 1185–1198 | `PlayerCollection::Aggregate` variant |
| `crates/engine/src/query.rs` | 1625–1627 | `todo!()` — unreachable for stage participation (B-1), but reachable for winner/OutOfPlayer |

### Fix requires

Frontend: implement `Quantifier::All`/`Any` in `resolve_player_collection`. For `all`, return all
in-game players; for `any`, the semantics need clarification (likely "all in-game players" is
intended since cgdsl stages don't meaningfully support "at least one"). This is blocked on B-1
fixing first since the IR must carry the `PlayerCollection` to the engine for this to be reachable.

---

## B-3: `SeqStage` and `SimStage` produce identical sequential IR

**Severity:** Low — silently wrong, no error.

### DSL syntax affected

```cgdsl
stage Play for current 2 times { ... }   -- SeqStage (sequential turns)
stage Play for all 2 times { ... }        -- SimStage (simultaneous)
```

### What the author probably intended

- `SeqStage` (`for current`): one player acts at a time; the stage iterates once per eligible player
  in turn order. The body of the stage runs as a "while current player can act" loop.
- `SimStage` (`for all` or any player collection): all players in the collection act simultaneously
  (or in parallel sub-flows); the stage ends when all have satisfied its end condition.

### What actually happens

Both lower to the **same sequential IR**: a single flow of `EndCondition`-at-entry + `StageRoundCounter`
back-edge, executed single-threaded. There is no simultaneous/parallet execution — it's purely
sequential, one FSM step at a time.

The `SimStage` TODO comment at `ir.rs:648` explicitly states this:

> `/// TODO: SimStage is not implemented!`
> `/// Additionally we need to spawn a sub graph for each player and each edge needs to check
> /// if no player has finished the stage already (or any other condition).`

### Relevant code

| File | Lines | Role |
|---|---|---|
| `crates/front_end/src/ir.rs` | 565–641 | `build_seq_stage` |
| `crates/front_end/src/ir.rs` | 654–730 | `build_sim_stage` — identical to `build_seq_stage` |
| `crates/front_end/src/ast.rs` | 1627–1640 | `SeqStage` doc comment says "single player … deterministic and synchronous, typically representing a turn" |
| `crates/front_end/src/ast.rs` | 1642–1658 | `SimStage` doc comment says "concurrent … multiple players act simultaneously" |

### Fix requires

Frontend: `build_sim_stage` needs a fundamentally different IR shape — either:
- Parallel sub-FSMs per player, with an aggregation step to check "all done"
- Or a `SimStage`-specific payload that coordinates per-player iteration in the engine

This is closely tied to B-1 (the participant collection must be available in the IR).

---

*Last updated: 2026-07-03. Add new entries above. Each bug gets a `B-N` designation (B for
"backend/frontend bug", distinct from invariant `I-N` in `invariants.md`).*
