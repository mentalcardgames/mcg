---
type: agent_wiki_node
module: crates::engine
scope: [interpreter::types, controller, interpreter::mod, interpreter::quant_driver, TUI, cgdsl-play, tests]
topics: [input, player-fingerprinting, turn-gating, authentication]
last_validated: 2026-07-28
---

# Input Refactor: Player Fingerprinting & Turn Gating

## Goal

Every engine input carries a `player_id` identifying its submitter. The engine
validates that the submitter matches the current player before accepting input. The
TUI only enables interactive controls when viewing the current player's perspective.

---

## 1. New types (`crates/engine/src/interpreter/types.rs`)

```rust
/// An input submitted by a player.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    /// Name of the player who submitted this input (e.g. "P1").
    pub player_id: String,
    /// The kind of input selected.
    pub kind: InputKind,
}

/// The choice made by a player, without identity metadata.
#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
    Choice { idx: usize },
    OptionalAccept,
    OptionalDecline,
    /// Chosen player index (0-based into `InputType::ChoosePlayer::candidates`).
    ChoosePlayer { idx: usize },
    /// Chosen card indices (0-based into `InputType::ChooseCards::display`).
    ChooseCards { selected: Vec<usize> },
}
```

**Changes from current `Input` enum:**
- Old `Input` enum variants become `InputKind` variants.
- `Input` is now a struct wrapping `player_id: String` + `kind: InputKind`.
- Accessor methods `idx()`, `player_idx()`, `card_selection()` move to `InputKind`; `Input`
  adds thin delegates:

```rust
impl Input {
    pub fn idx(&self) -> usize            { self.kind.idx() }
    pub fn player_idx(&self) -> Option<usize> { self.kind.player_idx() }
    pub fn card_selection(&self) -> Option<&[usize]> { self.kind.card_selection() }
}
```

- `InputKind` retains the existing `idx()` → `0` default for non-choice variants
  (preserving compatibility with `Choice`/`Optional` dispatch which uses `input.idx()`).

---

## 2. Controller validation (`crates/engine/src/controller/mod.rs`)

### 2.1 `validate_player_input` — add player check

```rust
fn validate_player_input(
    input: &Input,
    input_type: &InputType,
    current_player_name: &str,
) -> bool {
    // Reject inputs from non-current players during active play
    if !current_player_name.is_empty() && input.player_id != current_player_name {
        return false;
    }
    // --- existing range/format checks unchanged below ---
    match (input, input_type) {
        (
            Input { kind: InputKind::Choice { idx }, .. },
            InputType::Choice { max_index, .. },
        ) => *idx <= *max_index,
        (
            Input { kind: InputKind::ChoosePlayer { idx }, .. },
            InputType::ChoosePlayer { candidates, .. },
        ) => *idx < candidates.len(),
        (
            Input { kind: InputKind::ChooseCards { selected }, .. },
            InputType::ChooseCards { display, min, max, .. },
        ) => {
            !selected.iter().any(|&i| i >= display.len())
                && selected.len() >= *min
                && selected.len() <= *max
        }
        _ => true,
    }
}
```

When `current_player` is `None` (pre-setup), `current_player_name` is empty → all inputs
accepted. After setup, only inputs from the current player pass.

### 2.2 `get_input` — pass current player name

```rust
fn get_input(&mut self, input_type: InputType) -> Result<Input, String> {
    let current_name = self
        .interpreter
        .game_data
        .get_current_player()
        .map(|p| p.name.as_str())
        .unwrap_or("");

    self.input_sequence += 1;
    let input = match &self.input_source {
        InputSource::Player(callback) => loop {
            let raw = callback(input_type.clone());
            if validate_player_input(&raw, &input_type, current_name) {
                break raw;
            }
        },
        InputSource::TestFile(path) => {
            let path = path.clone();
            self.read_test_file(&path)?
        }
    };
    Ok(input)
}
```

### 2.3 `read_test_file` — player name prefix

Each line now optionally starts with `PlayerName:`:

```
# Backwards compatible — no prefix defaults to "P1"
y
2
P2:1              ← P2 submitted choice #1
P3:c 1,3,5        ← P3 submitted cards 1,3,5
P1:p 2            ← P1 chose player candidate #2
```

Parser change — extract optional `Name:` prefix before parsing the body:

```rust
fn read_test_file(&mut self, path: &PathBuf) -> Result<Input, String> {
    // ... file loading unchanged ...

    let line = self
        .line_buffer
        .pop_front()
        .ok_or_else(|| format!("Test input file exhausted (input #{})", self.input_sequence))?;

    // Extract optional player prefix: "P2:y" → player_id="P2", body="y"
    let (player_id, body) = if let Some(colon) = line.find(':') {
        if colon > 0 && colon + 1 < line.len() {
            (
                line[..colon].to_string(),
                line[colon + 1..].trim_start().to_string(),
            )
        } else {
            ("P1".to_string(), line.clone())
        }
    } else {
        ("P1".to_string(), line.clone())
    };

    let lower = body.to_lowercase();
    let kind = if let Some(rest) = lower.strip_prefix("p ") {
        // ... parse p <N> → InputKind::ChoosePlayer { idx: n - 1 }
    } else if let Some(rest) = lower.strip_prefix("c ") {
        // ... parse c <csv> → InputKind::ChooseCards { selected }
    } else {
        match lower.as_str() {
            "y" | "yes" => InputKind::OptionalAccept,
            "n" | "no" => InputKind::OptionalDecline,
            _ => { /* parse <N> → InputKind::Choice { idx: n - 1 } */ }
        }
    }?; // existing error handling per parsing arm

    Ok(Input { player_id, kind })
}
```

