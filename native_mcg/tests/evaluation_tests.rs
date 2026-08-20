//! Tests for hand evaluation logic, especially tiebreaker scenarios

use mcg_shared::{Card, CardRank, CardSuit, HandRankCategory};
use native_mcg::poker::evaluation::*;

/// Test that pair tiebreakers work correctly
#[test]
fn test_pair_tiebreakers() {
    // Player 1: K♠, K♥ with community 3♦, 4♣, 9♠, 8♥, 7♦ (Kings)
    // Player 2: Q♠, Q♥ with community 3♦, 4♣, 9♠, 8♥, 7♦ (Queens)
    // Player 3: J♠, J♥ with community 3♦, 4♣, 9♠, 8♥, 7♦ (Jacks)

    let hole1 = [
        Card::new(CardRank::King, CardSuit::Clubs),
        Card::new(CardRank::King, CardSuit::Hearts),
    ]; // K♠, K♥
    let hole2 = [
        Card::new(CardRank::Queen, CardSuit::Clubs),
        Card::new(CardRank::Queen, CardSuit::Hearts),
    ]; // Q♠, Q♥
    let hole3 = [
        Card::new(CardRank::Jack, CardSuit::Clubs),
        Card::new(CardRank::Jack, CardSuit::Hearts),
    ]; // J♠, J♥

    let community = [
        Card::new(CardRank::Three, CardSuit::Diamonds), // 3♦
        Card::new(CardRank::Four, CardSuit::Diamonds),  // 4♦
        Card::new(CardRank::Nine, CardSuit::Clubs),     // 9♠
        Card::new(CardRank::Eight, CardSuit::Hearts),   // 8♥
        Card::new(CardRank::Seven, CardSuit::Spades),   // 7♠
    ];

    let rank1 = evaluate_best_hand(hole1, &community);
    let rank2 = evaluate_best_hand(hole2, &community);
    let rank3 = evaluate_best_hand(hole3, &community);

    println!("Rank1: {:?}", rank1);
    println!("Rank2: {:?}", rank2);
    println!("Rank3: {:?}", rank3);

    // All should be pairs
    assert_eq!(rank1.category, HandRankCategory::Pair);
    assert_eq!(rank2.category, HandRankCategory::Pair);
    assert_eq!(rank3.category, HandRankCategory::Pair);

    // Kings > Queens > Jacks
    assert!(rank1 > rank2); // Kings beat Queens
    assert!(rank2 > rank3); // Queens beat Jacks
    assert!(rank1 > rank3); // Kings beat Jacks

    // Check tiebreaker values
    assert_eq!(rank1.tiebreakers[0], 13); // Kings
    assert_eq!(rank2.tiebreakers[0], 12); // Queens
    assert_eq!(rank3.tiebreakers[0], 11); // Jacks
}

/// Test that two pair tiebreakers work correctly
#[test]
fn test_two_pair_tiebreakers() {
    // Player 1: A♠, K♥ with community A♥, K♦, Q♣, J♠, 2♦ (Aces and Kings)
    // Player 2: A♠, Q♥ with community A♥, K♦, Q♣, J♠, 2♦ (Aces and Queens)

    let community = [
        Card::new(CardRank::Ace, CardSuit::Hearts),     // A♥
        Card::new(CardRank::King, CardSuit::Diamonds),  // K♦
        Card::new(CardRank::Queen, CardSuit::Diamonds), // Q♦
        Card::new(CardRank::Jack, CardSuit::Clubs),     // J♠
        Card::new(CardRank::Two, CardSuit::Diamonds),   // 2♦
    ];

    let hole1 = [
        Card::new(CardRank::Ace, CardSuit::Clubs),
        Card::new(CardRank::King, CardSuit::Hearts),
    ]; // A♠, K♥
    let hole2 = [
        Card::new(CardRank::Ace, CardSuit::Clubs),
        Card::new(CardRank::Queen, CardSuit::Hearts),
    ]; // A♠, Q♥

    let rank1 = evaluate_best_hand(hole1, &community);
    let rank2 = evaluate_best_hand(hole2, &community);

    assert_eq!(rank1.category, HandRankCategory::TwoPair);
    assert_eq!(rank2.category, HandRankCategory::TwoPair);

    // Aces and Kings should beat Aces and Queens
    assert!(rank1 > rank2);
}

