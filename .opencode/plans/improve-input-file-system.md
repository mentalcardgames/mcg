# Feature: Input File System Improvements

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Improve the test input file system for the CGDSL game engine with three key changes:
1. Extend `InputType::Choice` with `max_index` for bounds validation
2. Track input sequence number (not file line number) in error messages
3. Add integration test with a test game and input file

## User Story

As a developer writing test input files for the CGDSL engine
I want my inputs to be validated and error messages to show correct sequence numbers
So that test failures are easy to diagnose and fix

## Problem Statement

Current issues with the input file system:
1. `InputType::Choice` only provides option labels, no validation context for bounds checking
2. Error messages report "line X" where X is `loaded_line_count - remaining_buffer` which doesn't correspond to actual input sequence (comments/blanks are filtered)
3. No integration test exists that runs an actual game with a test input file

## Solution Statement

- Change `InputType::Choice` from tuple variant to struct variant with `max_index` field
- Add `input_sequence: usize` field to `Controller` struct to track which input (1-based) we're processing
- Validate choice indices in `get_input()` before passing to interpreter
- For `InputSource::Player`, retry on invalid input; for `InputSource::TestFile`, return error (test file must be correct)
- Create test game and input file for integration testing

## Feature Metadata

**Feature Type**: Enhancement
**Estimated Complexity**: Low
**Primary Systems Affected**: `crates/engine/src/interpreter.rs`, `crates/engine/src/controller.rs`, `crates/engine/src/bin/cgdsl-play.rs`
**Dependencies**: None (only internal changes)

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `crates/engine/src/interpreter.rs` (lines 178-182) - Why: Contains `InputType` definition to modify
- `crates/engine/src/interpreter.rs` (lines 48-56) - Why: Call site where `InputType::Choice` is constructed
- `crates/engine/src/controller.rs` (lines 1-96) - Why: `Controller` struct and `get_input` function to modify
- `crates/engine/src/controller.rs` (lines 108-149) - Why: `read_test_file` function with error messages to fix
- `crates/engine/src/controller.rs` (lines 162-241) - Why: Existing test patterns to follow
- `crates/engine/src/bin/cgdsl-play.rs` (lines 50-77) - Why: Interactive input handler using `InputType::Choice`
- `crates/engine/test_games/test.cgdsl` - Why: Existing test game for reference

### New Files to Create

- `crates/engine/test_games/ordering_test.cgdsl` - Test game with 2 rounds, each having `choose` + `optional`
- `crates/engine/test_games/ordering_test.txt` - Test input file with comments, blanks, and 4 valid inputs

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- No external documentation required - this is purely internal engine changes

### Patterns to Follow

**Existing `Input` enum pattern (interpreter.rs:155-170):**
```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Choice { idx: usize },
    OptionalAccept,
    OptionalDecline,
}

impl Input {
    pub fn idx(&self) -> usize {
        match self {
            Input::Choice { idx } => *idx,
            Input::OptionalAccept => 0,
            Input::OptionalDecline => 1,
        }
    }
}
```

**Existing Controller struct initialization pattern (controller.rs:31-43):**
```rust
let mut controller = Controller {
    interpreter: Interpreter {
        ir,
        game_data,
        input_buffer: Vec::new(),
        current_state: entry,
    },
    input_source,
    event_sender,
    line_buffer: VecDeque::new(),
    file_loaded: false,
    loaded_line_count: 0,
};
```

**Integration test pattern using `env!("CARGO_MANIFEST_DIR")`:**
```rust
let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
let source = std::fs::read_to_string(&game_path).expect("Failed to read test game file");
let game = parse_document(&source).expect("Failed to parse game");
let ir = game.to_lowered_graph();
```

---

## IMPLEMENTATION PLAN

### Phase 1: Update InputType Definition

**Tasks:**

- Modify `InputType` enum in `interpreter.rs` to change `Choice` from tuple to struct variant

### Phase 2: Update Call Sites

**Tasks:**

- Update `interpreter.rs` where `InputType::Choice` is constructed
- Update `bin/cgdsl-play.rs` pattern match for `InputType::Choice`

### Phase 3: Controller Enhancements

**Tasks:**

- Add `input_sequence: usize` field to `Controller` struct
- Initialize `input_sequence: 0` in `run_game()`
- Add validation logic in `get_input()` with retry loop for Player source
- Fix error messages in `read_test_file()` to use `input_sequence`

### Phase 4: Test Files

**Tasks:**

- Create `test_games/ordering_test.cgdsl` test game
- Create `test_games/ordering_test.txt` test input file

### Phase 5: Integration Test

**Tasks:**

- Add `test_input_file_ordering_and_validation` test in `controller.rs`

---

## STEP-BY-STEP TASKS

### UPDATE crates/engine/src/interpreter.rs

- **TARGET**: Lines 178-182, change `InputType` definition
- **IMPLEMENT**: Replace tuple variant with struct variant
```rust
pub enum InputType {
    Choice { options: Vec<String>, max_index: usize },
    Optional(String),
}
```
- **PATTERN**: See existing `Input` enum at lines 155-170 for struct variant pattern
- **VALIDATE**: `cargo check -p cgdsl-engine`

