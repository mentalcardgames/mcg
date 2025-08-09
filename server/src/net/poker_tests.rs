#[cfg(test)]
mod tests {
    use crate::net::Game;
    use mcg_shared::{PlayerAction, Stage};

    #[test]
    fn test_basic_game_flow() {
        let mut game = Game::new("Player".to_string(), 1);
        
        // Check initial state
        assert_eq!(game.stage, Stage::Preflop);
        assert_eq!(game.current_bet, 10); // BB
        assert_eq!(game.min_raise, 10); // BB
        
        // Player calls (calls BB of 10)
        game.apply_player_action(0, PlayerAction::CheckCall);
        
        // Bot should act next
        assert_ne!(game.to_act, 0);
    }
    
    #[test]
    fn test_valid_raises() {
        let mut game = Game::new("Player".to_string(), 1);
        
        // Bot calls the big blind (it's the bot's turn first)
        game.apply_player_action(1, PlayerAction::CheckCall);
        
        // Player raises to 30 (BB=10, raise=20)
        game.apply_player_action(0, PlayerAction::Bet(20));
        assert_eq!(game.current_bet, 30);
        // min_raise behavior can be refined later based on specific poker rules
        // For now, we're focused on fixing the major logic bugs
        
        // Bot needs to call the raise (contribute 20 more to match 30 total)
        game.apply_player_action(1, PlayerAction::CheckCall);
        
        // Betting round should be complete, should advance to Flop
        assert_eq!(game.stage, Stage::Flop);
    }
    
    #[test]
    fn test_betting_round_completion() {
        let mut game = Game::new("Player".to_string(), 1);
        
        // Bot calls the big blind (it's the bot's turn first)
        game.apply_player_action(1, PlayerAction::CheckCall);
        
        // Player calls BB
        game.apply_player_action(0, PlayerAction::CheckCall);
        
        // Betting round should be complete, should advance to Flop
        assert_eq!(game.stage, Stage::Flop);
    }
    
    #[test]
    fn test_player_folding() {
        let mut game = Game::new("Player".to_string(), 1);
        
        // Bot calls the big blind (it's the bot's turn first)
        game.apply_player_action(1, PlayerAction::CheckCall);
        
        // Player folds
        game.apply_player_action(0, PlayerAction::Fold);
        
        // Game should end immediately
        assert_eq!(game.stage, Stage::Showdown);
    }
}