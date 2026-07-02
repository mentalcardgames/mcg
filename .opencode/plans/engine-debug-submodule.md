# Feature: engine::debug Submodule

## Feature Description

Add a debug submodule to the cgdsl-engine crate that provides utilities for printing and saving a compact view of the game state at any point during execution. The debug output will support three detail levels (Low, Medium, High) and can be printed to console or appended to a file.

## User Story

As a developer debugging the cgdsl-engine
I want to easily output the current game state to console or file
So that I can inspect game data, trace execution, and diagnose issues

## Problem Statement

Currently, inspecting game state during debugging requires manually accessing and formatting individual fields of `GameData`. There's no centralized, reusable way to get a consistent debug view at any point in the engine.

## Solution Statement

Create a `engine::debug` submodule with three functions:
- `format_game_data()` - returns formatted string (centralized logic)
- `print_game_data()` - prints to stdout
- `save_game_data()` - appends to file (parses level from first line marker)

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: Low
**Primary Systems Affected**: `crates/engine`
**Dependencies**: None new (uses only `std::fs`, `std::io`, `std::path`)

---

## CONTEXT REFERENCES

### Relevant Codebase Files

- `crates/engine/src/lib.rs` (lines 1-9) - Why: Shows module structure and exports pattern
- `crates/engine/src/game_data.rs` (lines 22-99) - Why: Contains all `GameData` fields that need formatting
- `crates/engine/src/game_data.rs` (lines 41-51) - Why: `MemoryValue` enum variants need formatting
- `crates/engine/src/controller.rs` (lines 168-267) - Why: Shows `#[cfg(test)] mod tests` pattern in engine crate

### New Files to Create

- `crates/engine/src/debug.rs` - Debug module implementation

### Patterns to Follow

**Module declaration** (lib.rs:1-5):
```rust
pub mod action;
pub mod controller;
pub mod game_data;
pub mod interpreter;
pub mod query;
```

**Test module pattern** (controller.rs:168-169):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // tests here
}
```

---

## IMPLEMENTATION PLAN

### Phase 1: Module Setup

**Tasks:**

- CREATE `crates/engine/src/debug.rs` with empty module and `DebugLevel` enum
- ADD `pub mod debug;` to `crates/engine/src/lib.rs`
- ADD `pub use debug::{DebugLevel, format_game_data, print_game_data, save_game_data};` to `crates/engine/src/lib.rs`

### Phase 2: DebugLevel Enum

**Tasks:**

- IMPLEMENT `DebugLevel` enum with `Low`, `Medium`, `High` variants
- IMPLEMENT `DebugLevel::from_marker(s: &str) -> Option<DebugLevel>` to parse `<!--LOW-->`, `<!--MEDIUM-->`, `<!--HIGH-->` markers
- IMPLEMENT `DebugLevel::marker(&self) -> &str` to get the string representation

### Phase 3: format_game_data Function

**Tasks:**

- IMPLEMENT `format_game_data(data: &GameData, level: DebugLevel) -> String`
- IMPLEMENT Low level formatting: player names, current player, stage, turn order indices, card counts per location
- IMPLEMENT Medium level formatting: add scores, teams, memories (key:value), first 5 cards per location (truncated)
- IMPLEMENT High level formatting: full GameData dump - all cards with attributes, all locations, all player fields, stage counters, stage stack, combos, precedences, point maps

### Phase 4: Output Functions

**Tasks:**

- IMPLEMENT `print_game_data(data: &GameData, level: DebugLevel)` - calls `format_game_data` and prints to stdout with `println!`
- IMPLEMENT `save_game_data(data: &GameData, path: &Path) -> Result<(), Error>`:
  - Read first line of existing file (if any)
  - Parse `DebugLevel` from `<!--LEVEL-->` marker (default to Medium if not found or file doesn't exist)
  - Append formatted output to file using `std::fs::OpenOptions` with `append(true)`

### Phase 5: Tests

**Tasks:**

- ADD `#[cfg(test)] mod tests` block to `debug.rs`
- ADD test for `DebugLevel::from_marker` with valid and invalid inputs
- ADD test for `DebugLevel::marker` roundtrip
- ADD test for `format_game_data` at each level using `GameData::new()` (empty state)
- ADD test for `save_game_data` file creation and append behavior

---

## STEP-BY-STEP TASKS

### CREATE crates/engine/src/debug.rs

- **IMPLEMENT**: Empty module with imports, `DebugLevel` enum, `format_game_data`, `print_game_data`, `save_game_data` stubs
- **PATTERN**: Follow `game_data.rs` structure (lines 1-22)
- **IMPORTS**: `use crate::game_data::GameData;`, `use std::path::Path;`, `use std::io::{self, Write};`, `use std::fs::{self, OpenOptions};`

### UPDATE crates/engine/src/lib.rs

- **ADD**: `pub mod debug;` after line 5
- **ADD**: `pub use debug::{DebugLevel, format_game_data, print_game_data, save_game_data};` after line 8
- **VALIDATE**: `cargo build -p cgdsl-engine` succeeds

