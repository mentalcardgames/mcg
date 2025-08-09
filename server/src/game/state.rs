//! Game state management.

use crate::game::Player;
use mcg_shared::{Stage, ActionEvent, LogEntry};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct GameState {
    // Table
    pub players: Vec<Player>,
    pub deck: VecDeque<u8>,
    pub community: Vec<u8>,

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
    pub recent_actions: Vec<ActionEvent>,
    pub action_log: Vec<LogEntry>,
    pub winner_ids: Vec<usize>,
}

impl GameState {
    pub fn new(human_name: String, bot_count: usize) -> Self {
        let mut deck: Vec<u8> = (0..52).collect();
        use rand::seq::SliceRandom;
        deck.shuffle(&mut rand::rng());

        let mut players = Vec::with_capacity(1 + bot_count);
        players.push(Player {
            id: 0,
            name: human_name,
            stack: 1000,
            cards: [0, 0],
            has_folded: false,
            all_in: false,
        });
        for i in 0..bot_count {
            players.push(Player {
                id: i + 1,
                name: format!("Bot {}", i + 1),
                stack: 1000,
                cards: [0, 0],
                has_folded: false,
                all_in: false,
            });
        }

        let mut g = Self {
            players,
            deck: VecDeque::from(deck),
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
            action_log: Vec::new(),
            winner_ids: Vec::new(),
        };
        g.start_new_hand();
        g
    }
}