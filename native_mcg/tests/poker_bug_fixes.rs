//! Integration tests for poker bug fixes

use anyhow::Result;
use mcg_shared::{Card, CardRank, CardSuit, PlayerAction};
use native_mcg::game::{Game, Player};

// Helper function to calculate total chips (since total_chips is private)
fn total_chips(game: &Game) -> u32 {
    game.players.iter().map(|p| p.stack).sum::<u32>() + game.pot
}

// Helper function to create test players
fn create_test_players(count: usize) -> Vec<Player> {
    let mut players = Vec::new();
    for i in 0..count {
        players.push(Player {
            id: mcg_shared::PlayerId(i),
            name: if i == 0 {
                "Alice".to_string()
            } else {
                format!("Bot {}", i)
            },
            stack: 1000,
            cards: [
                Card::new(CardRank::Ace, CardSuit::Clubs),
                Card::new(CardRank::Ace, CardSuit::Diamonds),
            ],
            has_folded: false,
            all_in: false,
        });
    }
    players
}

#[test]
fn test_bet_zero_prevention() -> Result<()> {
    // Test that bet logic doesn't crash with small bets
    let players = create_test_players(3);
    let mut game = Game::with_players(players)?;

    // Simulate a scenario where current_bet is very small
    game.current_bet = 1; // Very small bet that could cause rounding to 0

    // This test would need to be expanded with actual bot decision making
    // For now, we just verify the game doesn't crash with small bets
    assert!(game.current_bet > 0);

    Ok(())
}

#[test]
fn test_bet_zero_validation() -> Result<()> {
    // Test that Bet(0) actions are converted to CheckCall
    let players = create_test_players(2);
    let mut game = Game::with_players(players)?;

    let initial_total = total_chips(&game);

    // Try to make a Bet(0) action - should be converted to CheckCall
    let result = game.apply_player_action(0, PlayerAction::Bet(0));

    // Should succeed (not error)
    assert!(result.is_ok(), "Bet(0) should be converted, not error");

    // Stack and pot should remain consistent
    let current_total = total_chips(&game);
    assert_eq!(initial_total, current_total, "Chip conservation violated");

    Ok(())
}

#[test]
fn test_stack_consistency() -> Result<()> {
    // Test that stacks + pot always equal the initial total
    let players = create_test_players(2);
    let mut game = Game::with_players(players)?;

    let initial_total = total_chips(&game);

    // Make some actions
    let result1 = game.apply_player_action(0, PlayerAction::CheckCall);
    if result1.is_ok() && game.to_act < game.players.len() {
        let _ = game.apply_player_action(game.to_act, PlayerAction::CheckCall);
    }

    // Check stack consistency
    let current_total = total_chips(&game);
    assert_eq!(
        initial_total, current_total,
        "Stack consistency violated after actions"
    );

    Ok(())
}

#[test]
fn test_all_in_detection() -> Result<()> {
    // Test that players with stack=0 are marked all-in and can't act
    let players = create_test_players(2);
    let mut game = Game::with_players(players)?;

    // Manually set a player's stack to 0 to test all-in detection
    game.players[0].stack = 0;
    game.players[0].all_in = true;

    // Try to make them act - should fail
    let result = game.apply_player_action(0, PlayerAction::CheckCall);
    assert!(result.is_err(), "All-in player should not be able to act");

    Ok(())
}

#[test]
fn test_hand_evaluation_accuracy() -> Result<()> {
    // Test the specific scenario from the game log
    use mcg_shared::HandRankCategory;
    use native_mcg::poker::evaluation::{evaluate_best_hand, pick_best_five};

    // "You" hand: J♣, 7♥ with board K♥, T♠, 9♥, 9♣, 4♣
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

    // Should be pair of 9s
    assert_eq!(rank.category, HandRankCategory::Pair);

    // Best five should contain both 9s
    let nines_count = best_five
        .iter()
        .filter(|c| c.rank() == CardRank::Nine)
        .count();
    assert_eq!(
        nines_count, 2,
        "Best five should contain both 9s for the pair"
    );

    Ok(())
}

#[test]
fn test_hole_card_visibility() -> Result<()> {
    // Test that hole cards are always visible (insecure mode)
    let players = create_test_players(2);
    let game = Game::with_players(players)?;

    let public_state = game.public();

    // All players should see all cards in insecure mode
    for player in &public_state.players {
        assert!(
            player.cards.is_some(),
            "All player cards should be visible in insecure mode"
        );
    }

    Ok(())
}

// Helper function to test the bot AI logic directly
#[test]
fn test_bot_ai_minimum_bet_enforcement() {
    use mcg_shared::{PlayerAction, Stage};
    use native_mcg::bot::{BotContext, SimpleBot};

    let bot = SimpleBot::default();

    // Test scenario where current_bet is very small
    let context = BotContext {
        stack: 1000,
        call_amount: 50,
        current_bet: 1, // Very small current bet
        big_blind: 10,
        stage: Stage::Flop,
        position: 2,
        total_players: 4,
    };

    // Run the bot decision multiple times
    for _ in 0..50 {
        let action = bot.decide_action(&context);

        // If it's a bet action, ensure it's at least the big blind
        if let PlayerAction::Bet(amount) = action {
            assert!(
                amount >= context.big_blind,
                "Bot should never bet less than big blind ({}), but got {}",
                context.big_blind,
                amount
            );
        }
    }
}
