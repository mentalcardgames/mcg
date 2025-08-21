//! Core Game and Player definitions + constructors and small helpers.

use anyhow::{Context, Result};
use mcg_shared::{ActionEvent, Card, GameStatePublic, PlayerPublic, Stage};
use rand::seq::SliceRandom;
use std::collections::VecDeque;

pub(crate) const MAX_RECENT_ACTIONS: usize = 50;

#[derive(Clone, Debug)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub stack: u32,
    pub cards: [Card; 2],
    pub has_folded: bool,
    pub all_in: bool,
}

#[derive(Clone, Debug)]
pub struct Game {
    // Table
    pub players: Vec<Player>,
    pub deck: VecDeque<Card>,
    pub community: Vec<Card>,

    // Betting state
    pub pot: u32,
    pub stage: Stage,
    pub dealer_idx: usize,
    pub to_act: usize,
    pub current_bet: u32,
    pub min_raise: u32,
    pub round_bets: Vec<u32>, // contributions this street, indexed by player idx

    // Blinds
    pub sb: u32,
    pub bb: u32,

    // Flow bookkeeping
    pub pending_to_act: Vec<usize>, // players that still need to act this street (non-folded, non-all-in)
    // canonical in-memory store of typed events
    pub recent_actions: Vec<ActionEvent>,
    pub winner_ids: Vec<usize>,
}

