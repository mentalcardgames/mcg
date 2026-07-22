//! Communication module for the Mental Card Game.
//!
//! This module contains data structures for representing encrypted card game communications
//! using Elgamal cryptosystem over Z/pZ, where p is a large prime (up to 2048 bits).

use ::num_bigint::BigUint;
use std::fmt;

/// Maximum bit length for prime p (modulus)
pub const MAX_PRIME_BIT_LENGTH: usize = 2048;

/// A single element in Z/pZ (the smallest natural number from a residue class)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModularElement {
    /// The value of the element (smallest natural number representative)
    pub value: BigUint,
    /// The modulus (prime p)
    pub modulus: BigUint,
}

impl ModularElement {
    /// Create a new element in Z/pZ
    pub fn new(value: BigUint, modulus: BigUint) -> Self {
        let normalized_value = value % &modulus;
        Self {
            value: normalized_value,
            modulus,
        }
    }
}

impl fmt::Display for ModularElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (mod {})", self.value, self.modulus)
    }
}

/// An Elgamal ciphertext, which is a pair of elements from Z/pZ
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ElgamalCiphertext {
    /// First component of the ciphertext pair
    pub c1: ModularElement,
    /// Second component of the ciphertext pair
    pub c2: ModularElement,
}

impl ElgamalCiphertext {
    /// Create a new Elgamal ciphertext
    pub fn new(c1: ModularElement, c2: ModularElement) -> Self {
        // Ensure both elements have the same modulus
        assert_eq!(
            c1.modulus, c2.modulus,
            "Both elements must have the same modulus"
        );
        Self { c1, c2 }
    }
}

/// A bitstring with maximum length of |p| bits
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BitString {
    /// The actual bits stored as a BigUint
    pub bits: BigUint,
    /// The length of the bitstring
    pub length: usize,
}

impl BitString {
    /// Create a new bitstring from a BigUint and a length
    pub fn new(bits: BigUint, length: usize) -> Self {
        assert!(
            length <= MAX_PRIME_BIT_LENGTH,
            "Bitstring length exceeds maximum"
        );
        Self { bits, length }
    }

    /// Check if a specific bit is set
    pub fn get_bit(&self, position: usize) -> bool {
        assert!(position < self.length, "Bit position out of range");
        let mask = BigUint::from(1u32) << position;
        (&self.bits & mask) != BigUint::from(0u32)
    }
}

/// A deck of cards represented as a tuple of Elgamal ciphertexts
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CardDeck {
    /// Vector of encrypted cards
    pub cards: Vec<ElgamalCiphertext>,
    /// The modulus used for all cards
    pub modulus: BigUint,
}

impl CardDeck {
    /// Create a new card deck with the given modulus
    pub fn new(modulus: BigUint) -> Self {
        Self {
            cards: Vec::new(),
            modulus,
        }
    }

    /// Add a card to the deck
    pub fn add_card(&mut self, card: ElgamalCiphertext) {
        assert_eq!(
            card.c1.modulus, self.modulus,
            "Card must use the same modulus as the deck"
        );
        self.cards.push(card);
    }

    /// Get the number of cards in the deck
    pub fn size(&self) -> usize {
        self.cards.len()
    }
}

/// Enumeration of the different communication modes
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommunicationMode {
    /// Direct communication from one player to another
    DirectPlayerToPlayer,
    /// Broadcast from one player to all others
    Broadcast,
    /// All players to all players (unicast)
    Unicast,
}

/// A message that can be sent between players
#[derive(Clone, Debug)]
pub enum GameMessage {
    /// A bitstring message
    BitString(BitString),
    /// A single modular element
    ModularElement(ModularElement),
    /// A card deck message
    CardDeck(CardDeck),
}

/// A communication packet containing a message and its mode
#[derive(Clone, Debug)]
pub struct CommunicationPacket {
    /// The mode of communication
    pub mode: CommunicationMode,
    /// The sender's identifier
    pub sender: usize,
    /// The recipient's identifier (None for broadcast)
    pub recipient: Option<usize>,
    /// The actual message being sent
    pub message: GameMessage,
}

impl CommunicationPacket {
    /// Create a new direct player-to-player communication
    pub fn direct(sender: usize, recipient: usize, message: GameMessage) -> Self {
        Self {
            mode: CommunicationMode::DirectPlayerToPlayer,
            sender,
            recipient: Some(recipient),
            message,
        }
    }

    /// Create a new broadcast communication
    pub fn broadcast(sender: usize, message: GameMessage) -> Self {
        Self {
            mode: CommunicationMode::Broadcast,
            sender,
            recipient: None,
            message,
        }
    }

    /// Create a new unicast communication
    pub fn unicast(sender: usize, message: GameMessage) -> Self {
        Self {
            mode: CommunicationMode::Unicast,
            sender,
            recipient: None,
            message,
        }
    }
}
