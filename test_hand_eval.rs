// Test for the specific scenario from the game log
use mcg_shared::{Card, CardRank, CardSuit};
use native_mcg::poker::evaluation::{evaluate_best_hand, pick_best_five};

fn main() {
    // Scenario from game log:
    // Your hole cards: J♣, 7♥
    // Board: K♥, T♠, 9♥, 9♣, 4♣
    // Should be Two Pair (9s and Ks), not just Pair
    
    let hole = [
        Card::new(CardRank::Jack, CardSuit::Clubs),   // J♣
        Card::new(CardRank::Seven, CardSuit::Hearts), // 7♥
    ];
    
    let community = vec![
        Card::new(CardRank::King, CardSuit::Hearts),  // K♥
        Card::new(CardRank::Ten, CardSuit::Spades),   // T♠
        Card::new(CardRank::Nine, CardSuit::Hearts),  // 9♥
        Card::new(CardRank::Nine, CardSuit::Clubs),   // 9♣
        Card::new(CardRank::Four, CardSuit::Clubs),   // 4♣
    ];
    
    let rank = evaluate_best_hand(hole, &community);
    let best_five = pick_best_five(hole, &community);
    
    println!("Hole cards: J♣, 7♥");
    println!("Community: K♥, T♠, 9♥, 9♣, 4♣");
    println!("Evaluation: {:?}", rank);
    println!("Best five: {:?}", best_five);
    
    // The best hand should be: K♥, K♥, 9♥, 9♣, J♣ (two pair, Kings and Nines, Jack kicker)
    // Wait, there's only one King on the board - so it should be pair of 9s with K, J, T kickers
}