impl Game {
    pub fn with_players(players: Vec<Player>) -> Result<Self> {
        let mut deck: Vec<Card> = (0..52).map(|i| Card(i)).collect();
        deck.shuffle(&mut rand::rng());
        let player_count = players.len();

        let mut g = Self {
            players,
            deck: VecDeque::from(deck.clone()),
            community: vec![],

            pot: 0,
            stage: Stage::Preflop,
            dealer_idx: 0,
            to_act: 0,
            current_bet: 0,
            min_raise: 0,
            round_bets: vec![0; player_count],

            sb: 5,
            bb: 10,

            pending_to_act: Vec::new(),
            recent_actions: Vec::new(),
            winner_ids: Vec::new(),
        };
        // delegate dealing/init to sibling module
        super::dealing::start_new_hand_from_deck(&mut g, deck)
            .context("Failed to initialize new hand from deck")?;
        Ok(g)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new_with_seed(human_name: String, bot_count: usize, seed: u64) -> Result<Self> {
        let deck = super::dealing::shuffled_deck_with_seed(seed);

        let mut players = Vec::with_capacity(1 + bot_count);
        players.push(Player {
            id: 0,
            name: human_name,
            stack: 1000,
            cards: [Card(0), Card(0)],
            has_folded: false,
            all_in: false,
        });
        for i in 0..bot_count {
            players.push(Player {
                id: i + 1,
                name: format!("Bot {}", i + 1),
                stack: 1000,
                cards: [Card(0), Card(0)],
                has_folded: false,
                all_in: false,
            });
        }

        let mut g = Self {
            players,
            deck: VecDeque::new(),
            community: vec![],

            pot: 0,
            stage: Stage::Preflop,
            dealer_idx: 0,
            to_act: 0,
            current_bet: 0,
            min_raise: 0,
            round_bets: vec![],

            sb: 5,
            bb: 10,

            pending_to_act: Vec::new(),
            recent_actions: Vec::new(),
            winner_ids: Vec::new(),
        };
        super::dealing::start_new_hand_from_deck(&mut g, deck)
            .context("Failed to initialize new hand from deterministic deck")?;
        Ok(g)
    }

    pub fn public(&self) -> GameStatePublic {
        let players = self
            .players
            .iter()
            .enumerate()
            .map(|(idx, p)| PlayerPublic {
                id: mcg_shared::PlayerId(p.id),
                name: p.name.clone(),
                stack: p.stack,
                // Expose hole cards for all players in the public state.
                cards: Some(p.cards),
                has_folded: p.has_folded,
                bet_this_round: self.round_bets[idx],
            })
            .collect();

        GameStatePublic {
            players,
            community: self.community.clone(),
            pot: self.pot,
            sb: self.sb,
            bb: self.bb,
            to_act: mcg_shared::PlayerId(self.to_act),
            stage: self.stage,
            winner_ids: self
                .winner_ids
                .clone()
                .into_iter()
                .map(mcg_shared::PlayerId)
                .collect(),
            action_log: self.recent_actions.clone(),
        }
    }

    pub(crate) fn log(&mut self, ev: ActionEvent) {
        // canonical store is recent_actions (typed ActionEvent).
        self.recent_actions.push(ev);
        // cap logs via utils helper
        super::utils::cap_logs(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::dealing;
    use anyhow::Result;
    use std::collections::VecDeque;

    #[test]
    fn heads_up_blinds_and_dealing() -> Result<()> {
        // deterministic deck via provided helper
        let g = Game::new_with_seed("Alice".to_owned(), 1, 42)?;
        assert_eq!(g.players.len(), 2);

        // Each player should have two hole cards assigned
        assert_ne!(g.players[0].cards[0], g.players[0].cards[1]);
        assert_ne!(g.players[1].cards[0], g.players[1].cards[1]);

        // In heads-up (2 players) dealer posts SB (player 0) and other posts BB (player 1)
        let sb = g.sb;
        let bb = g.bb;
        assert_eq!(g.players[0].stack, 1000 - sb);
        assert_eq!(g.players[1].stack, 1000 - bb);

        // Pot should be sum of blinds
        assert_eq!(g.pot, sb + bb);

        // Preflop to_act for heads-up should be dealer (player 0) per dealing logic
        assert_eq!(g.to_act, 0);

        Ok(())
    }

    #[test]
    fn three_players_blinds_and_to_act() -> Result<()> {
        let g = Game::new_with_seed("Player".to_owned(), 2, 7)?;
        assert_eq!(g.players.len(), 3);
        let n = g.players.len();
        let sb_idx = (g.dealer_idx + 1) % n;
        let bb_idx = (g.dealer_idx + 2) % n;

        // round_bets should reflect blinds posted
        assert_eq!(g.round_bets[sb_idx], g.sb);
        assert_eq!(g.round_bets[bb_idx], g.bb);

        // Pot should equal sum of blinds
        assert_eq!(g.pot, g.sb + g.bb);

        // To act should be left of BB
        assert_eq!(g.to_act, (bb_idx + 1) % n);

        Ok(())
    }

    #[test]
    fn blind_caps_to_stack_and_marks_all_in() -> Result<()> {
        // Build a minimal game manually so we can set stack small before dealing.
        let deck = dealing::shuffled_deck_with_seed(123);
        let mut players = Vec::with_capacity(2);
        players.push(Player {
            id: 0,
            name: "Short".to_owned(),
            stack: 3, // less than small blind (5)
            cards: [Card(0), Card(0)],
            has_folded: false,
            all_in: false,
        });
        players.push(Player {
            id: 1,
            name: "Normal".to_owned(),
            stack: 1000,
            cards: [Card(0), Card(0)],
            has_folded: false,
            all_in: false,
        });

        let mut g = Game {
            players,
            deck: VecDeque::new(),
            community: vec![],

            pot: 0,
            stage: mcg_shared::Stage::Preflop,
            dealer_idx: 0,
            to_act: 0,
            current_bet: 0,
            min_raise: 0,
            round_bets: vec![],

            sb: 5,
            bb: 10,

            pending_to_act: Vec::new(),
            recent_actions: Vec::new(),
            winner_ids: Vec::new(),
        };

        // Start the hand using deterministic deck
        dealing::start_new_hand_from_deck(&mut g, deck)?;

        // Player 0 had stack 3 < sb 5, so should be all-in with their stack reduced to 0
        assert_eq!(g.players[0].stack, 0);
        assert!(g.players[0].all_in);

        // Their contribution should be equal to their stack (3) and pot should include both blinds
        assert_eq!(g.round_bets[0], 3);
        assert_eq!(g.pot, 3 + g.bb);

        Ok(())
    }
}
