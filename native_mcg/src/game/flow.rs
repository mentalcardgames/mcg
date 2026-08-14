use anyhow::Result;
use mcg_shared::{ActionEvent, GameAction, Stage};

use crate::game::Game;
use crate::poker::cards::card_str;

impl Game {
    /// After an action is applied, update the game flow (next actor, stage changes, etc).
    /// This centralized function replaces the distributed logic that was previously
    /// at the end of `apply_player_action`.
    pub fn post_action_update(&mut self, actor: usize, prev_current_bet: u32) -> Result<()> {
        // If the action was a bet or raise, the list of players who need to act changes.
        if self.current_bet > prev_current_bet {
            self.rebuild_pending_to_act(actor);
        }

        // The player who just acted is no longer pending for this street.
        self.remove_from_pending(actor);

        // Validate stack integrity after each action
        if let Some(initial_total) = self.recent_actions.first().map(|_| {
            // Use a reasonable default for total chips (players.len() * 1000)
            self.players.len() as u32 * 1000
        }) {
            if let Err(e) = self.validate_stack_consistency(initial_total) {
                eprintln!("[ERROR] Stack consistency check failed: {}", e);
                // Don't bail to avoid crashing the game, but log the error
            }
        }

        // Check for end-of-hand conditions (e.g. only one player left).
        if self.active_players().len() <= 1 {
            self.stage = mcg_shared::Stage::Showdown;
            crate::game::showdown::finish_showdown(self);
            return Ok(());
        }

        // If the betting round is complete, advance to the next stage.
        if self.is_betting_round_complete() {
            self.advance_stage()?;
            // Advancing might have led to showdown.
            if self.stage == mcg_shared::Stage::Showdown {
                crate::game::showdown::finish_showdown(self);
                return Ok(());
            }
            // Initialize the next round's betting state.
            self.init_round_for_stage();
        } else {
            // Otherwise, simply advance to the next player in the pending list.
            self.to_act = self.pending_to_act.first().copied().unwrap_or(self.to_act);
        }

        Ok(())
    }

    /// Rebuild the list of players that still need to act this street.
    /// Should be called when a bet or raise increases `current_bet`.
    fn rebuild_pending_to_act(&mut self, actor: usize) {
        let n = self.players.len();
        self.pending_to_act.clear();
        // Iterate through players starting from the one after the actor.
        for i in 1..=n {
            let idx = (actor + i) % n;
            if !self.players[idx].has_folded
                && !self.players[idx].all_in
                && self.round_bets[idx] < self.current_bet
            {
                self.pending_to_act.push(idx);
            }
        }
    }

    pub(crate) fn active_players(&self) -> Vec<usize> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| (!p.has_folded).then_some(i))
            .collect()
    }

    pub(crate) fn remove_from_pending(&mut self, actor: usize) {
        if let Some(pos) = self.pending_to_act.iter().position(|&i| i == actor) {
            let need = self.current_bet.saturating_sub(self.round_bets[actor]);
            if self.players[actor].has_folded || self.players[actor].all_in || need == 0 {
                self.pending_to_act.remove(pos);
            }
        }
    }

    /// A betting round ends when no one is left to act for this street.
    pub(crate) fn is_betting_round_complete(&self) -> bool {
        self.pending_to_act.is_empty()
    }

    /// Initialize per-street state and who acts first for each stage.
    /// For post-flop streets, round contributions are reset and the minimum
    /// future raise size starts at the big blind.
    pub(crate) fn init_round_for_stage(&mut self) {
        // For streets after preflop, reset round contributions and betting state.
        // For preflop, do NOT zero round_bets: blinds were already posted into round_bets.
        if self.stage != Stage::Preflop {
            // fill round_bets with zeros sized to players
            self.round_bets = vec![0; self.players.len()];
            self.current_bet = 0;
            self.min_raise = self.bb;
        }

        let n = self.players.len();
        let start = match self.stage {
            Stage::Preflop => {
                if n > 1 {
                    if n == 2 {
                        // Heads-up: dealer (SB) acts first preflop
                        self.dealer_idx
                    } else {
                        // 3+ players: first to act is left of BB
                        (self.dealer_idx + 3) % n
                    }
                } else {
                    self.dealer_idx
                }
            }
            Stage::Flop | Stage::Turn | Stage::River => (self.dealer_idx + 1) % n,
            Stage::Showdown => self.dealer_idx,
        };
        self.pending_to_act.clear();
        for i in 0..n {
            let idx = (start + i) % n;
            if !self.players[idx].has_folded && !self.players[idx].all_in {
                self.pending_to_act.push(idx);
            }
        }
        self.to_act = *self.pending_to_act.first().unwrap_or(&self.dealer_idx);
    }

    /// Deal community cards and advance the hand's stage, emitting appropriate logs.
    pub(crate) fn advance_stage(&mut self) -> Result<()> {
        match self.stage {
            Stage::Preflop => {
                // Flop (burn ignored for simplicity)
                let c1 = self
                    .deck
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("Deck underflow while dealing flop card 1"))?;
                let c2 = self
                    .deck
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("Deck underflow while dealing flop card 2"))?;
                let c3 = self
                    .deck
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("Deck underflow while dealing flop card 3"))?;
                self.community.push(c1);
                self.community.push(c2);
                self.community.push(c3);
                self.stage = Stage::Flop;
                self.log(ActionEvent::game(GameAction::DealtCommunity {
                    cards: self.community.clone(),
                }));
                println!(
                    "[STAGE] Flop: {} {} {}",
                    card_str(self.community[0]),
                    card_str(self.community[1]),
                    card_str(self.community[2])
                );
            }
            Stage::Flop => {
                let c = self
                    .deck
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("Deck underflow while dealing turn card"))?;
                self.community.push(c);
                self.stage = Stage::Turn;
                self.log(ActionEvent::game(GameAction::DealtCommunity {
                    cards: self.community.clone(),
                }));
                println!("[STAGE] Turn: {}", card_str(self.community[3]));
            }
            Stage::Turn => {
                let c = self
                    .deck
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("Deck underflow while dealing river card"))?;
                self.community.push(c);
                self.stage = Stage::River;
                self.log(ActionEvent::game(GameAction::DealtCommunity {
                    cards: self.community.clone(),
                }));
                println!("[STAGE] River: {}", card_str(self.community[4]));
            }
            Stage::River => {
                self.stage = Stage::Showdown;
            }
            Stage::Showdown => {}
        }
        self.log(ActionEvent::game(GameAction::StageChanged(self.stage)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::game::Game;
    use mcg_shared::{PlayerAction, Stage};

    #[test]
    fn test_post_action_update_advances_turn() {
        let mut game = Game::new_with_seed("Alice".to_string(), 2, 42).unwrap();
        let initial_to_act = game.to_act;

        // Player 0 (Alice) calls.
        let action = PlayerAction::CheckCall;
        game.apply_player_action(initial_to_act, action).unwrap();

        // The turn should advance to the next player.
        assert_ne!(game.to_act, initial_to_act, "Turn should have advanced");
    }

    #[test]
    fn test_betting_round_completion_advances_stage() {
        let mut game = Game::new_with_seed("Alice".to_string(), 1, 42).unwrap();
        assert_eq!(game.stage, Stage::Preflop);

        // Both players call, ending the pre-flop betting round.
        game.apply_player_action(0, PlayerAction::CheckCall)
            .unwrap();
        game.apply_player_action(1, PlayerAction::CheckCall)
            .unwrap();

        // The stage should advance to Flop.
        assert_eq!(
            game.stage,
            Stage::Flop,
            "Stage should have advanced to Flop"
        );
    }
}