---

## 3. Interpreter (`crates/engine/src/interpreter/`)

### 3.1 `mod.rs` — pattern match on `input.kind`

All existing `Input::Variant { ... }` patterns become `InputKind::Variant { ... }` accessed
via `input.kind`. Affected sections:

- `Payload::Choice` dispatch (line 173-194):
  `Input::Choice { idx }` → `InputKind::Choice { idx }` via `input.kind`
- `Payload::Optional` dispatch (line 195-228):
  `Input::OptionalAccept`/`OptionalDecline` → `InputKind::OptionalAccept`/`OptionalDecline`
  via `input.kind`

### 3.2 `quant_driver.rs` — quantifier resume matching

`take_quant_resume` matches input variant against pending kind (line 42-79). All
`Input::Variant { ... }` patterns become `InputKind::Variant { ... }` via `input.kind`:

```rust
Some(match (&pq.kind, input) {
    (
        PendingKind::DestPlayerAny { .. },
        Input { kind: InputKind::ChoosePlayer { idx }, .. },
    ) => Resume::Player { idx: *idx, ... },
    (
        PendingKind::CardsAnyOrRange { .. },
        Input { kind: InputKind::ChooseCards { selected }, .. },
    ) => Resume::Cards { selected: selected.clone(), ... },
    // ... etc
})
```

---

## 4. TUI (`crates/engine/src/bin/engine-tui/`)

### 4.1 `ui/state.rs` — track current player name

Add field:

```rust
pub current_player_name: String,
```

Initialize to `String::new()` in `new()`.

### 4.2 `main.rs` — extract current player from snapshots

In the `state_rx.try_recv()` loop:

```rust
while let Ok(gd) = state_rx.try_recv() {
    tui_state.current_player_name = gd
        .get_current_player()
        .map(|p| p.name.clone())
        .unwrap_or_default();
    tui_state.current_state = Some(gd);
    tui_state.game_state_auto_scroll = true;
}
```

### 4.3 `main.rs` — keyboard gating

Define a helper:

```rust
fn is_current_player(state: &TuiState) -> bool {
    let perspective_name = state
        .current_state
        .as_ref()
        .and_then(|gd| gd.players.get(state.perspective_idx))
        .map(|p| p.name.as_str())
        .unwrap_or("");
    !state.waiting_for_input
        || state.current_player_name.is_empty()
        || perspective_name == state.current_player_name
}
```

`is_current_player` returns `true` when:
- Not waiting for input (no prompt to gate)
- Pre-setup (current player unknown)
- Viewer IS the current player

**Modified keyboard handler structure:**

```
Key                     | Always works? | When not current player
------------------------+---------------+------------------------
q / F10 (quit)          | Yes           | —
l / t (detail)          | Yes           | —
p (perspective)         | Yes           | —
Tab (focus)             | Yes           | —
Arrows, PgUp/PgDn       | Yes           | Scroll, no cursor move
Home, End               | Yes           | Scroll
Space                   | No            | Blocked
Enter                   | No            | Blocked
y / n                   | No            | Blocked
1-9                     | No            | Blocked
```

When not current player, the Up/Down arrows fall through to `adjust_focused_scroll`
instead of moving the chooser cursor. All other navigation is unrestricted.

### 4.4 `main.rs` — include player_id in sends

All `tx.send(Input::...)` calls become `tx.send(Input { player_id, kind })`. The
`player_id` is derived from the perspective:

```rust
let player_name = tui_state
    .current_state
    .as_ref()
    .and_then(|gd| gd.players.get(tui_state.perspective_idx))
    .map(|p| p.name.clone())
    .unwrap_or_else(|| format!("Player{}", tui_state.perspective_idx));
```

This is computed once at the top of each keyboard-handler branch where input is sent.

### 4.5 `ui/input.rs` — render gating

`render()` gains `current_player_name: &str` parameter.

When `waiting` AND perspective ≠ current player:

```
┌─Perspective: P1 | WAITING FOR INPUT─────────────────┐
│                                                      │
│  Waiting for P2's turn... (you are viewing as P1)    │
│                                                      │
└──────────────────────────────────────────────────────┘
```

The perspective name is pulled from `player_names[perspective_idx]`.

Implementation — at the top of the `if waiting` block:

