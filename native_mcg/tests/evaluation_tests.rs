//! Tests for hand evaluation logic, especially tiebreaker scenarios

use native_mcg::poker::evaluation::*;
use mcg_shared::{Card, HandRank, HandRankCategory};

/// Test that pair tiebreakers work correctly
#[test]
fn test_pair_tiebreakers() {
    // Player 1: K♠, K♥ with community 3♦, 4♣, 9♠, 8♥, 7♦ (Kings)
    // Player 2: Q♠, Q♥ with community 3♦, 4♣, 9♠, 8♥, 7♦ (Queens)
    // Player 3: J♠, J♥ with community 3♦, 4♣, 9♠, 8♥, 7♦ (Jacks)

    let hole1 = [Card(0 * 13 + 12), Card(2 * 13 + 12)]; // K♠, K♥
    let hole2 = [Card(0 * 13 + 11), Card(2 * 13 + 11)]; // Q♠, Q♥
    let hole3 = [Card(0 * 13 + 10), Card(2 * 13 + 10)]; // J♠, J♥

    let community = [
        Card(1 * 13 + 2),  // 3♦
        Card(1 * 13 + 3),  // 4♣
        Card(0 * 13 + 7),  // 9♠
        Card(2 * 13 + 6),  // 8♥
        Card(3 * 13 + 5),  // 7♦
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
    // Player 1: A♠, A♥, K♠ with community K♥, Q♣, Q♠, 2♦
    // Player 2: A♠, A♥, Q♠ with community K♥, Q♣, Q♠, 2♦

    let community = [
        Card(2 * 13 + 12), // K♥
        Card(1 * 13 + 11), // Q♣
        Card(0 * 13 + 11), // Q♠
        Card(1 * 13 + 1),  // 2♦
    ];

    let hole1 = [Card(0 * 13 + 0), Card(2 * 13 + 0)]; // A♠, A♥
    let hole2 = [Card(3 * 13 + 12), Card(0 * 13 + 12)]; // K♠, K♥ (for comparison)

    let rank1 = evaluate_best_hand(hole1, &community);
    let rank2 = evaluate_best_hand(hole2, &community);

    assert_eq!(rank1.category, HandRankCategory::TwoPair);
    assert_eq!(rank2.category, HandRankCategory::TwoPair);

    // Aces and Kings should beat Aces and Queens
    assert!(rank2 > rank1);
}

/// Test high card tiebreakers
#[test]
fn test_high_card_tiebreakers() {
    // Player 1: A♠, K♥ with community Q♣, J♠, 9♦, 7♣, 2♥
    // Player 2: A♠, Q♥ with community K♣, J♠, 9♦, 7♣, 2♥

    let community1 = [
        Card(1 * 13 + 11), // Q♣
        Card(0 * 13 + 10), // J♠
        Card(1 * 13 + 8),  // 9♦
        Card(1 * 13 + 6),  // 7♣
        Card(2 * 13 + 1),  // 2♥
    ];

    let community2 = [
        Card(1 * 13 + 12), // K♣
        Card(0 * 13 + 10), // J♠
        Card(1 * 13 + 8),  // 9♦
        Card(1 * 13 + 6),  // 7♣
        Card(2 * 13 + 1),  // 2♥
    ];

    let hole1 = [Card(0 * 13 + 0), Card(2 * 13 + 12)]; // A♠, K♥
    let hole2 = [Card(0 * 13 + 0), Card(2 * 13 + 11)]; // A♠, Q♥

    let rank1 = evaluate_best_hand(hole1, &community1);
    let rank2 = evaluate_best_hand(hole2, &community2);

    assert_eq!(rank1.category, HandRankCategory::HighCard);
    assert_eq!(rank2.category, HandRankCategory::HighCard);

    // Both have Ace high, but Player 1 has King kicker vs Player 2's Queen kicker
    assert_eq!(rank1.tiebreakers[0], rank2.tiebreakers[0]); // Both have Ace
    assert!(rank1.tiebreakers[1] > rank2.tiebreakers[1]); // King > Queen
    assert!(rank1 > rank2);
}

/// Test three of a kind tiebreakers
#[test]
fn test_three_kind_tiebreakers() {
    // Player 1: A♠, A♥ with community A♦, K♣, Q♠, 2♥, 7♦
    // Player 2: K♠, K♥ with community K♦, A♣, Q♠, 2♥, 7♦

    let community1 = [
        Card(1 * 13 + 0),  // A♦
        Card(1 * 13 + 12), // K♣
        Card(0 * 13 + 11), // Q♠
        Card(2 * 13 + 1),  // 2♥
        Card(3 * 13 + 6),  // 7♦
    ];

    let community2 = [
        Card(1 * 13 + 12), // K♦
        Card(1 * 13 + 0),  // A♣
        Card(0 * 13 + 11), // Q♠
        Card(2 * 13 + 1),  // 2♥
        Card(3 * 13 + 6),  // 7♦
    ];

    let hole1 = [Card(0 * 13 + 0), Card(2 * 13 + 0)]; // A♠, A♥
    let hole2 = [Card(0 * 13 + 12), Card(2 * 13 + 12)]; // K♠, K♥

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
        Card(3 * 13 + 11), // Q♠
        Card(2 * 13 + 0),  // A♥
        Card(1 * 13 + 1),  // 2♦
        Card(2 * 13 + 10), // J♥
        Card(2 * 13 + 7),  // 8♥
    ];

    // Same hands as in the bug report
    let hole_you = [Card(2 * 13 + 12), Card(1 * 13 + 12)]; // K♥, K♦
    let hole_bot1 = [Card(1 * 13 + 0), Card(0 * 13 + 6)];   // A♦, 7♣
    let hole_bot2 = [Card(0 * 13 + 2), Card(1 * 13 + 0)];  // 3♠, A♣
    let hole_bot3 = [Card(0 * 13 + 9), Card(2 * 13 + 11)]; // T♠, Q♥

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
    assert!(rank_bot1 > rank_you);  // Aces beat Kings
    assert!(rank_bot2 > rank_you);  // Aces beat Kings
    assert!(rank_you > rank_bot3);  // Kings beat Queens

    // Check that aces are not equal to each other due to different kickers
    assert_ne!(rank_bot1, rank_bot2); // Different kickers make them unequal

    // Verify the tiebreaker values
    assert_eq!(rank_bot1.tiebreakers[0], 14); // Aces
    assert_eq!(rank_bot2.tiebreakers[0], 14); // Aces
    assert_eq!(rank_you.tiebreakers[0], 13);  // Kings
    assert_eq!(rank_bot3.tiebreakers[0], 12);  // Queens

    // Aces should have different kickers
    assert_ne!(rank_bot1.tiebreakers[1], rank_bot2.tiebreakers[1]);
}

