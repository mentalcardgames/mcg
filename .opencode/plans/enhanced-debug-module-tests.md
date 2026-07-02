# Feature: Enhanced Debug Module Tests

## Feature Description

Add comprehensive tests to the `engine::debug` module that verify actual content of formatted output at each debug level, using both direct GameData construction (unit tests) and Controller-driven integration tests that capture game state after each step.

## User Story

As a developer debugging cgdsl-engine
I want tests that verify debug output contains expected values
So that I can confidently rely on debug output when diagnosing game engine issues

## Problem Statement

Current tests only verify that debug output contains expected section headers (e.g., "Players:", "Scores:"). They don't verify that actual game data values appear in the output (e.g., "Alice" player name, score of 100, card named "Fireball").

## Solution Statement

1. **Unit tests**: Create populated `GameData` structs with known values, format at each level, verify specific content appears
2. **Integration tests**: Use `Controller` with `event_sender` callback to capture snapshots at each game step, verify debug output reflects game progression
3. **Edge case tests**: Test boundary conditions (empty state, None values, truncated lists)

## Feature Metadata

**Feature Type**: Enhancement (Test Coverage)
**Estimated Complexity**: Low
**Primary Systems Affected**: `crates/engine/src/debug.rs`
**Dependencies**: None new (uses existing test infrastructure)

---

## CONTEXT REFERENCES

### Relevant Codebase Files

- `crates/engine/src/debug.rs` (lines 273-380) - Current test patterns to enhance
- `crates/engine/src/debug.rs` (lines 40-157) - format_game_data_low and _medium implementations
- `crates/engine/src/debug.rs` (lines 159-249) - format_game_data_high implementation
- `crates/engine/src/controller.rs` (lines 24-44) - run_game signature with event_sender
- `crates/engine/src/controller.rs` (lines 161-165) - emit_event callback pattern
- `crates/engine/src/controller.rs` (lines 251-267) - Full integration test pattern with parse_document
- `crates/engine/src/game_data.rs` (lines 102-118) - GameData::new() and field setters
- `crates/engine/src/game_data.rs` (lines 141-151) - add_player returns index
- `crates/engine/test_games/ordering_test.cgdsl` - Test game for integration tests
- `crates/engine/test_games/ordering_test.txt` - Test input file

### Patterns to Follow

**Unit test with populated GameData** (new tests):
```rust
#[test]
fn test_format_game_data_low_with_players() {
    let mut data = GameData::new();
    data.add_player("Alice".to_string());
    data.add_player("Bob".to_string());
    data.turn_order = vec![0, 1];
    data.enter_stage("Play".to_string(), vec!["Alice".to_string(), "Bob".to_string()]);

    let output = format_game_data(&data, DebugLevel::Low);
    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
    assert!(output.contains("Play"));  // stage name
    assert!(output.contains("[0, 1]"));  // turn order
}
```

**Integration test with event_sender** (new tests):
```rust
#[test]
fn test_debug_integration_game_snapshots() {
    use front_end::validation::parse_document;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
    let input_path = manifest_dir.join("test_games/ordering_test.txt");

    let source = std::fs::read_to_string(&game_path).expect("...");
    let game = parse_document(&source).expect("...");
    let ir = game.to_lowered_graph();

    let mut snapshots = Vec::new();
    let game_data = GameData::new();

    run_game(ir, game_data, InputSource::TestFile(input_path), Some(Box::new(|gd| {
        snapshots.push(gd.clone());
    })))?;

    assert!(!snapshots.is_empty());

    let output = format_game_data(&snapshots[0], DebugLevel::Low);
    assert!(output.contains("P1") || output.contains("P2"));
}
```

**GameData population helpers** (game_data.rs):
```rust
let mut data = GameData::new();
let p1 = data.add_player("Alice".to_string());  // returns usize
let p2 = data.add_player("Bob".to_string());
data.turn_order = vec![p1, p2];
data.players[p1].score = 100;
let stock_loc = data.add_location("Table".to_string(), Location { name: "Stock".to_string(), cards: vec![] });
let card_id = data.add_card(stock_loc, HashMap::from([("name".to_string(), "Fireball".to_string())]));
data.enter_stage("Play".to_string(), vec!["Alice".to_string(), "Bob".to_string()]);
```

---

## IMPLEMENTATION PLAN

### Phase 1: Unit Tests with Populated GameData

**Tasks:**

