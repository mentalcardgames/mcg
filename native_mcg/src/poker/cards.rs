use mcg_shared::Card;

// Re-export the shared types for backward compatibility
pub use mcg_shared::{CardRank, CardSuit};

/// Returns the rank of the card as a CardRank enum.
#[inline]
pub fn card_rank(c: Card) -> CardRank {
    c.rank()
}

/// Returns the suit of the card as a CardSuit enum.
#[inline]
pub fn card_suit(c: Card) -> CardSuit {
    c.suit()
}

/// Returns a string like "A♣", "T♦", etc.
pub fn card_str(c: Card) -> String {
    let rank = c.rank();
    let suit = c.suit();
    let rank_str = match rank {
        CardRank::Ace => "A",
        CardRank::Two => "2",
        CardRank::Three => "3",
        CardRank::Four => "4",
        CardRank::Five => "5",
        CardRank::Six => "6",
        CardRank::Seven => "7",
        CardRank::Eight => "8",
        CardRank::Nine => "9",
        CardRank::Ten => "T",
        CardRank::Jack => "J",
        CardRank::Queen => "Q",
        CardRank::King => "K",
    };
    let suit_char = match suit {
        CardSuit::Clubs => '♣',
        CardSuit::Diamonds => '♦',
        CardSuit::Hearts => '♥',
        CardSuit::Spades => '♠',
    };
    format!("{}{}", rank_str, suit_char)
}
