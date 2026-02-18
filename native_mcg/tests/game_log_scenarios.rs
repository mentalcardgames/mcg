//! Test for the specific hand evaluation scenarios from the game log

use mcg_shared::{Card, CardRank, CardSuit, HandRankCategory};
use native_mcg::poker::evaluation::{evaluate_best_hand, pick_best_five};

#[test]
fn test_game_log_scenario() {
    // Test the "You" hand from game log:
    // Your hole cards: J♣, 7♥
    // Board: K♥, T♠, 9♥, 9♣, 4♣
    // Should be Pair of 9s

    let hole = [
        Card::new(CardRank::Jack, CardSuit::Clubs),   // J♣
        Card::new(CardRank::Seven, CardSuit::Hearts), // 7♥
    ];

    let community = vec![
        Card::new(CardRank::King, CardSuit::Hearts), // K♥
        Card::new(CardRank::Ten, CardSuit::Spades),  // T♠
        Card::new(CardRank::Nine, CardSuit::Hearts), // 9♥
        Card::new(CardRank::Nine, CardSuit::Clubs),  // 9♣
        Card::new(CardRank::Four, CardSuit::Clubs),  // 4♣
    ];

    let rank = evaluate_best_hand(hole, &community);
    let best_five = pick_best_five(hole, &community);

    println!("Hole cards: J♣, 7♥");
    println!("Community: K♥, T♠, 9♥, 9♣, 4♣");
    println!("Evaluation: {:?}", rank);
    println!("Best five: {:?}", best_five);

    // Should be pair of 9s
    assert_eq!(rank.category, HandRankCategory::Pair);
    // Tiebreakers should be [9, 13, 11, 10] (pair of 9s, then K, J, T kickers)
    assert_eq!(rank.tiebreakers, vec![9, 13, 11, 10]);
}

#[test]
fn test_two_pair_scenario() {
    // Test a clear two-pair scenario
    // Hole: K♣, 4♠
    // Board: K♥, T♠, 9♥, 9♣, 4♣
    // Should be Two Pair (Ks and 9s)

    let hole = [
        Card::new(CardRank::King, CardSuit::Clubs),  // K♣
        Card::new(CardRank::Four, CardSuit::Spades), // 4♠
    ];

    let community = vec![
        Card::new(CardRank::King, CardSuit::Hearts), // K♥
        Card::new(CardRank::Ten, CardSuit::Spades),  // T♠
        Card::new(CardRank::Nine, CardSuit::Hearts), // 9♥
        Card::new(CardRank::Nine, CardSuit::Clubs),  // 9♣
        Card::new(CardRank::Four, CardSuit::Clubs),  // 4♣
    ];

    let rank = evaluate_best_hand(hole, &community);
    let best_five = pick_best_five(hole, &community);

    println!("Hole cards: K♣, 4♠");
    println!("Community: K♥, T♠, 9♥, 9♣, 4♣");
    println!("Evaluation: {:?}", rank);
    println!("Best five: {:?}", best_five);

    // Should be two pair (Ks and 9s with T kicker)
    assert_eq!(rank.category, HandRankCategory::TwoPair);
    // Tiebreakers should be [13, 9, 10] (pair of Ks, pair of 9s, T kicker)
    assert_eq!(rank.tiebreakers, vec![13, 9, 10]);
}
