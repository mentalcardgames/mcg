//! Card-related types and constants for the Mental Card Game.

use egui::Vec2;
use serde::{Deserialize, Serialize};

/// Natural size for card display in the UI
pub const CARD_NATURAL_SIZE: Vec2 = Vec2::new(140.0, 190.0);

/// Card rank values (0=Ace, 1=2, ..., 12=King)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl CardRank {
    /// Convert from u8 to CardRank. Panics if value > 12.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CardRank::Ace,
            1 => CardRank::Two,
            2 => CardRank::Three,
            3 => CardRank::Four,
            4 => CardRank::Five,
            5 => CardRank::Six,
            6 => CardRank::Seven,
            7 => CardRank::Eight,
            8 => CardRank::Nine,
            9 => CardRank::Ten,
            10 => CardRank::Jack,
            11 => CardRank::Queen,
            12 => CardRank::King,
            _ => panic!("Invalid card rank: {}", value),
        }
    }

    /// Convert to usize for array indexing.
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

/// Card suit values (0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardSuit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

impl CardSuit {
    /// Convert from u8 to CardSuit. Panics if value > 3.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CardSuit::Clubs,
            1 => CardSuit::Diamonds,
            2 => CardSuit::Hearts,
            3 => CardSuit::Spades,
            _ => panic!("Invalid card suit: {}", value),
        }
    }

    /// Convert to usize for array indexing.
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

/// A playing card represented as a compact u8 value
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Card(pub u8);

impl Card {
    /// Create a new card from rank and suit
    pub fn new(rank: CardRank, suit: CardSuit) -> Self {
        Card((suit as u8) * 13 + (rank as u8))
    }

    /// Get the rank of this card
    pub fn rank(self) -> CardRank {
        CardRank::from_u8(self.0 % 13)
    }

    /// Get the suit of this card
    pub fn suit(self) -> CardSuit {
        CardSuit::from_u8(self.0 / 13)
    }

    /// Get the rank as a string (A, 2, 3, ..., K)
    pub fn rank_str(self) -> &'static str {
        match self.rank() {
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
        }
    }

    /// Get the suit as a character (♣, ♦, ♥, ♠)
    pub fn suit_char(self) -> char {
        match self.suit() {
            CardSuit::Clubs => '♣',
            CardSuit::Diamonds => '♦',
            CardSuit::Hearts => '♥',
            CardSuit::Spades => '♠',
        }
    }

    /// Get the card as a string like "A♣", "T♦", etc.
    pub fn to_string(self) -> String {
        format!("{}{}", self.rank_str(), self.suit_char())
    }

    /// Check if this is a red suit (hearts or diamonds)
    pub fn is_red(self) -> bool {
        matches!(self.suit(), CardSuit::Hearts | CardSuit::Diamonds)
    }

    /// Check if this is a black suit (clubs or spades)
    pub fn is_black(self) -> bool {
        matches!(self.suit(), CardSuit::Clubs | CardSuit::Spades)
    }
}