/// Test high card tiebreakers
#[test]
fn test_high_card_tiebreakers() {
    // Player 1: A♠, K♥ with community 9♦, 7♣, 5♠, 3♥, 2♦ (Ace high)
    // Player 2: A♦, Q♥ with community 9♦, 7♣, 5♠, 3♥, 2♦ (Ace high, Queen kicker)
    // Player 3: K♠, Q♦ with community 9♦, 7♣, 5♠, 3♥, 2♦ (King high)

    let community = [
        Card::new(CardRank::Nine, CardSuit::Diamonds),  // 9♦
        Card::new(CardRank::Seven, CardSuit::Diamonds), // 7♦
        Card::new(CardRank::Five, CardSuit::Clubs),     // 5♠
        Card::new(CardRank::Three, CardSuit::Hearts),   // 3♥
        Card::new(CardRank::Two, CardSuit::Spades),     // 2♠
    ];

    let hole1 = [
        Card::new(CardRank::Ace, CardSuit::Clubs),
        Card::new(CardRank::King, CardSuit::Hearts),
    ]; // A♠, K♥
    let hole2 = [
        Card::new(CardRank::Ace, CardSuit::Diamonds),
        Card::new(CardRank::Queen, CardSuit::Hearts),
    ]; // A♦, Q♥
    let hole3 = [
        Card::new(CardRank::King, CardSuit::Clubs),
        Card::new(CardRank::Queen, CardSuit::Diamonds),
    ]; // K♠, Q♦

    let rank1 = evaluate_best_hand(hole1, &community);
    let rank2 = evaluate_best_hand(hole2, &community);
    let rank3 = evaluate_best_hand(hole3, &community);

    println!("Rank1 (A-K high): {:?}", rank1);
    println!("Rank2 (A-Q high): {:?}", rank2);
    println!("Rank3 (K-Q high): {:?}", rank3);

    // All should be high card
    assert_eq!(rank1.category, HandRankCategory::HighCard);
    assert_eq!(rank2.category, HandRankCategory::HighCard);
    assert_eq!(rank3.category, HandRankCategory::HighCard);

    // A-K > A-Q > K-Q
    assert!(rank1 > rank2); // A-K beats A-Q (same ace, but King vs Queen kicker)
    assert!(rank2 > rank3); // A-Q beats K-Q (Ace vs King)
    assert!(rank1 > rank3); // A-K beats K-Q

    // Check tiebreaker values - should be ordered by high cards
    assert_eq!(rank1.tiebreakers[0], 14); // Ace
    assert_eq!(rank1.tiebreakers[1], 13); // King
    assert_eq!(rank2.tiebreakers[0], 14); // Ace
    assert_eq!(rank2.tiebreakers[1], 12); // Queen
    assert_eq!(rank3.tiebreakers[0], 13); // King
    assert_eq!(rank3.tiebreakers[1], 12); // Queen
}