```rust
let perspective_name = &self.player_names[self.perspective_idx];
if !current_player_name.is_empty() && perspective_name != current_player_name {
    let msg = format!(
        "Waiting for {}'s turn... (you are viewing as {})",
        current_player_name, perspective_name
    );
    f.render_widget(Paragraph::new(msg), inner);
    return;
}
// else: normal prompt rendering
```

---

## 5. `cgdsl-play.rs`

No CLI argument. Use shared state updated by `event_sender`:

```rust
use std::sync::{Arc, Mutex};

fn main() {
    // ... parse, lower ...

    let player_name: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    // event_sender: update shared current-player name on every iteration
    let pn_writer = player_name.clone();
    let state_sender = Some(Box::new(move |gd: &GameData| {
        *pn_writer.lock().unwrap() = gd.get_current_player().map(|p| p.name.clone());
    }) as Box<dyn Fn(&GameData) + Send>);

    // input closure: read current player name, fall back to "P1" pre-setup
    let pn_reader = player_name.clone();
    let input_source = match input_file {
        Some(path) => InputSource::TestFile(PathBuf::from(path)),
        None => InputSource::Player(Box::new(move |it: InputType| {
            let name = pn_reader
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_else(|| "P1".to_string());
            interactive_input(it, &name)
        })),
    };

    run_game(ir, GameData::new(), input_source, state_sender, None)?;
}
```

`interactive_input` signature becomes:

```rust
fn interactive_input(input_type: InputType, player_name: &str) -> Input {
    // ... existing validation/parsing loops ...
    // each return wraps: Input { player_id: player_name.to_string(), kind: ... }
}
```

---

## 6. Test files — mechanical refactor

Every construction of `Input::Variant { ... }` becomes
`Input { player_id: "P1".into(), kind: InputKind::Variant { ... } }`.

| File | Est. changes |
|---|---|
| `crates/engine/src/interpreter/tests.rs` | ~40 sites |
| `crates/engine/src/interpreter/quant_driver_tests.rs` | ~10 sites |
| `crates/engine/src/interpreter/types_tests.rs` | ~5 sites |
| `crates/engine/tests/quantifier_test.rs` | ~15 sites |
| `crates/engine/tests/action_test.rs` | ~5 sites |
| `crates/engine/src/controller/tests.rs` | ~10 sites |

Also update `Display` impl for `TraceEntry` in `interpreter/trace.rs` — the `Choice`
variant accesses `chosen_idx: input.idx()`. No structural change needed since
`Input::idx()` delegates the same way as before.

---

## 7. Documentation

### 7.1 `invariants.md`

Add I-23 after I-22:

> **I-23 — Inputs are rejected if `player_id` does not match the current player.**
> `validate_player_input` (`crates/engine/src/controller/mod.rs`) checks
> `input.player_id == current_player_name` before any range validation. During active
> play, only the current player's inputs are accepted. Pre-setup
> (`current_player == None`), `current_player_name` is empty and all inputs pass. The
> check uses the player **name** (`Player::name: String`) for readability. The `Player`
> closure path re-prompts on rejection; the `TestFile` path parses `player_id` from an
> optional `Name:` prefix (defaulting to `"P1"`).

### 7.2 `testing.md` §4.1

Update the TestFile format table:

| Line | Yields |
|---|---|
| `y` / `yes` | `Input { player_id: "P1", kind: InputKind::OptionalAccept }` |
| `n` / `no` | `Input { player_id: "P1", kind: InputKind::OptionalDecline }` |
| `<N>` | `Input { player_id: "P1", kind: InputKind::Choice { idx: N-1 } }` (1-based) |
| `p <N>` | `Input { player_id: "P1", kind: InputKind::ChoosePlayer { idx: N-1 } }` |
| `c <csv>` | `Input { player_id: "P1", kind: InputKind::ChooseCards { selected: [..] } }` |
| `Name:y` / `Name:<N>` / etc. | Same as above, with `player_id: "Name"` |

Add note: *"Lines without a `Name:` prefix default to player `"P1"`. Use the prefix in
multi-player test scenarios where different players submit inputs in sequence."*

### 7.3 `api-usage.md`

Update the `InputSource::Player` closure example to show `Input { player_id, kind }`
construction with the new API shape.

---

## 8. Implementation order

1. **`types.rs`** — Define `Input` struct + `InputKind` enum + accessors. All subsequent
   steps depend on this.
2. **`controller/mod.rs`** — Update `validate_player_input`, `get_input`,
   `read_test_file`. Core logic.
3. **`interpreter/mod.rs`** — Pattern match updates (small scope, mechanical).
4. **`interpreter/quant_driver.rs`** — Pattern match updates (small scope, mechanical).
5. **All test files** — Mechanical wrapping. Large volume but straightforward.
6. **TUI changes** (`main.rs`, `state.rs`, `input.rs`) — Gating logic, rendering, player
   name tracking.
7. **`cgdsl-play.rs`** — Shared state + `event_sender`.
8. **Docs** — `invariants.md`, `testing.md`, `api-usage.md`.
9. **Full test suite** — `cargo test -p cgdsl-engine` to verify no regressions.
