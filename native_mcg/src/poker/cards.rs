use super::constants::*;
use mcg_shared::Card;

/// Card rank values (0=Ace, 1=2, ..., 12=King)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardRank {
    Ace = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
    Eight = 7,
    Nine = 8,
    Ten = 9,
    Jack = 10,
    Queen = 11,
    King = 12,
}

/// Card suit values (0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSuit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

/// Returns the rank index (0..=12) where 0 is Ace, 1 is 2, ..., 12 is King.
#[inline]
pub fn card_rank(c: Card) -> u8 {
    c.0 % NUM_RANKS as u8
}

/// Returns the suit index (0..=3) where 0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades.
#[inline]
pub fn card_suit(c: Card) -> u8 {
    c.0 / NUM_RANKS as u8
}

/// Returns a string like "A♣", "T♦", etc.
pub fn card_str(c: Card) -> String {
    let rank_idx = card_rank(c) as usize;
    let suit_idx = card_suit(c) as usize;
    let ranks = [
        "A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K",
    ];
    let suits = ['♣', '♦', '♥', '♠'];
    format!("{}{}", ranks[rank_idx], suits[suit_idx])
}