/// Test three of a kind tiebreakers
#[test]
fn test_three_kind_tiebreakers() {
    // Player 1: A♠, A♥ with community A♦, K♣, Q♠, 2♥, 7♦
    // Player 2: K♠, K♥ with community K♦, A♣, Q♠, 2♥, 7♦

    let community1 = [
        Card::new(CardRank::Ace, CardSuit::Diamonds),  // A♦
        Card::new(CardRank::King, CardSuit::Diamonds), // K♦
        Card::new(CardRank::Queen, CardSuit::Clubs),   // Q♠
        Card::new(CardRank::Two, CardSuit::Hearts),    // 2♥
        Card::new(CardRank::Seven, CardSuit::Spades),  // 7♠
    ];

    let community2 = [
        Card::new(CardRank::King, CardSuit::Diamonds), // K♦
        Card::new(CardRank::Ace, CardSuit::Diamonds),  // A♦
        Card::new(CardRank::Queen, CardSuit::Clubs),   // Q♠
        Card::new(CardRank::Two, CardSuit::Hearts),    // 2♥
        Card::new(CardRank::Seven, CardSuit::Spades),  // 7♠
    ];

    let hole1 = [
        Card::new(CardRank::Ace, CardSuit::Clubs),
        Card::new(CardRank::Ace, CardSuit::Hearts),
    ]; // A♠, A♥
    let hole2 = [
        Card::new(CardRank::King, CardSuit::Clubs),
        Card::new(CardRank::King, CardSuit::Hearts),
    ]; // K♠, K♥

    let rank1 = evaluate_best_hand(hole1, &community1);
    let rank2 = evaluate_best_hand(hole2, &community2);

    assert_eq!(rank1.category, HandRankCategory::ThreeKind);
    assert_eq!(rank2.category, HandRankCategory::ThreeKind);

    // Trip Aces > Trip Kings
    assert!(rank1 > rank2);
    assert_eq!(rank1.tiebreakers[0], 14); // Aces
    assert_eq!(rank2.tiebreakers[0], 13); // Kings
}

/// Test that hands with same category but different kickers are ranked correctly
#[test]
fn test_same_category_different_kickers() {
    // This recreates the scenario from the bug report where
    // different pairs should not be considered equal

    let community = [
        Card::new(CardRank::Queen, CardSuit::Spades), // Q♠
        Card::new(CardRank::Ace, CardSuit::Hearts),   // A♥
        Card::new(CardRank::Two, CardSuit::Diamonds), // 2♦
        Card::new(CardRank::Jack, CardSuit::Hearts),  // J♥
        Card::new(CardRank::Eight, CardSuit::Hearts), // 8♥
    ];

    // Same hands as in the bug report
    let hole_you = [
        Card::new(CardRank::King, CardSuit::Hearts),
        Card::new(CardRank::King, CardSuit::Diamonds),
    ]; // K♥, K♦
    let hole_bot1 = [
        Card::new(CardRank::Ace, CardSuit::Diamonds),
        Card::new(CardRank::Seven, CardSuit::Clubs),
    ]; // A♦, 7♣
    let hole_bot2 = [
        Card::new(CardRank::Three, CardSuit::Clubs),
        Card::new(CardRank::Ace, CardSuit::Diamonds),
    ]; // 3♠, A♣
    let hole_bot3 = [
        Card::new(CardRank::Ten, CardSuit::Clubs),
        Card::new(CardRank::Queen, CardSuit::Hearts),
    ]; // T♠, Q♥

    let rank_you = evaluate_best_hand(hole_you, &community);
    let rank_bot1 = evaluate_best_hand(hole_bot1, &community);
    let rank_bot2 = evaluate_best_hand(hole_bot2, &community);
    let rank_bot3 = evaluate_best_hand(hole_bot3, &community);

    // All should be pairs
    assert_eq!(rank_you.category, HandRankCategory::Pair);
    assert_eq!(rank_bot1.category, HandRankCategory::Pair);
    assert_eq!(rank_bot2.category, HandRankCategory::Pair);
    assert_eq!(rank_bot3.category, HandRankCategory::Pair);

    // Rankings: Aces > Kings > Queens
    assert!(rank_bot1 > rank_you); // Aces beat Kings
    assert!(rank_bot2 > rank_you); // Aces beat Kings
    assert!(rank_you > rank_bot3); // Kings beat Queens

    // In this specific case, both ace hands should be equal since they use the same kickers
    assert_eq!(rank_bot1, rank_bot2); // Same kickers make them equal

    // Verify the tiebreaker values
    assert_eq!(rank_bot1.tiebreakers[0], 14); // Aces
    assert_eq!(rank_bot2.tiebreakers[0], 14); // Aces
    assert_eq!(rank_you.tiebreakers[0], 13); // Kings
    assert_eq!(rank_bot3.tiebreakers[0], 12); // Queens

    // Aces should have the same kickers in this specific case
    assert_eq!(rank_bot1.tiebreakers[1], rank_bot2.tiebreakers[1]);
}