/// Test straight tiebreakers (high card determines winner)
#[test]
fn test_straight_tiebreakers() {
    // Player 1: A♠, 2♥ with community 3♦, 4♣, 5♠, 7♥, 8♦ (Ace-high straight)
    // Player 2: K♠, Q♥ with community J♦, 10♣, 9♠, 7♥, 8♦ (King-high straight)

    let community1 = [
        Card(1 * 13 + 1),  // 3♦
        Card(1 * 13 + 2),  // 4♣
        Card(0 * 13 + 3),  // 5♠
        Card(2 * 13 + 6),  // 7♥
        Card(3 * 13 + 7),  // 8♦
    ];

    let community2 = [
        Card(1 * 13 + 10), // J♦
        Card(1 * 13 + 9),  // 10♣
        Card(0 * 13 + 8),  // 9♠
        Card(2 * 13 + 6),  // 7♥
        Card(3 * 13 + 7),  // 8♦
    ];

    let hole1 = [Card(0 * 13 + 0), Card(2 * 13 + 1)]; // A♠, 2♥
    let hole2 = [Card(0 * 13 + 12), Card(2 * 13 + 11)]; // K♠, Q♥

    let rank1 = evaluate_best_hand(hole1, &community1);
    let rank2 = evaluate_best_hand(hole2, &community2);

    assert_eq!(rank1.category, HandRankCategory::Straight);
    assert_eq!(rank2.category, HandRankCategory::Straight);

    // Ace-high straight (A-5) vs King-high straight (9-K)
    // In our implementation, the highest card determines the straight
    assert!(rank2 > rank1); // King-high straight should beat Ace-low straight
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
        Card(3 * 13 + 11), // Q♠ (Spades=3, Queen=11)
        Card(2 * 13 + 0),  // A♥ (Hearts=2, Ace=0)
        Card(1 * 13 + 1),  // 2♦ (Diamonds=1, 2=1)
        Card(2 * 13 + 10), // J♥ (Hearts=2, Jack=10)
        Card(2 * 13 + 7),  // 8♥ (Hearts=2, 8=7)
    ];

    let hole_you = [Card(2 * 13 + 12), Card(1 * 13 + 12)]; // K♥, K♦
    let hole_bot1 = [Card(1 * 13 + 0), Card(0 * 13 + 6)];   // A♦, 7♣
    let hole_bot2 = [Card(0 * 13 + 2), Card(1 * 13 + 0)];  // 3♠, A♣
    let hole_bot3 = [Card(0 * 13 + 9), Card(2 * 13 + 11)]; // T♠, Q♥

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
    let mut ranks = vec![rank_you, rank_bot1, rank_bot2, rank_bot3];
    ranks.sort();

    // Should be ordered: Queens < Kings < Aces (with kickers)
    assert_eq!(ranks[0].tiebreakers[0], 12); // Queens
    assert_eq!(ranks[1].tiebreakers[0], 13); // Kings
    assert_eq!(ranks[2].tiebreakers[0], 14); // Aces
    assert_eq!(ranks[3].tiebreakers[0], 14); // Aces

    // Verify the two ace hands are different (different kickers)
    assert_ne!(ranks[2], ranks[3]);

    // Verify that the highest hand is unique (no 4-way tie)
    let highest = &ranks[3];
    let count_highest = ranks.iter().filter(|r| **r == *highest).count();
    assert_eq!(count_highest, 1); // Should be only one winner
}