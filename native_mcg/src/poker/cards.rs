use mcg_shared::Card;

// Re-export the shared types for backward compatibility
pub use mcg_shared::{CardRank, CardSuit};

/// Returns a string like "A♣", "T♦", etc.
pub fn card_str(c: Card) -> String {
    c.to_string()
}