/// Test straight tiebreakers (high card determines winner)
#[test]
fn test_straight_tiebreakers() {
    // Test different types of straights to ensure proper detection and ranking

    // Player 1: Ace-high straight (10-J-Q-K-A)
    let hole1 = [
        Card::new(CardRank::Ace, CardSuit::Clubs),
        Card::new(CardRank::Ten, CardSuit::Hearts),
    ]; // A♠, T♥
    let community1 = [
        Card::new(CardRank::Jack, CardSuit::Diamonds),  // J♦
        Card::new(CardRank::Queen, CardSuit::Clubs),    // Q♠
        Card::new(CardRank::King, CardSuit::Hearts),    // K♥
        Card::new(CardRank::Six, CardSuit::Spades),     // 6♠ (unused)
        Card::new(CardRank::Three, CardSuit::Diamonds), // 3♦ (unused)
    ];

    // Player 2: King-high straight (9-10-J-Q-K)
    let hole2 = [
        Card::new(CardRank::Nine, CardSuit::Diamonds),
        Card::new(CardRank::Ten, CardSuit::Hearts),
    ]; // 9♦, T♥
    let community2 = [
        Card::new(CardRank::Jack, CardSuit::Clubs),    // J♠
        Card::new(CardRank::Queen, CardSuit::Hearts),  // Q♥
        Card::new(CardRank::King, CardSuit::Diamonds), // K♦
        Card::new(CardRank::Six, CardSuit::Spades),    // 6♠ (unused)
        Card::new(CardRank::Three, CardSuit::Clubs),   // 3♠ (unused)
    ];

    // Player 3: Ace-low straight (A-2-3-4-5) - "wheel"
    let hole3 = [
        Card::new(CardRank::Ace, CardSuit::Hearts),
        Card::new(CardRank::Two, CardSuit::Clubs),
    ]; // A♥, 2♠
    let community3 = [
        Card::new(CardRank::Three, CardSuit::Diamonds), // 3♦
        Card::new(CardRank::Four, CardSuit::Clubs),     // 4♠
        Card::new(CardRank::Five, CardSuit::Hearts),    // 5♥
        Card::new(CardRank::King, CardSuit::Spades),    // K♠ (unused)
        Card::new(CardRank::Ten, CardSuit::Diamonds),   // T♦ (unused)
    ];

    let rank1 = evaluate_best_hand(hole1, &community1);
    let rank2 = evaluate_best_hand(hole2, &community2);
    let rank3 = evaluate_best_hand(hole3, &community3);

    println!("Rank1 (Ace-high straight): {:?}", rank1);
    println!("Rank2 (King-high straight): {:?}", rank2);
    println!("Rank3 (Ace-low straight): {:?}", rank3);

    // All should be straights
    assert_eq!(rank1.category, HandRankCategory::Straight);
    assert_eq!(rank2.category, HandRankCategory::Straight);
    assert_eq!(rank3.category, HandRankCategory::Straight);

    // Ace-high > King-high > Ace-low (wheel)
    assert!(rank1 > rank2); // Ace-high straight beats King-high straight
    assert!(rank2 > rank3); // King-high straight beats wheel
    assert!(rank1 > rank3); // Ace-high straight beats wheel

    // Check tiebreaker values
    assert_eq!(rank1.tiebreakers[0], 14); // Ace-high straight
    assert_eq!(rank2.tiebreakers[0], 13); // King-high straight
    assert_eq!(rank3.tiebreakers[0], 5); // Ace-low straight (5 is high card)
}

