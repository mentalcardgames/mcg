---
type: agent_wiki_node
module: crates::engine
scope: [front_end / parser / DSL bugs affecting engine behaviour]
topics: [known-bugs, parser, DSL, front_end, player-collection, simstage, seqstage]
last_validated: 2026-07-04
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
`Payload::StageRoundCounter`. The engine's `ensure_stage_entered` (`crates/engine/src/game_data.rs:244`)
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
| `crates/engine/src/game_data.rs` | 244–250 | `ensure_stage_entered` marks all players in-stage (no participant info available) |
| `crates/engine/src/query/player.rs` | 227–284 | `resolve_player_collection` — would handle `PlayerCollection` at runtime but is never called for stage participation |

### Partial mitigation

`end <stage>` / `out of` (`ActionRule::OutAction`, `crates/engine/src/action.rs:183–198`) removes
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

## B-2: `PlayerCollection::Aggregate::Quantifier::Any` in setup is rejected; `All` resolved via preprocessor

**Severity:** Low — setup and Move-dest quantifiers are now handled; the `todo!()` remains for `winner`/`OutOfPlayer`.

### DSL syntax affected

```cgdsl
location Hand on all        -- Owner::PlayerCollection { Aggregate::Quantifier::All }
location Hand on any        -- Owner::PlayerCollection { Aggregate::Quantifier::Any }
turnorder all               -- PlayerCollection::Aggregate::Quantifier::All
turnorder any               -- PlayerCollection::Aggregate::Quantifier::Any
team T1 with all            -- CreateTeams with PlayerCollection::Aggregate::Quantifier::All
```

### What the author probably intended

`all` means all in-game players participate. `any` would mean "at least one player participates," but
setup-time `any` is almost certainly a DSL author mistake — there is no synchronous input path at
game init.

### What actually happens

`Quantifier::All` in setup rules now works correctly:

- `resolve_player_candidates` (`quantifier.rs:140-153`) intercepts `Aggregate::Quantifier::All`
  and returns all in-game players, bypassing the `todo!()` in `resolve_player_collection`.
- `execute_setup_rule` fans out per-player for `CreateLocation`, and directly assigns for
  `CreateTurnorder`/`CreateTurnorderRandom`/`CreateTeams`.

`Quantifier::Any` in setup rules is **rejected** before dispatch by `setup_contains_any`
(`crates/engine/src/quantifier.rs:170-185`) + the interpreter's setup-`Any` guard
(`crates/engine/src/interpreter/mod.rs:155-161`), returning
`StepResult::Error("quantifier 'any' is not supported in setup rules")` with no trace entry
and no `GameData` mutation (invariant I-20 in [`invariants.md`](./invariants.md)).

The **remaining `todo!()`** at `crates/engine/src/query/player.rs:246` is untouched — it is still
reachable for `end game with winner(for all ...)` or `OutOfPlayer { players: for all, ... }`
expressions (scope: B-1 / winner / OutOfPlayer). The engine's quantifier paths intercept
`Aggregate { Quantifier }` *before* reaching this `todo!()` (see
`crates/engine/src/quantifier.rs:140-153` `resolve_player_candidates`), so the engine's own
quantifier-driven moves never trigger it.

### Relevant code

| File | Lines | Role |
|---|---|---|
| `crates/engine/src/quantifier.rs` | 140–153 | `resolve_player_candidates` — handles `Aggregate::Quantifier::All` for setup and Move dests |
| `crates/engine/src/quantifier.rs` | 170–185 | `setup_contains_any` — predicate that detects `Any` in setup rules |
| `crates/engine/src/interpreter/mod.rs` | 155–161 | Interpreter setup-`Any` guard — returns `StepResult::Error` for setup `Any` |
| `crates/engine/src/query/player.rs` | 246 | `todo!("PlayerCollection::Aggregate not yet implemented")` — still reached for `winner(for all ...)` / `OutOfPlayer` (not fixed) |
| `crates/engine/src/query/player.rs` | 301–327 | `resolve_owner_to_names` — resolves `Owner::PlayerCollection` to multiple names via `quantifier::resolve_player_candidates` |
| `crates/engine/src/action.rs` | 91–107 | `CreateLocation` — fans out per owner name returned by `resolve_owner_to_names` |

### Fix requires

Setup and Move-dest quantifiers are handled. The remaining `todo!()` for `winner`/`OutOfPlayer` is
deferred (out of scope for this fix). No further engine changes required for setup-rule quantifiers.

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

*Last updated: 2026-07-04. Add new entries above. Each bug gets a `B-N` designation (B for
"backend/frontend bug", distinct from invariant `I-N` in `invariants.md`).*
