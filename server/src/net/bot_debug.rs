#[cfg(test)]
mod tests {
    use crate::net::{Game, Player};
    use mcg_shared::{PlayerAction, Stage};

    #[test]
    fn debug_bot_behavior() {
        let mut game = Game::new("Player".to_string(), 1);
        
        println!("Initial state:");
        println!("  Current bet: {}", game.current_bet);
        println!("  Bot round_bets: {}", game.round_bets[1]);
        println!("  Bot need: {}", game.current_bet.saturating_sub(game.round_bets[1]));
        
        // Bot calls the big blind
        game.apply_player_action(1, PlayerAction::CheckCall);
        
        println!("After bot calls BB:");
        println!("  Current bet: {}", game.current_bet);
        println!("  Bot round_bets: {}", game.round_bets[1]);
        println!("  Bot need: {}", game.current_bet.saturating_sub(game.round_bets[1]));
        
        // Player bets 10 more
        game.apply_player_action(0, PlayerAction::Bet(10));
        
        println!("After player bets 10 more:");
        println!("  Current bet: {}", game.current_bet);
        println!("  Bot round_bets: {}", game.round_bets[1]);
        println!("  Bot need: {}", game.current_bet.saturating_sub(game.round_bets[1]));
    }
}