- ADD `test_format_game_data_low_with_players` - verifies player names appear in Low output
- ADD `test_format_game_data_low_with_stage` - verifies stage name appears in Low output
- ADD `test_format_game_data_low_turn_order_indices` - verifies turn order [0, 1] format appears
- ADD `test_format_game_data_low_card_counts` - verifies location card counts appear
- ADD `test_format_game_data_medium_scores` - verifies player scores appear (not in Low)
- ADD `test_format_game_data_medium_teams` - verifies team name and member names appear
- ADD `test_format_game_data_medium_memories` - verifies memory key:value pairs appear
- ADD `test_format_game_data_medium_truncated_cards` - verifies "...", truncating when >5 cards
- ADD `test_format_game_data_high_full_player_details` - verifies score, in_game, in_stage
- ADD `test_format_game_data_high_all_cards` - verifies all card HashMaps appear
- ADD `test_format_game_data_high_combos_precedences_pointmaps` - verifies these sections
- ADD `test_format_game_data_high_memories_typed` - verifies Int(), String() prefixes

### Phase 2: MemoryValue Variant Tests

**Tasks:**

- ADD `test_format_game_data_memory_value_variants` - test all 8 MemoryValue types at High level:
  - Int(42) -> "Int(42)"
  - String("foo") -> "String(\"foo\")"
  - CardSet([1,2,3]) -> "CardSet([1, 2, 3])"
  - PlayerCollection([0,1]) -> "PlayerCollection([0, 1])"
  - Team("T1") -> "Team(\"T1\")"
  - IntCollection([1,2]) -> "IntCollection([1, 2])"
  - StringCollection(["a","b"]) -> "StringCollection([\"a\", \"b\"])"
  - LocationCollection([0,1]) -> "LocationCollection([0, 1])"

### Phase 3: Edge Case Tests

**Tasks:**

- ADD `test_format_game_data_empty_players` - empty player list
- ADD `test_format_game_data_empty_locations` - no locations
- ADD `test_format_game_data_empty_cards` - no cards
- ADD `test_format_game_data_single_player` - only one player
- ADD `test_format_game_data_current_player_none` - current_player = None
- ADD `test_format_game_data_empty_turn_order` - turn_order is empty

### Phase 4: Integration Tests with Controller

**Tasks:**

- ADD `test_debug_integration_game_snapshots` in `controller.rs`:
  - Use `ordering_test.cgdsl` + `ordering_test.txt`
  - Capture snapshots via event_sender
  - Verify each snapshot produces valid debug output at all 3 levels
  - Assert that after Setup stage, certain expectations are met
- ADD `test_debug_integration_verify_game_progression`:
  - Verify snapshots show state changing across steps
  - E.g., cards move from Stock to Hand, stage changes from Setup to Play

### Phase 5: Save/Load Roundtrip Tests

**Tasks:**

- ADD `test_save_game_data_then_format` - save at one level, verify saved content matches format
- ADD `test_save_game_data_preserves_level_marker` - file first line contains level marker

---

## STEP-BY-STEP TASKS

### ADD test_format_game_data_low_with_players to debug.rs

- **IMPLEMENT**: Create GameData with 2 players ("Alice", "Bob"), format Low, assert "Alice" and "Bob" in output
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_low_with_players`

### ADD test_format_game_data_low_with_stage to debug.rs

- **IMPLEMENT**: Enter stage "Play", format Low, assert "Play" in output
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_low_with_stage`

### ADD test_format_game_data_low_turn_order_indices to debug.rs

- **IMPLEMENT**: Set turn_order = [0, 1], format Low, assert "[0, 1]" in output
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_low_turn_order_indices`

### ADD test_format_game_data_low_card_counts to debug.rs

- **IMPLEMENT**: Add location with 3 cards, format Low, assert "Stock: 3 cards" (or similar)
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_low_card_counts`

### ADD test_format_game_data_medium_scores to debug.rs

- **IMPLEMENT**: Set player score to 100, format Medium, assert "100" appears, Low must NOT show scores
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_medium_scores`

### ADD test_format_game_data_medium_teams to debug.rs

- **IMPLEMENT**: Create team "T1" with players [0, 1], format Medium, assert "T1" and player names
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_medium_teams`

### ADD test_format_game_data_medium_memories to debug.rs