---

### UPDATE crates/engine/src/interpreter.rs

- **TARGET**: Line ~55, where `InputType::Choice` is constructed
- **IMPLEMENT**: Pass `max_index` when creating variant
```rust
StepResult::NeedsInput(InputType::Choice { 
    options, 
    max_index: options.len().saturating_sub(1) 
})
```
- **PATTERN**: Existing construction at line 55 was `InputType::Choice(options)`
- **VALIDATE**: `cargo check -p cgdsl-engine`

---

### UPDATE crates/engine/src/bin/cgdsl-play.rs

- **TARGET**: Lines 55-60, interactive input handler pattern match
- **IMPLEMENT**: Destructure struct variant, use `max_index + 1` for prompt
```rust
InputType::Choice { options, max_index } => {
    println!("\n--- Choice ---");
    for (i, opt) in options.iter().enumerate() {
        println!("  {}. {opt}", i + 1);
    }
    print!("Enter 1-{}: ", max_index + 1);
```
- **PATTERN**: Existing pattern at lines 55-60
- **VALIDATE**: `cargo check -p cgdsl-engine`

---

### UPDATE crates/engine/src/controller.rs

- **TARGET**: Lines ~47-63, `Controller` struct definition
- **IMPLEMENT**: Add `input_sequence: usize` field after `loaded_line_count`
```rust
struct Controller {
    interpreter: Interpreter,
    input_source: InputSource,
    event_sender: Option<Box<dyn Fn(&GameData) + Send>>,
    line_buffer: VecDeque<String>,
    file_loaded: bool,
    loaded_line_count: usize,
    input_sequence: usize,  // NEW: tracks which input we're on (1-based)
}
```
- **PATTERN**: Existing struct definition at lines 48-63
- **VALIDATE**: `cargo check -p cgdsl-engine`

---

### UPDATE crates/engine/src/controller.rs

- **TARGET**: Lines ~31-44, `run_game` function where `Controller` is initialized
- **IMPLEMENT**: Add `input_sequence: 0` to the `Controller` initialization
```rust
let mut controller = Controller {
    interpreter: Interpreter {
        ir,
        game_data,
        input_buffer: Vec::new(),
        current_state: entry,
    },
    input_source,
    event_sender,
    line_buffer: VecDeque::new(),
    file_loaded: false,
    loaded_line_count: 0,
    input_sequence: 0,  // NEW
};
```
- **PATTERN**: Existing initialization at lines 31-43
- **VALIDATE**: `cargo check -p cgdsl-engine`

---

### UPDATE crates/engine/src/controller.rs

- **TARGET**: Lines ~86-96, `get_input` function
- **IMPLEMENT**: Replace entire function with validation logic
```rust
fn get_input(&mut self, input_type: InputType) -> Result<Input, String> {
    self.input_sequence += 1;

    let input = match &self.input_source {
        InputSource::Player(callback) => {
            loop {
                let raw = callback(input_type.clone());
                if let Input::Choice { idx } = &raw {
                    if let InputType::Choice { max_index, .. } = &input_type {
                        if idx > max_index {
                            continue; // Ask again
                        }
                    }
                }
                break raw;
            }
        }
        InputSource::TestFile(path) => self.read_test_file(path)?,
    };

    Ok(input)
}
```
- **PATTERN**: Original function at lines 88-96
- **GOTCHA**: For Player, invalid choice index causes retry loop; for TestFile, validation happens at read time (see next task)
- **VALIDATE**: `cargo check -p cgdsl-engine`

---

### UPDATE crates/engine/src/controller.rs

- **TARGET**: Lines ~128-147, `read_test_file` error messages
- **IMPLEMENT**: Replace `consumed_lines` with `self.input_sequence` in error messages
- **ORIGINAL** (line 136):
```rust
"Invalid test input at line {}: expected number, 'y', or 'n', got '{}'",
consumed_lines, line
```
- **NEW**:
```rust
"Invalid test input #{}: expected number, 'y', or 'n', got '{}'",
self.input_sequence, line
```
- **ORIGINAL** (line 141-143):
```rust
return Err(format!(
    "Invalid test input at line {}: choice indices start at 1, got 0",
    consumed_lines
));
```
- **NEW**:
```rust
return Err(format!(
    "Invalid test input #{}: choice indices start at 1, got 0",
    self.input_sequence
));
```
- **PATTERN**: Original error messages at lines 136 and 141-143
- **VALIDATE**: `cargo check -p cgdsl-engine`

---

### CREATE crates/engine/test_games/ordering_test.cgdsl

