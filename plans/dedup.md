# Code Deduplication Analysis Report

## Overview

This report identifies significant code duplication across the MCG codebase and provides actionable recommendations for consolidation. The analysis covers all three main crates: `frontend`, `native_mcg`, and `shared`.

## Critical Duplications Found

### 1. Stage/Game State Display Functions

**Location:** 
- `frontend/src/game/screens/poker/ui_components.rs`
- `frontend/src/game/screens/poker_online.rs`

**Issue:** Identical functions duplicated across files:

```rust
// In ui_components.rs
pub fn stage_badge(stage: Stage) -> WidgetText {
    let (txt, color) = match stage {
        Stage::Preflop => ("Preflop", Color32::from_rgb(100, 150, 255)),
        Stage::Flop => ("Flop", Color32::from_rgb(100, 200, 120)),
        Stage::Turn => ("Turn", Color32::from_rgb(230, 180, 80)),
        Stage::River => ("River", Color32::from_rgb(220, 120, 120)),
        Stage::Showdown => ("Showdown", Color32::from_rgb(180, 100, 220)),
    };
    RichText::new(txt).color(color).strong().into()
}

pub fn stage_to_str(stage: Stage) -> &'static str {
    match stage {
        Stage::Preflop => "Preflop",
        Stage::Flop => "Flop",
        Stage::Turn => "Turn",
        Stage::River => "River",
        Stage::Showdown => "Showdown",
    }
}

// IDENTICAL functions in poker_online.rs (lines 800-823)
```

**Recommendation:** Remove duplicates from `poker_online.rs` and use the implementations from `ui_components.rs`.

### 2. Card Display Functions

**Location:**
- `frontend/src/game/screens/poker/ui_components.rs`
- `frontend/src/game/screens/poker_online.rs`

**Issue:** Multiple duplicate card-related functions:

```rust
// In ui_components.rs
pub fn card_text_and_color(c: Card) -> (String, Color32) {
    let text = c.to_string();
    let color = if c.is_red() {
        Color32::from_rgb(220, 50, 50)
    } else {
        Color32::WHITE
    };
    (text, color)
}

pub fn card_text(c: Card) -> String {
    c.to_string()
}

fn card_chip(ui: &mut Ui, c: Card) {
    let (text, color) = card_text_and_color(c);
    let b = egui::widgets::Button::new(RichText::new(text).color(color).size(28.0))
        .min_size(egui::vec2(48.0, 40.0));
    ui.add(b);
}

// IDENTICAL functions in poker_online.rs (lines 793-798)
```

**Recommendation:** Remove duplicates from `poker_online.rs` and import from `ui_components.rs`.

### 3. Game State Formatting Functions

**Location:**
- `frontend/src/game/screens/poker/ui_components.rs`  
- `frontend/src/game/screens/poker_online.rs`

**Issue:** Massive duplication of game state formatting logic (200+ lines):

```rust
// Nearly identical functions:
- format_game_for_clipboard()
- format_game_summary()
- format_players_section()
- format_board_section()
- format_action_log()
- format_board_by_length()
```

**Recommendation:** Consolidate all formatting functions in `ui_components.rs` and remove from `poker_online.rs`.

### 4. Player Name Resolution

**Location:**
- `frontend/src/game/screens/poker/ui_components.rs`
- `native_mcg/src/pretty.rs`

**Issue:** Similar but slightly different implementations:

```rust
// In ui_components.rs
pub fn name_of(players: &[PlayerPublic], id: PlayerId) -> String {
    players
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Player {}", id))
}

// In native_mcg/src/pretty.rs
fn player_name(players: &[PlayerPublic], id: PlayerId) -> String {
    players
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("P{}", id))
}
```

**Recommendation:** Move this function to `shared/src/player.rs` as a utility method.

### 5. Card Formatting Cross-Crate Duplication

**Location:**
- `shared/src/cards.rs` (has `Card::to_string()`, `Card::rank_str()`, `Card::suit_char()`)
- `native_mcg/src/pretty.rs` (has duplicate `card_faces()`, `suit_icon()`, `format_card()`)

**Issue:** The shared crate already provides card string representation, but native_mcg reimplements similar functionality:

```rust
// In shared/src/cards.rs (ALREADY EXISTS)
impl Card {
    pub fn rank_str(self) -> &'static str { /* A, 2, 3, ... K */ }
    pub fn suit_char(self) -> char { /* ♣, ♦, ♥, ♠ */ }
    pub fn to_string(self) -> String { /* "A♣", "T♦", etc. */ }
}

// In native_mcg/src/pretty.rs (DUPLICATE LOGIC)
fn card_faces(rank: CardRank) -> &'static str { /* A, 2, 3, ... K */ }
fn suit_icon(suit: CardSuit) -> char { /* ♣, ♦, ♥, ♠ */ }
fn format_card(c: Card, color: bool) -> String { /* Custom formatting */ }
```

**Recommendation:** Extend shared Card implementation with optional color formatting and remove duplicates from native_mcg.

### 8. Default Implementation Pattern

**Location:** Throughout the codebase

**Issue:** Many structs have both `Default` and `new()` implementations doing the same thing:

```rust
impl Default for SomeStruct {
    fn default() -> Self { /* ... */ }
}

impl SomeStruct {
    pub fn new() -> Self { Self::default() } // Redundant
}
```

**Recommendation:** Remove redundant `new()` methods that just call `Default::default()`.

### 9. WebSocket Message Handling

**Location:**
- `frontend/src/game/screens/poker_online.rs`
- `frontend/src/game/screens/poker/connection_manager.rs`

**Issue:** Similar WebSocket connection and message handling patterns appear to be duplicated.

There is only one poker implementation, you can move files around if that makes it more clear

**Recommendation:** consolidate connection management.