### IMPLEMENT DebugLevel enum

- **IMPLEMENT**: 
```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DebugLevel {
    Low,
    Medium,
    High,
}
```
- **IMPLEMENT**: `impl DebugLevel { pub fn from_marker(s: &str) -> Option<Self>`, `pub fn marker(&self) -> &'static str` }`
- **GOTCHA**: Case-insensitive matching for markers (use `to_uppercase` or `to_lowercase`)
- **VALIDATE**: `cargo build -p cgdsl-engine` succeeds

### IMPLEMENT format_game_data (Low)

- **IMPLEMENT**: Format players (names only), current player name, current stage, turn order (as indices), card counts per location
- **PATTERN**: Use `game_data.players.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ")`
- **VALIDATE**: `cargo build -p cgdsl-engine && cargo test -p cgdsl-engine`

### IMPLEMENT format_game_data (Medium)

- **ADD**: scores, teams (name + player names), memories (key: value format), truncate cards to first 5 per location
- **PATTERN**: For truncated cards, use `.iter().take(5)` and append "..." if more exist
- **VALIDATE**: `cargo build -p cgdsl-engine && cargo test -p cgdsl-engine`

### IMPLEMENT format_game_data (High)

- **ADD**: All fields from `GameData` struct including:
  - All cards with all attributes (iterate `cards: Vec<Card>` where `Card = HashMap<String, String>`)
  - All locations with all cards
  - All player fields (name, score, owner, in_game, in_stage)
  - stage_counters, stage_stack
  - combos (name + filter), precedences, point_maps
- **GOTCHA**: `MemoryValue` is an enum - match on all variants to format each
- **VALIDATE**: `cargo build -p cgdsl-engine && cargo test -p cgdsl-engine`

### IMPLEMENT print_game_data

- **IMPLEMENT**: `println!("{}", format_game_data(data, level));`
- **VALIDATE**: `cargo build -p cgdsl-engine`

### IMPLEMENT save_game_data

- **IMPLEMENT**: 
  1. Try to read first line of existing file with `fs::read_to_string` 
  2. Parse `DebugLevel` from `<!--LEVEL-->` marker (default Medium)
  3. Open with `OpenOptions::new().append(true).create(true).open(path)?`
  4. Write formatted output with `writeln!(file, "\n{}", formatted)?`
- **PATTERN**: Use `fs::read_to_string` to check if file exists, `OpenOptions` for append
- **GOTCHA**: Handle file-not-found gracefully (file creation is expected)
- **VALIDATE**: `cargo build -p cgdsl-engine && cargo test -p cgdsl-engine`

### ADD tests

- **IMPLEMENT**: Tests in `#[cfg(test)] mod tests { ... }` block
- **PATTERN**: Copy pattern from `controller.rs:168-267`
- **VALIDATE**: `cargo test -p cgdsl-engine`

---

## TESTING STRATEGY

### Unit Tests

- `DebugLevel::from_marker` with valid markers (`<!--LOW-->`, `<!--MEDIUM-->`, `<!--HIGH-->`) and invalid input
- `DebugLevel::marker` roundtrip (Low -> `<!--LOW-->`, etc.)
- `format_game_data` produces non-empty output for each level
- `format_game_data` Low includes expected fields (players, stage, etc.)
- `save_game_data` creates file if not exists, appends if exists

### Edge Cases

- Empty `GameData` (no players, no cards)
- File write permission errors
- Invalid marker in existing file (should default to Medium)
- Very long card attribute values (should not panic)

---

## VALIDATION COMMANDS

### Level 1: Syntax & Build

```bash
cargo build -p cgdsl-engine
```

### Level 2: Unit Tests

```bash
cargo test -p cgdsl-engine
```

### Level 3: Clippy & Format

```bash
cargo clippy -p cgdsl-engine --all-targets -- -D warnings
cargo fmt -p cgdsl-engine --all
```

---

## ACCEPTANCE CRITERIA

- [ ] `DebugLevel` enum exists with `Low`, `Medium`, `High` variants
- [ ] `format_game_data(data, level)` returns a `String`
- [ ] `print_game_data(data, level)` prints to stdout
- [ ] `save_game_data(data, path)` appends to file, creates if not exists
- [ ] `save_game_data` reads `<!--LEVEL-->` marker from first line of existing files
- [ ] Low level shows: player names, current player, stage, turn order indices, card counts
- [ ] Medium level adds: scores, teams, memories, first 5 cards per location
- [ ] High level shows: all game data fields
- [ ] All tests pass
- [ ] Clippy passes with no warnings
- [ ] Code is formatted with `cargo fmt`

---

## NOTES

- File marker format: `<!--LEVEL-->` where LEVEL is uppercase (LOW, MEDIUM, HIGH)
- Case-insensitive marker parsing to be forgiving
- No new dependencies required - uses `std::fs`, `std::io`, `std::path`
- Debug output is not meant to be machine-parseable; human-readable formatting is priority