- **IMPLEMENT**: Create test game with 2 rounds, each having `choose` (2 options) + `optional`
```cgdsl
player P1, P2
turnorder (P:P1, P:P2)
location Hand on all
location Stock, Table on table

card on Stock:
  Rank(Ace, Two, Three, Four)
    for Suite(Diamonds, Hearts)

stage Setup for current 1 times {
  deal 2 from top(Stock) private to Hand
}

stage Play for current 2 times {
  choose {
    move top(Hand) face down to Table
    or
    move top(Hand) face up to Table
  }
  
  optional {
    deal 1 from top(Stock) private to Hand
  }
  
  cycle to next
}

stage End for current 1 times {
  end Play
}
```
- **PATTERN**: See `test_games/test.cgdsl` for reference
- **VALIDATE**: `cargo build -p cgdsl-engine --message-format=plain` (game file is parsed at runtime)

---

### CREATE crates/engine/test_games/ordering_test.txt

- **IMPLEMENT**: Create test input file with comments, blanks, and 4 valid inputs
```
# Test input file for ordering verification
# Blank lines and comments are skipped
# First valid line = first input consumed

1

y

2

n
```
- **PATTERN**: Comments start with `#`, blank lines ignored, FIFO processing
- **EXPECTED**: Inputs in order: 1, y, 2, n (matches choose/optional/choose/optional)
- **VALIDATE**: See integration test below

---

### UPDATE crates/engine/src/controller.rs

- **TARGET**: Add new test function in `#[cfg(test)]` module (after line 241)
- **IMPLEMENT**:
```rust
#[test]
fn test_input_file_ordering_and_validation() {
    use front_end::validation::parse_document;
    
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let game_path = manifest_dir.join("test_games/ordering_test.cgdsl");
    let input_path = manifest_dir.join("test_games/ordering_test.txt");

    let source = std::fs::read_to_string(&game_path)
        .expect("Failed to read test game file");
    let game = parse_document(&source).expect("Failed to parse game");
    let ir = game.to_lowered_graph();

    let game_data = GameData::new();
    let result = run_game(
        ir,
        game_data,
        InputSource::TestFile(input_path),
        None,
    );

    assert!(result.is_ok(), "Game should complete successfully");
}
```
- **PATTERN**: See existing tests at lines 168-241 for test structure
- **IMPORTS**: Add `use front_end::validation::parse_document;` at top of test module
- **VALIDATE**: `cargo test -p cgdsl-engine test_input_file_ordering_and_validation`

---

## TESTING STRATEGY

### Unit Tests

- Existing tests in `controller.rs:162-241` test `read_test_file` parsing logic
- New validation logic in `get_input` is tested indirectly via integration test

### Integration Tests

- `test_input_file_ordering_and_validation`: Parses actual game file, runs with test input file, verifies successful completion
- Validates that:
  - Comments and blank lines are properly skipped
  - First non-comment/blank line is first input consumed
  - 4 inputs (1, y, 2, n) are consumed in correct order

### Edge Cases

- **Invalid choice index in test file**: `read_test_file` returns error (test file must be valid)
- **Invalid choice index in interactive mode**: Retry loop asks player again
- **Exhausted test file mid-game**: Returns "Test input file exhausted" error

---

## VALIDATION COMMANDS

Execute every command in order to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
cargo fmt --all
```

### Level 2: Compilation

```bash
cargo check -p cgdsl-engine
```

### Level 3: Unit Tests

```bash
cargo test -p cgdsl-engine
```

### Level 4: Clippy

```bash
cargo clippy -p cgdsl-engine --all-targets -- -D warnings
```

---

## ACCEPTANCE CRITERIA

- [ ] `InputType::Choice` changed to struct variant with `max_index` field
- [ ] All call sites updated (interpreter.rs, cgdsl-play.rs)
- [ ] `Controller` has `input_sequence` field tracking input number (1-based)
- [ ] `get_input` validates choice indices, retries for Player source
- [ ] Error messages use input sequence number, not file line number
- [ ] Test game `ordering_test.cgdsl` created with 2 rounds (choose + optional each)
- [ ] Test input `ordering_test.txt` created with 4 valid inputs (1, y, 2, n)
- [ ] Integration test `test_input_file_ordering_and_validation` added
- [ ] All validation commands pass

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] `cargo fmt --all` executed
- [ ] `cargo check -p cgdsl-engine` passes
- [ ] `cargo test -p cgdsl-engine` passes (all 3 tests including new one)
- [ ] `cargo clippy -p cgdsl-engine --all-targets -- -D warnings` passes with no warnings
- [ ] No regressions in existing functionality

---

## NOTES

**On Input Validation Strategy:**
- For `InputSource::Player` (interactive): Invalid input triggers retry loop. Player keeps getting asked until valid input is given.
- For `InputSource::TestFile`: Invalid input returns error. Test files are expected to be correct by design.

**On OptionalAccept/OptionalDecline Validation:**
- Not explicitly validated because the Controller only validates what it receives. If a test file provides `y` when a Choice is expected, it falls through to the number parser and returns an error.

**On max_index Calculation:**
- `options.len().saturating_sub(1)` handles empty options case (returns 0, making any index invalid)

**On input_sequence:**
- Starts at 0, incremented BEFORE reading input
- First input gets sequence number 1
- Tracks total inputs consumed, not total lines in file
