//! Core Game and Player definitions + constructors and small helpers.

use mcg_shared::{ActionEvent, GameStatePublic, LogEntry, PlayerPublic, Stage};
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use anyhow::{Result, Context};

pub(crate) const MAX_RECENT_ACTIONS: usize = 50;
pub(crate) const MAX_LOG_ENTRIES: usize = 200;

#[derive(Clone, Debug)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub stack: u32,
    pub cards: [u8; 2],
    pub has_folded: bool,
    pub all_in: bool,
}

#[derive(Clone, Debug)]
pub struct Game {
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

impl Game {
    pub fn new(human_name: String, bot_count: usize) -> Result<Self> {
        let mut deck: Vec<u8> = (0..52).collect();
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
            deck: VecDeque::from(deck.clone()),
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
        // delegate dealing/init to sibling module
        super::dealing::start_new_hand_from_deck(&mut g, deck)
            .context("Failed to initialize new hand from deck")?;
        Ok(g)
    }

    #[cfg(test)]
    pub fn new_with_seed(human_name: String, bot_count: usize, seed: u64) -> Result<Self> {
        let deck = super::dealing::shuffled_deck_with_seed(seed);
 
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
            action_log: Vec::new(),
            winner_ids: Vec::new(),
        };
        super::dealing::start_new_hand_from_deck(&mut g, deck)
            .context("Failed to initialize new hand from deterministic deck")?;
        Ok(g)
    }

    pub fn public_for(&self, viewer_id: usize) -> GameStatePublic {
        let players = self
            .players
            .iter()
            .map(|p| PlayerPublic {
                id: p.id,
                name: p.name.clone(),
                stack: p.stack,
                cards: if p.id == viewer_id {
                    Some(p.cards)
                } else {
                    None
                },
                has_folded: p.has_folded,
            })
            .collect();

        GameStatePublic {
            players,
            community: self.community.clone(),
            pot: self.pot,
            to_act: self.to_act,
            stage: self.stage,
            you_id: viewer_id,
            bot_count: self.players.len().saturating_sub(1),
            recent_actions: self.recent_actions.clone(),
            winner_ids: self.winner_ids.clone(),
            action_log: self.action_log.clone(),
        }
    }

    pub(crate) fn log(&mut self, entry: LogEntry) {
        self.action_log.push(entry);
        // cap logs via utils helper
        super::utils::cap_logs(self);
    }
}
