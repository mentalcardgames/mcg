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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Returns the rank of the card as a CardRank enum.
#[inline]
pub fn card_rank(c: Card) -> CardRank {
    CardRank::from_u8(c.0 % NUM_RANKS as u8)
}

/// Returns the suit of the card as a CardSuit enum.
#[inline]
pub fn card_suit(c: Card) -> CardSuit {
    CardSuit::from_u8(c.0 / NUM_RANKS as u8)
}

/// Returns a string like "A♣", "T♦", etc.
pub fn card_str(c: Card) -> String {
    let rank = card_rank(c);
    let suit = card_suit(c);
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
