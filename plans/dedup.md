# Code Deduplication Analysis Report

### Card Formatting Cross-Crate Duplication

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

