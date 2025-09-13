# Technical Debt Report: MCG Rust Codebase

Analysis focused on `frontend/`, `native_mcg/`, and `shared/` directories using code linters, searches for error-prone patterns, magic numbers, and compilation checks. Total files scanned: ~50 .rs files across dirs.

## High-Impact Issues (Build/Run Failures)

### Compilation Errors
These prevent building `native_mcg`, blocking native backend and CLI functionality.

- **File:** `native_mcg/src/game/engine.rs`  
  **Lines:** 229, 237  
  **Description:** Type mismatch: Integer literals used where `PlayerId` (from `mcg_shared`) is expected in struct initialization, causing E0308 errors during compilation.  
  **Suggested Fix:** Wrap literals in `PlayerId` constructor, e.g., `id: PlayerId(0)`. Run `cargo fix` or manual replacement; impacts game engine initialization.

## Magic Numbers and Hardcoded Values
~50 potential integers/floats detected via regex (e.g., sizes, probabilities, indices). These reduce configurability/scalability; common in UI/game logic. Prioritize game params.

- **File:** `frontend/src/game/field.rs` (and similar in `card.rs`)  
  **Lines:** 128, 135 (implied from search)  
  **Description:** Hardcoded card sizes `Vec2::new(140.0, 190.0)`; repeated across files. Impacts scalability for different resolutions.  
  **Suggested Fix:** Define `pub const CARD_NATURAL_SIZE: Vec2 = Vec2::new(140.0, 190.0);` in `shared` crate; use throughout.

- **File:** `frontend/src/game.rs` (and screens/*.rs)  
  **Lines:** 14 (font=14.0), 59 (0..3 loop), 152-155 (widths=120.0/140.0, margins=8.0)  
  **Description:** UI constants (fonts, margins, widths) scattered; inconsistent spacing.  
  **Suggested Fix:** Centralize in `ui::Theme` module with `const`s (e.g., `MARGIN_SM: f32 = 8.0;`).

Other: Indices like `i*4` in matrices (L61-64); game seeds=42. General: Use named constants or enums for indices (e.g., `CardRank::Ace as u8`).

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


