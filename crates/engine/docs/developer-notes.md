---
type: agent_wiki_node
module: crates::engine
scope: [all]
topics: [design-decisions, notes]
last_validated: 2026-08-09
---

# Developer Notes

Design decisions and tribal knowledge for the next maintainer. **This page is
deliberately small** — status, bugs, and divergences have dedicated homes:

| Topic | Where it lives |
|---|---|
| Per-construct implementation status | [`dsl-completeness.md`](./dsl-completeness.md) |
| Known bugs & design divergences (with repros) | [`engine-vs-design.md`](./engine-vs-design.md) |
| Panic sites / recoverable errors / silent no-ops | [`error-handling.md`](./error-handling.md) |
| Guardrails (invariants I-1 … I-23) | [`invariants.md`](./invariants.md) |

---

## 1. Design Decisions

### 1.1 Memory is Owned and Scoped

Memory is the engine's name-value store (`GameData::memories: HashMap<String, MemoryValue>`).
Every memory slot is owned by an entity — a Player or the Table.
This ownership is encoded by **prefixing the key** in the flat HashMap:

| Owner | Prefix | Example key |
|-------|--------|-------------|
| Player `P1` | `"P1_"` | `"P1_M"` |
| Player `P2` | `"P2_"` | `"P2_score"` |
| Table | `"Table_"` | `"Table_pot"` |

**Setup time:** `CreateMemory` / `CreateMemoryWithMemoryType` resolve the `owner` to names and insert `"{owner}_{name}"`.

**Runtime reads:** All evaluator memory arms use `Evaluator::resolve_memory_key` / `resolve_collection_memory_key`. When the AST carries a `WithOwner` variant (`&I:M of P1`), the owner is used directly. The parser already supports `of <owner>` in all memory-reference grammar rules (`&I:M of ...`, `&P:M of ...`, etc.). The bare `Memory { memory }` variant (no owner) returns an error.

**Runtime writes (bridge):** `SetMemory` (`M is 42`) and `ResetMemory` (`reset M`) lack an owner clause in the grammar. As a bridge, they prefix the key with the current player name; without a current player they return a recoverable error. Each site is flagged with `// NOTE(grammar-gap)`. When the grammar adds `of <owner>` to these rules, the bridge code is replaced with the explicit owner from the AST (see `engine-vs-design.md` D-14).

**Why not a nested data structure?** `GameData` is `Clone` and the flat `HashMap` is serializable. Owner-prefixed keys need zero type-system changes — just string formatting at the access site, with guaranteed non-collision.

### 1.2 Scoring: ScoreMemory Semantics

`score N to M of P1` writes the evaluated int to memory slot `"P1_M"`. It does **not** affect `Player::score`. Only `score N to P1` (without a memory clause) mutates `Player::score`.

### 1.3 Scoring: WinnerWith::Position Interpretation

`winner is highest position` / `winner is lowest position` uses the player's index in `turn_order` (0-based). `turn_order [P2, P1, P3]` → P2=0, P1=1, P3=2. `highest position` → P3 wins. `lowest position` → P2 wins. Players not in `turn_order` get `usize::MAX`. This interpretation may not match the intended DSL semantics (see `engine-vs-design.md` D-10).

### 1.4 CGDSL Identifiers Must Start With Capital Letter

Per the Pest grammar `ident = { &capital ~ ... }`, names must start with a capital letter. Memory named `m` fails to parse; use `M`. This applies to: memory names, stage names, team names, combo names, precedence names, point map names, location names, and token names.

### 1.5 Card Creation Order is Deterministic (unless shuffled)

`expand_types` expands rank-major with the innermost dimension last (`Rank(Ace, Two) for Suit(D, C)` → `Ace-D, Ace-C, Two-D, Two-C`), and `deal N` takes the top N cards. **Without `shuffle`, the deck order is fully predictable** — the deterministic `behavior_*.cgdsl` fixtures rely on this (see `testing.md` §12). For exact per-player hands, use single-suit decks or comma-separated type groups.

### 1.6 The TUI Treats All Prompt Lists the Same

`Choice`, `ChoosePlayer`, and `ChooseCards` prompts share one interaction model in `engine-tui`: a cursor-highlighted list scrolled with ↑/↓ and confirmed with Enter, with digit shortcuts (1-9, 0 = option 10) for `Choice`. The windowing logic (`cursor_scroll_offset`) is unit-tested. The trace log has two rendering modes — simplified (DSL text) and raw (`Debug` output) — toggled with `r`; the four `TraceDetail` filter levels are unchanged.