- **IMPLEMENT**: Add memory "counter" = Int(5), format Medium, assert "counter: 5"
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_medium_memories`

### ADD test_format_game_data_medium_truncated_cards to debug.rs

- **IMPLEMENT**: Add 7 cards to location, format Medium, assert "..." appears (truncation)
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_medium_truncated_cards`

### ADD test_format_game_data_high_full_player_details to debug.rs

- **IMPLEMENT**: Set player score=50, in_game=true, format High, assert score=50 and in_game=true
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_high_full_player_details`

### ADD test_format_game_data_high_all_cards to debug.rs

- **IMPLEMENT**: Add cards with names "Fireball", "Lightning", format High, assert both names
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_high_all_cards`

### ADD test_format_game_data_high_combos_precedences_pointmaps to debug.rs

- **IMPLEMENT**: Add combo, precedence, point_map, format High, verify all three sections exist
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_high_combos_precedences_pointmaps`

### ADD test_format_game_data_high_memories_typed to debug.rs

- **IMPLEMENT**: Test all 8 MemoryValue variants, verify type prefix in High output (Int(), String(), etc.)
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_high_memories_typed`

### ADD test_format_game_data_memory_value_variants to debug.rs

- **IMPLEMENT**: Create GameData with all MemoryValue types, format High, verify each typed format
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_memory_value_variants`

### ADD edge case tests (empty, single player, none) to debug.rs

- **IMPLEMENT**: Six tests for edge cases (see Phase 3)
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_format_game_data_empty_*`

### ADD integration test test_debug_integration_game_snapshots to controller.rs

- **IMPLEMENT**: Use ordering_test.cgdsl, capture snapshots, verify each produces valid output
- **VALIDATE**: `cargo test -p cgdsl-engine controller::tests::test_debug_integration_game_snapshots`

### ADD integration test test_debug_integration_verify_game_progression to controller.rs

- **IMPLEMENT**: Verify snapshots show state changing (cards moving, stages changing)
- **VALIDATE**: `cargo test -p cgdsl-engine controller::tests::test_debug_integration_verify_game_progression`

### ADD save/load roundtrip tests to debug.rs

- **IMPLEMENT**: test_save_game_data_then_format and test_save_game_data_preserves_level_marker
- **VALIDATE**: `cargo test -p cgdsl-engine debug::tests::test_save_game_data_then_format`

---

## TESTING STRATEGY

### Unit Tests (in debug.rs)

- Test each debug level with known GameData state
- Verify specific field values appear in output
- Verify truncation behavior at Medium level
- Verify typed memory values at High level
- Edge cases: empty collections, None values, single items

### Integration Tests (in controller.rs)

- Use real game files (ordering_test.cgdsl)
- Capture GameData snapshots at each step via event_sender
- Verify debug output is consistent with captured game state
- Verify state progression across multiple steps

### Edge Cases

- Empty players/teams/locations/cards
- current_player = None
- Empty turn_order
- Single player/team
- More than 5 cards in location (truncation)
- All 8 MemoryValue variants

---

## VALIDATION COMMANDS

### Unit Tests

```bash
cargo test -p cgdsl-engine debug::tests
```

### Integration Tests

```bash
cargo test -p cgdsl-engine controller::tests::test_debug_integration
```

### All Engine Tests

```bash
cargo test -p cgdsl-engine
```

### Clippy and Format

```bash
cargo fmt -p cgdsl-engine --all
cargo check -p cgdsl-engine
```

---

## ACCEPTANCE CRITERIA

- [ ] Low level tests verify: player names, current player, stage, turn order indices, card counts
- [ ] Medium level tests verify: scores, teams, memories, truncated cards (>5 shows "...")
- [ ] High level tests verify: full player details (score, in_game, in_stage), all cards, combos, precedences, point_maps, typed memories
- [ ] MemoryValue variant tests cover all 8 types at High level
- [ ] Edge case tests for: empty collections, current_player=None, single player
- [ ] Integration tests capture snapshots via event_sender from real game execution
- [ ] All tests pass
- [ ] Code formatted with cargo fmt

---

## NOTES

- Integration tests should go in `controller.rs` alongside existing integration tests, not in `debug.rs`
- Use `PathBuf::from(env!("CARGO_MANIFEST_DIR"))` to locate test game files reliably
- event_sender callback receives `&GameData` reference - clone if storing multiple snapshots
- Test file cleanup: use `let _ = fs::remove_file(&path)` before assertions, and cleanup in test end