/// Test exact scenario from bug report to ensure it's fixed
#[test]
fn test_bug_report_scenario() {
    // Recreate the exact hands from the bug report:
    // Board: Q♠, A♥, 2♦, J♥, 8♥
    // You: K♥, K♦ (Pair of Kings)
    // Bot 1: A♦, 7♣ (Pair of Aces)
    // Bot 2: 3♠, A♣ (Pair of Aces)
    // Bot 3: T♠, Q♥ (Pair of Queens)

    let community = [
        Card::new(CardRank::Queen, CardSuit::Spades), // Q♠ (Spades=3, Queen=11)
        Card::new(CardRank::Ace, CardSuit::Hearts),   // A♥ (Hearts=2, Ace=0)
        Card::new(CardRank::Two, CardSuit::Diamonds), // 2♦ (Diamonds=1, 2=1)
        Card::new(CardRank::Jack, CardSuit::Hearts),  // J♥ (Hearts=2, Jack=10)
        Card::new(CardRank::Eight, CardSuit::Hearts), // 8♥ (Hearts=2, 8=7)
    ];

    let hole_you = [
        Card::new(CardRank::King, CardSuit::Hearts),
        Card::new(CardRank::King, CardSuit::Diamonds),
    ]; // K♥, K♦
    let hole_bot1 = [
        Card::new(CardRank::Ace, CardSuit::Diamonds),
        Card::new(CardRank::Seven, CardSuit::Clubs),
    ]; // A♦, 7♣
    let hole_bot2 = [
        Card::new(CardRank::Three, CardSuit::Clubs),
        Card::new(CardRank::Ace, CardSuit::Diamonds),
    ]; // 3♠, A♣
    let hole_bot3 = [
        Card::new(CardRank::Ten, CardSuit::Clubs),
        Card::new(CardRank::Queen, CardSuit::Hearts),
    ]; // T♠, Q♥

    let rank_you = evaluate_best_hand(hole_you, &community);
    let rank_bot1 = evaluate_best_hand(hole_bot1, &community);
    let rank_bot2 = evaluate_best_hand(hole_bot2, &community);
    let rank_bot3 = evaluate_best_hand(hole_bot3, &community);

    println!("You: {:?}", rank_you);
    println!("Bot 1: {:?}", rank_bot1);
    println!("Bot 2: {:?}", rank_bot2);
    println!("Bot 3: {:?}", rank_bot3);

    // Verify all are pairs
    assert_eq!(rank_you.category, HandRankCategory::Pair);
    assert_eq!(rank_bot1.category, HandRankCategory::Pair);
    assert_eq!(rank_bot2.category, HandRankCategory::Pair);
    assert_eq!(rank_bot3.category, HandRankCategory::Pair);

    // The bug was that all hands appeared equal, so let's verify they're properly ranked
    let mut ranks = [rank_you, rank_bot1, rank_bot2, rank_bot3];
    ranks.sort();

    // Should be ordered: Queens < Kings < Aces (with kickers)
    assert_eq!(ranks[0].tiebreakers[0], 12); // Queens
    assert_eq!(ranks[1].tiebreakers[0], 13); // Kings
    assert_eq!(ranks[2].tiebreakers[0], 14); // Aces
    assert_eq!(ranks[3].tiebreakers[0], 14); // Aces

    // In this specific case, both ace hands should be equal since they use the same kickers
    // from the community cards (Q, J, 8) and their other hole cards (7♣ vs 3♠) are not used
    assert_eq!(ranks[2], ranks[3]);

    // Verify that there are 2 winners with the same ace hands (tie)
    let highest = &ranks[3];
    let count_highest = ranks.iter().filter(|r| **r == *highest).count();
    assert_eq!(count_highest, 2); // Should be 2 winners with aces
}
