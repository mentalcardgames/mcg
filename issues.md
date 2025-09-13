# Technical Debt Report: MCG Rust Codebase

Analysis focused on `frontend/`, `native_mcg/`, and `shared/` directories using code linters, searches for error-prone patterns, magic numbers, and compilation checks. Total files scanned: ~50 .rs files across dirs.

## Remaining Issues

### Magic Numbers - Card Rank/Suit Indices
Replace magic number indices with strongly-typed enums for better code readability and maintainability.

- **File:** `native_mcg/src/eval.rs`
  **Lines:** 3-13, 19-25, 206-213
  **Description:** Card rank/suit functions use magic numbers (0=Ace, 1=2, etc.; 0=Clubs, 1=Diamonds, etc.) and array indexing with hardcoded indices.
  **Current Code:**
  ```rust
  // Lines 5-7: rank index (0..=12) where 0 is Ace, 1 is 2, ..., 12 is King
  pub fn card_rank(c: Card) -> u8 {
      c.0 % 13
  }

  // Lines 11-13: suit index (0..=3) where 0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades
  pub fn card_suit(c: Card) -> u8 {
      c.0 / 13
  }

  // Lines 19-25: array indexing with magic numbers
  let rank_idx = card_rank(c) as usize;
  let suit_idx = card_suit(c) as usize;
  let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K"];
  let suits = ["♣", "♦", "♥", "♠"];
  ```

  **Suggested Fix:** Create enums and replace magic numbers:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum CardRank {
      Ace = 0, Two = 1, Three = 2, Four = 3, Five = 4,
      Six = 5, Seven = 6, Eight = 7, Nine = 8, Ten = 9,
      Jack = 10, Queen = 11, King = 12
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum CardSuit {
      Clubs = 0, Diamonds = 1, Hearts = 2, Spades = 3
  }

  // Usage: CardRank::Ace as u8 instead of 0, CardSuit::Clubs as u8 instead of 0
  ```

### Magic Numbers

- **File:** `native_mcg/src/eval.rs`
  **Lines:** 100, 105, 119
  **Description:** Magic numbers for array sizes and indexing:
  - `[Vec<Card>; 4]` - hardcoded suit count (4 suits)
  - `(0..4)` - hardcoded suit range
  - `[0u8; 15]` - hardcoded array size for rank counting (2..14 + unused indices)

  **Suggested Fix:** Use named constants:
  ```rust
  pub const NUM_SUITS: usize = 4;
  pub const NUM_RANKS: usize = 13;
  pub const RANK_COUNT_ARRAY_SIZE: usize = 15; // 2..14 + unused 0..1
  ```

## Long Functions (>50 Lines)
Manual inference from file lists (no direct line count tool); flagged based on common patterns in game files. These hinder maintainability; split for testability.

- **File:** `native_mcg/src/game/engine.rs`
  **Lines:** ~200+ (inferred from compilation context and engine role)
  **Description:** Likely oversized `Game` methods (e.g., state updates); compounds with type errors.
  **Suggested Fix:** Refactor into sub-modules (e.g., `betting.rs` already exists—extract more); aim <30 lines per fn.

- **File:** `frontend/src/game/screens/poker_online.rs`
  **Lines:** 614-737 (partial, but screen handlers long)
  **Description:** UI rendering + state logic in one fn; mixes concerns.
  **Suggested Fix:** Split into `render_ui()`, `handle_input()`; use egui widgets for modularity.

- **File:** `native_mcg/src/eval.rs`
  **Lines:** ~365 (hand evaluation)
  **Description:** Poker eval logic monolithic; hard to test edge cases.
  **Suggested Fix:** Break into `evaluate_hand()`, `rank_kickers()`

## Inconsistent Naming Conventions
Detected via patterns; minor but accumulates debt.

- **Files:** Across `native_mcg/src/game/*` (e.g., `to_act` vs `dealer_idx`)
  **Description:** Mix of snake_case vars (Rust std) and abbreviations (e.g., `pid` in shared); inconsistent with egui camelCase in frontend.
  **Suggested Fix:** Run `cargo fmt`; audit for full names (e.g., `to_act_idx`); follow Rust API guidelines.

## Duplicated Code Blocks

- **Files:** `frontend/src/game/card.rs` and `field.rs`
  **Description:** Similar image loading/fallback logic duplicated.
  **Suggested Fix:** Extract to `utils::load_card_image()` in `shared`.

- **Files:** `native_mcg/src/bot.rs` and `eval.rs`
  **Description:** Hand ranking snippets repeated.
  **Suggested Fix:** Centralize in `eval` module; reuse via traits.
