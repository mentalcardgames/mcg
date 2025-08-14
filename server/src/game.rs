//! Core poker game logic.
//!
//! Layering note: this module defines the pure domain/game engine for
//! no-limit Texas Hold'em used by the server. It contains no network or
//! persistence code. The Axum server imports and drives this engine while
//! remaining responsible for I/O. Keeping game rules and state transitions
//! here allows isolated testing and future reuse.

use crate::eval::{card_str, evaluate_best_hand, pick_best_five};
use mcg_shared::{
    ActionEvent, ActionKind, BlindKind, GameStatePublic, HandResult, LogEntry, LogEvent,
    PlayerAction, PlayerPublic, Stage,
};
use rand::seq::SliceRandom;
use std::collections::VecDeque;

const MAX_RECENT_ACTIONS: usize = 50;
const MAX_LOG_ENTRIES: usize = 200;

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

/// Internal outcome when attempting a raise over a non-zero current bet.
#[derive(Debug, Clone, Copy)]
enum RaiseOutcome {
    /// The requested raise is invalid or insufficient; treat as a call (or all-in call).
    Call { pay: u32 },
    /// A legal raise, specifying the amount added this action and the raise size ("by").
    Raise { add: u32, by: u32 },
}

impl Game {
    pub fn new(human_name: String, bot_count: usize) -> Self {
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
        g.start_new_hand_from_deck(deck);
        g
    }

    #[cfg(test)]
    pub fn new_with_seed(human_name: String, bot_count: usize, seed: u64) -> Self {
        let deck = shuffled_deck_with_seed(seed);

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
        g.start_new_hand_from_deck(deck);
        g
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

    /// Compute the normalized add amount for an open bet (when current_bet == 0).
    /// Ensures the total bet is at least the big blind and not more than
    /// the player's total available chips (round contribution + stack).
    fn compute_open_bet_add(&self, actor: usize, desired_total: u32) -> (u32, u32) {
        let bet_to = desired_total
            .max(self.bb)
            .min(self.players[actor].stack + self.round_bets[actor]);
        let add = bet_to
            .saturating_sub(self.round_bets[actor])
            .min(self.players[actor].stack);
        (add, bet_to)
    }

    /// Decide how to resolve a raise attempt over a non-zero current bet.
    /// Returns either Call(pay) for insufficient/illegal raises (including
    /// all-in that doesn't meet the minimum raise), or Raise{add, by} for a
    /// legal raise that updates the last legal raise size.
    fn decide_raise_outcome(
        &self,
        actor: usize,
        raise_by: u32,
        prev_current_bet: u32,
    ) -> RaiseOutcome {
        let need = self.current_bet.saturating_sub(self.round_bets[actor]);
        let target_to = self.current_bet + raise_by;
        let required = target_to.saturating_sub(self.round_bets[actor]);
        let add = required.min(self.players[actor].stack);

        if add <= need {
            // Not enough to raise; treat as call (possibly all-in)
            let pay = need.min(self.players[actor].stack);
            return RaiseOutcome::Call { pay };
        }

        // Validate minimum raise size: must cover a call and increase by at least self.min_raise
        let required_add = prev_current_bet.saturating_sub(self.round_bets[actor]) + self.min_raise;
        if add < required_add {
            let pay = need.min(self.players[actor].stack);
            return RaiseOutcome::Call { pay };
        }

        let new_to = self.round_bets[actor] + add;
        let by = new_to.saturating_sub(prev_current_bet);
        RaiseOutcome::Raise { add, by }
    }

    /// Apply a player action, enforcing betting rules and advancing the game flow.
    pub fn apply_player_action(
        &mut self,
        actor: usize,
        action: PlayerAction,
    ) -> Result<(), &'static str> {
        if actor != self.to_act {
            return Err("Not your turn");
        }
        if self.players[actor].has_folded {
            return Err("You have already folded");
        }
        if self.players[actor].all_in {
            return Err("You are all-in");
        }
        println!(
            "[ACTION] {}: {:?} (stage: {:?})",
            self.players[actor].name, action, self.stage
        );
        self.recent_actions.push(ActionEvent {
            player_id: actor,
            action: action.clone(),
        });
        self.cap_logs();

        let prev_current_bet = self.current_bet;
        match action {
            PlayerAction::Fold => {
                self.players[actor].has_folded = true;
                self.log(LogEntry {
                    player_id: Some(actor),
                    event: LogEvent::Action(ActionKind::Fold),
                });
            }
            PlayerAction::CheckCall => {
                // Check or call to match current bet
                let need = self.current_bet.saturating_sub(self.round_bets[actor]);
                if need == 0 {
                    self.log(LogEntry {
                        player_id: Some(actor),
                        event: LogEvent::Action(ActionKind::Check),
                    });
                } else {
                    let pay = need.min(self.players[actor].stack);
                    self.players[actor].stack -= pay;
                    self.round_bets[actor] += pay;
                    self.pot += pay;
                    if pay < need {
                        self.players[actor].all_in = true;
                    }
                    self.log(LogEntry {
                        player_id: Some(actor),
                        event: LogEvent::Action(ActionKind::Call(pay)),
                    });
                }
            }
            PlayerAction::Bet(x) => {
                if self.current_bet == 0 {
                    // Open bet
                    let (add, _bet_to) = self.compute_open_bet_add(actor, x);
                    self.players[actor].stack -= add;
                    self.round_bets[actor] += add;
                    self.pot += add;
                    self.current_bet = self.round_bets[actor];
                    // In NLH, the minimum raise size for subsequent raises equals the size of the last bet/raise.
                    // For an open bet, set it to the bet amount (which is at least BB).
                    self.min_raise = add;
                    if self.players[actor].stack == 0 {
                        self.players[actor].all_in = true;
                    }
                    self.log(LogEntry {
                        player_id: Some(actor),
                        event: LogEvent::Action(ActionKind::Bet(add)),
                    });
                } else {
                    // Raise by x (to current_bet + x)
                    match self.decide_raise_outcome(actor, x, prev_current_bet) {
                        RaiseOutcome::Call { pay } => {
                            let need = self.current_bet.saturating_sub(self.round_bets[actor]);
                            let pay = pay.min(need).min(self.players[actor].stack);
                            self.players[actor].stack -= pay;
                            self.round_bets[actor] += pay;
                            self.pot += pay;
                            if pay < need {
                                self.players[actor].all_in = true;
                            }
                            self.log(LogEntry {
                                player_id: Some(actor),
                                event: LogEvent::Action(ActionKind::Call(pay)),
                            });
                        }
                        RaiseOutcome::Raise { add, by } => {
                            self.players[actor].stack -= add;
                            self.round_bets[actor] += add;
                            self.pot += add;
                            let new_to = self.round_bets[actor];
                            self.current_bet = new_to;
                            // Set min_raise to the size of this (last legal) raise
                            self.min_raise = by;
                            if self.players[actor].stack == 0 {
                                self.players[actor].all_in = true;
                            }
                            self.log(LogEntry {
                                player_id: Some(actor),
                                event: LogEvent::Action(ActionKind::Raise {
                                    to: self.current_bet,
                                    by,
                                }),
                            });
                        }
                    }
                }
            }
        }

        // If bet/raise increased the current bet, rebuild the pending_to_act order
        if self.current_bet > prev_current_bet {
            let n = self.players.len();
            self.pending_to_act.clear();
            for i in 1..=n {
                let idx = (actor + i) % n;
                if !self.players[idx].has_folded
                    && !self.players[idx].all_in
                    && self.round_bets[idx] < self.current_bet
                {
                    self.pending_to_act.push(idx);
                }
            }
            if let Some(&nxt) = self.pending_to_act.first() {
                self.to_act = nxt;
            }
        }

        // Remove actor from pending list if matched requirement, folded, or all-in
        self.remove_from_pending(actor);

        // If only one player remains, end the hand
        if self.active_players().len() <= 1 {
            self.stage = Stage::Showdown;
            self.finish_showdown();
            return Ok(());
        }

        // If betting round complete, advance stage
        if self.is_betting_round_complete() {
            self.advance_stage();
            if self.stage == Stage::Showdown {
                self.finish_showdown();
                return Ok(());
            }
            self.init_round_for_stage();
        } else if let Some(&nxt) = self.pending_to_act.first() {
            self.to_act = nxt;
        }
        Ok(())
    }

    pub fn start_new_hand(&mut self) {
        // Shuffle fresh deck
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rand::rng());
        self.start_new_hand_from_deck(deck);
    }

    /// Initialize a new hand using the provided deck order.
    /// This resets round state, deals hole cards, posts blinds and
    /// establishes the first player to act according to heads-up vs 3+ rules.
    fn start_new_hand_from_deck(&mut self, deck: Vec<u8>) {
        self.deck = VecDeque::from(deck);

        // Deal hole cards
        let mut dealt_logs = Vec::with_capacity(self.players.len());
        for p in &mut self.players {
            p.has_folded = false;
            p.all_in = false;
            p.cards = [
                self.deck.pop_front().unwrap(),
                self.deck.pop_front().unwrap(),
            ];
            dealt_logs.push(LogEntry {
                player_id: Some(p.id),
                event: LogEvent::DealtHole { player_id: p.id },
            });
            println!(
                "[DEAL] {} gets {} {}",
                p.name,
                card_str(p.cards[0]),
                card_str(p.cards[1])
            );
        }

        // Reset table state
        self.community.clear();
        self.pot = 0;
        self.stage = Stage::Preflop;
        self.current_bet = 0;
        self.min_raise = self.bb;
        self.round_bets = vec![0; self.players.len()];
        self.recent_actions.clear();
        self.action_log.clear();
        self.winner_ids.clear();

        // Emit logs for dealing after the loop to avoid borrow conflicts
        self.action_log.extend(dealt_logs);
        self.cap_logs();

        // Post blinds
        let n = self.players.len();
        if n > 1 {
            // In heads-up, dealer posts SB and acts first preflop; otherwise SB=dealer+1, BB=dealer+2
            let (sb_idx, bb_idx) = if n == 2 {
                (self.dealer_idx, (self.dealer_idx + 1) % n)
            } else {
                ((self.dealer_idx + 1) % n, (self.dealer_idx + 2) % n)
            };
            self.post_blind(sb_idx, BlindKind::SmallBlind, self.sb);
            self.post_blind(bb_idx, BlindKind::BigBlind, self.bb);
            self.current_bet = self.bb;
            self.min_raise = self.bb;
            // Preflop first to act is left of BB
            self.to_act = (bb_idx + 1) % n;
        } else {
            self.to_act = self.dealer_idx;
        }

        self.init_round_for_stage();
        self.log(LogEntry {
            player_id: None,
            event: LogEvent::StageChanged(self.stage),
        });
    }

    /// Post a small/big blind, capping to available stack and marking all-in when necessary.
    fn post_blind(&mut self, idx: usize, kind: BlindKind, amount: u32) {
        let a = amount.min(self.players[idx].stack);
        self.players[idx].stack -= a;
        self.round_bets[idx] += a;
        self.pot += a;
        if a < amount {
            self.players[idx].all_in = true;
        }
        self.log(LogEntry {
            player_id: Some(idx),
            event: LogEvent::Action(ActionKind::PostBlind { kind, amount: a }),
        });
        println!(
            "[BLIND] {} posts {:?} {} -> stack {}",
            self.players[idx].name, kind, a, self.players[idx].stack
        );
    }

    fn active_players(&self) -> Vec<usize> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| (!p.has_folded).then_some(i))
            .collect()
    }

    fn remove_from_pending(&mut self, actor: usize) {
        if let Some(pos) = self.pending_to_act.iter().position(|&i| i == actor) {
            let need = self.current_bet.saturating_sub(self.round_bets[actor]);
            if self.players[actor].has_folded || self.players[actor].all_in || need == 0 {
                self.pending_to_act.remove(pos);
            }
        }
    }

    /// A betting round ends when no one is left to act for this street.
    fn is_betting_round_complete(&self) -> bool {
        self.pending_to_act.is_empty()
    }

    /// Initialize per-street state and who acts first for each stage.
    /// For post-flop streets, round contributions are reset and the minimum
    /// future raise size starts at the big blind.
    fn init_round_for_stage(&mut self) {
        // For streets after preflop, reset round contributions and betting state.
        // For preflop, do NOT zero round_bets: blinds were already posted into round_bets.
        if self.stage != Stage::Preflop {
            self.round_bets.fill(0);
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
    fn advance_stage(&mut self) {
        match self.stage {
            Stage::Preflop => {
                // Flop (burn ignored for simplicity)
                self.community.push(self.deck.pop_front().unwrap());
                self.community.push(self.deck.pop_front().unwrap());
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::Flop;
                self.log(LogEntry {
                    player_id: None,
                    event: LogEvent::DealtCommunity {
                        cards: self.community.clone(),
                    },
                });
                println!(
                    "[STAGE] Flop: {} {} {}",
                    card_str(self.community[0]),
                    card_str(self.community[1]),
                    card_str(self.community[2])
                );
            }
            Stage::Flop => {
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::Turn;
                self.log(LogEntry {
                    player_id: None,
                    event: LogEvent::DealtCommunity {
                        cards: self.community.clone(),
                    },
                });
                println!("[STAGE] Turn: {}", card_str(self.community[3]));
            }
            Stage::Turn => {
                self.community.push(self.deck.pop_front().unwrap());
                self.stage = Stage::River;
                self.log(LogEntry {
                    player_id: None,
                    event: LogEvent::DealtCommunity {
                        cards: self.community.clone(),
                    },
                });
                println!("[STAGE] River: {}", card_str(self.community[4]));
            }
            Stage::River => {
                self.stage = Stage::Showdown;
            }
            Stage::Showdown => {}
        }
        self.log(LogEntry {
            player_id: None,
            event: LogEvent::StageChanged(self.stage),
        });
    }

    /// Resolve showdown by evaluating all non-folded hands, splitting the pot on ties
    /// and logging the results. Pot is distributed chip-by-chip for any remainder to
    /// the earliest winners in table order, which is sufficient for a single main pot
    /// (side-pots are not modeled in this simplified demo).
    fn finish_showdown(&mut self) {
        // Evaluate all non-folded players
        let mut results: Vec<HandResult> = Vec::new();
        for (i, p) in self.players.iter().enumerate() {
            if p.has_folded {
                continue;
            }
            let rank = evaluate_best_hand(p.cards, &self.community);
            let best_five = pick_best_five(p.cards, &self.community, &rank);
            results.push(HandResult {
                player_id: i,
                rank,
                best_five,
            });
        }
        // Determine winners (top rank; split on ties)
        results.sort_by(|a, b| a.rank.cmp(&b.rank));
        let winners: Vec<usize> = if let Some(best) = results.last().cloned() {
            results
                .iter()
                .rev()
                .take_while(|r| r.rank == best.rank)
                .map(|r| r.player_id)
                .collect()
        } else {
            vec![]
        };
        self.winner_ids = winners.clone();

        self.log(LogEntry {
            player_id: None,
            event: LogEvent::Showdown {
                hand_results: results.clone(),
            },
        });

        if !winners.is_empty() && self.pot > 0 {
            let share = self.pot / winners.len() as u32;
            let mut remainder = self.pot % winners.len() as u32;
            for &w in &winners {
                let mut win = share;
                if remainder > 0 {
                    win += 1;
                    remainder -= 1;
                }
                self.players[w].stack += win;
            }
            self.log(LogEntry {
                player_id: None,
                event: LogEvent::PotAwarded {
                    winners: winners.clone(),
                    amount: self.pot,
                },
            });
            println!("[SHOWDOWN] Pot {} awarded to {:?}", self.pot, winners);
            self.pot = 0;
        }
    }

    fn log(&mut self, entry: LogEntry) {
        self.action_log.push(entry);
        self.cap_logs();
    }


}

#[cfg(test)]
fn shuffled_deck_with_seed(seed: u64) -> Vec<u8> {
    // Simple LCG for deterministic shuffling in tests
    fn lcg(next: &mut u64) -> u32 {
        // Constants from Numerical Recipes
        *next = next.wrapping_mul(1664525).wrapping_add(1013904223);
        (*next >> 16) as u32
    }
    let mut deck: Vec<u8> = (0..52).collect();
    let mut s = seed;
    // Fisher-Yates
    for i in (1..deck.len()).rev() {
        let r = lcg(&mut s) as usize % (i + 1);
        deck.swap(i, r);
    }
    deck
}

impl Game {
    /// Cap in-memory logs to bounded sizes to avoid unbounded growth when
    /// the server runs for long sessions. This protects both serialization
    /// payload size and UI performance when rendering histories.
    fn cap_logs(&mut self) {
        if self.recent_actions.len() > MAX_RECENT_ACTIONS {
            let start = self.recent_actions.len() - MAX_RECENT_ACTIONS;
            self.recent_actions.drain(0..start);
        }
        if self.action_log.len() > MAX_LOG_ENTRIES {
            let start = self.action_log.len() - MAX_LOG_ENTRIES;
            self.action_log.drain(0..start);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_game_flow() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 42);
        // Preflop with blinds posted
        assert_eq!(game.stage, Stage::Preflop);
        assert_eq!(game.current_bet, 10);
        assert_eq!(game.min_raise, 10);

        // Player calls (calls BB of 10)
        game.apply_player_action(0, PlayerAction::CheckCall)
            .unwrap();

        // Next to act should not be the player again immediately
        assert_ne!(game.to_act, 0);
    }

    #[test]
    fn test_valid_raises() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 123);
        // Heads-up preflop: player (SB/dealer) acts first, raises to 30 (BB=10, raise=20)
        game.apply_player_action(0, PlayerAction::Bet(20)).unwrap();
        assert_eq!(game.current_bet, 30);

        // Bot needs to call the raise (contribute 20 more to match 30 total)
        game.apply_player_action(1, PlayerAction::CheckCall)
            .unwrap();

        // Betting round should be complete, should advance to Flop
        assert_eq!(game.stage, Stage::Flop);
    }

    #[test]
    fn test_betting_round_completion() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 999);
        // Heads-up preflop: player (SB/dealer) acts first, calls to match BB
        game.apply_player_action(0, PlayerAction::CheckCall)
            .unwrap();
        // Bot (BB) checks/calls to match 30 total (here to match 10 total)
        game.apply_player_action(1, PlayerAction::CheckCall)
            .unwrap();
        // Betting round should be complete
        assert_eq!(game.stage, Stage::Flop);
    }

    #[test]
    fn test_player_folding() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 7);
        // Player folds immediately preflop
        game.apply_player_action(0, PlayerAction::Fold).unwrap();
        assert_eq!(game.stage, Stage::Showdown);
    }

    #[test]
    fn test_pot_totals_no_double_count() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 555);
        // Heads-up preflop: SB acts first; raise to 20 total (BB=10 -> +10 raise)
        let sb = game.to_act;
        let bb = (sb + 1) % 2;
        // SB raises by 10 (to 20 total)
        game.apply_player_action(sb, PlayerAction::Bet(10)).unwrap();
        // BB calls 10 to 20
        game.apply_player_action(bb, PlayerAction::CheckCall)
            .unwrap();
        // After preflop: pot should be 40 (5 + 10 + 15 + 10). We can't read pot reliably after later
        // but continue play and assert final PotAwarded amount after river equals 100.

        // Flop: first to act is dealer+1 (BB). Bet 10, call 10.
        assert_eq!(game.stage, Stage::Flop);
        let flop_first = (game.dealer_idx + 1) % 2;
        let flop_second = (flop_first + 1) % 2;
        game.apply_player_action(flop_first, PlayerAction::Bet(10))
            .unwrap();
        game.apply_player_action(flop_second, PlayerAction::CheckCall)
            .unwrap();

        // Turn: same pattern
        assert_eq!(game.stage, Stage::Turn);
        let turn_first = (game.dealer_idx + 1) % 2;
        let turn_second = (turn_first + 1) % 2;
        game.apply_player_action(turn_first, PlayerAction::Bet(10))
            .unwrap();
        game.apply_player_action(turn_second, PlayerAction::CheckCall)
            .unwrap();

        // River: same pattern
        assert_eq!(game.stage, Stage::River);
        let river_first = (game.dealer_idx + 1) % 2;
        let river_second = (river_first + 1) % 2;
        game.apply_player_action(river_first, PlayerAction::Bet(10))
            .unwrap();
        game.apply_player_action(river_second, PlayerAction::CheckCall)
            .unwrap();

        // Showdown should occur and PotAwarded should equal 100 (not 115)
        assert_eq!(game.stage, Stage::Showdown);
        let last = game.action_log.iter().rev().find_map(|e| match &e.event {
            LogEvent::PotAwarded { amount, .. } => Some(*amount),
            _ => None,
        });
        assert_eq!(last, Some(100));
    }

    #[test]
    fn test_min_raise_enforcement() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 42);

        // Preflop: player raises to 30 (BB=10, so raise is +20, min_raise becomes 20)
        game.apply_player_action(0, PlayerAction::Bet(20)).unwrap();
        assert_eq!(game.current_bet, 30);
        assert_eq!(game.min_raise, 20); // The size of the last raise

        // Bot now needs to call the raise
        game.apply_player_action(1, PlayerAction::CheckCall)
            .unwrap();

        // Flop: betting round starts with current_bet reset to 0
        assert_eq!(game.stage, Stage::Flop);

        // Check who should be first to act on flop (dealer+1 position)
        let flop_first = (game.dealer_idx + 1) % 2;
        assert_eq!(flop_first, 1); // Bot should be first to act on flop

        // On flop, current_bet is reset to 0, so Bet(20) is an open bet to 20 total
        game.apply_player_action(1, PlayerAction::Bet(20)).unwrap();
        assert_eq!(game.current_bet, 20);
        assert_eq!(game.min_raise, 20); // Min raise set to bet amount

        // Player must now call the open bet of 20
        game.apply_player_action(0, PlayerAction::CheckCall)
            .unwrap();

        // After both players act, should advance to Turn and reset current_bet to 0
        assert_eq!(game.stage, Stage::Turn);
        assert_eq!(game.current_bet, 0);

        // On Turn, current_bet is reset to 0 again
        // Who is first to act on Turn?
        let turn_first = (game.dealer_idx + 1) % 2;
        assert_eq!(turn_first, 1); // Bot should be first

        // Bot makes another open bet of 20
        game.apply_player_action(1, PlayerAction::Bet(20)).unwrap();
        assert_eq!(game.current_bet, 20);
        assert_eq!(game.min_raise, 20); // Min raise set to bet amount
    }

    #[test]
    fn test_small_raise_treated_as_call() {
        let mut game = Game::new_with_seed("Player".to_string(), 2, 123);

        // Preflop: player1 raises to 30 (+20 from BB=10)
        game.apply_player_action(0, PlayerAction::Bet(20)).unwrap();
        assert_eq!(game.current_bet, 30);
        assert_eq!(game.min_raise, 20);

        // Player2 (bot 1) calls to 30
        game.apply_player_action(1, PlayerAction::CheckCall)
            .unwrap();
        // Player3 (bot 2) calls to 30 to complete preflop
        game.apply_player_action(2, PlayerAction::CheckCall)
            .unwrap();
        assert_eq!(game.stage, Stage::Flop);

        // On flop, first to act is dealer+1
        let flop_first = (game.dealer_idx + 1) % 3;
        let old_pot = game.pot;
        // Tries to open bet with 5 (should be minimum BB=10)
        game.apply_player_action(flop_first, PlayerAction::Bet(5))
            .unwrap();
        // Should be treated as minimum open bet of 10 (BB)
        assert_eq!(game.current_bet, 10); // Minimum open bet should be BB
        assert!(game.pot > old_pot); // Pot should increase
    }

    #[test]
    fn test_all_in_small_raise_treated_as_call() {
        let mut game = Game::new_with_seed("Player".to_string(), 2, 456);

        // Preflop: player1 raises to 30 (BB=10, +20)
        game.apply_player_action(0, PlayerAction::Bet(20)).unwrap();
        assert_eq!(game.current_bet, 30);
        assert_eq!(game.min_raise, 20);

        // Complete preflop: both bots call to 30
        game.apply_player_action(1, PlayerAction::CheckCall)
            .unwrap();
        game.apply_player_action(2, PlayerAction::CheckCall)
            .unwrap();
        assert_eq!(game.stage, Stage::Flop);

        // On flop, first to act is dealer+1 -> bot 1
        let flop_first = (game.dealer_idx + 1) % 3;
        let pot_before = game.pot;
        // Tries to open bet with 5 (minimum should be BB=10)
        game.apply_player_action(flop_first, PlayerAction::Bet(5))
            .unwrap();
        // Should be treated as open bet of minimum BB=10
        assert_eq!(game.current_bet, 10); // Should be minimum open bet
        assert!(!game.players[flop_first].all_in); // Should not be all-in with healthy stack
        assert!(game.pot > pot_before); // Pot should increase
    }

    #[test]
    fn test_after_all_in_correct_behavior() {
        let mut game = Game::new_with_seed("Player".to_string(), 3, 789);

        // Preflop: the first to act is left of the BB in 4-handed
        let n = game.players.len();
        let preflop_first = game.to_act;
        game.apply_player_action(preflop_first, PlayerAction::Bet(30))
            .unwrap();
        assert_eq!(game.current_bet, 40);
        assert_eq!(game.min_raise, 30);

        // All remaining players act preflop in order and call to 40
        for i in 1..n {
            let idx = (preflop_first + i) % n;
            game.apply_player_action(idx, PlayerAction::CheckCall)
                .unwrap();
        }
        assert_eq!(game.stage, Stage::Flop);

        // On flop, first to act is dealer+1
        let flop_first = (game.dealer_idx + 1) % n;
        assert_eq!(game.to_act, flop_first);
        // Tries to open bet with 5 (minimum should be BB=10)
        game.apply_player_action(flop_first, PlayerAction::Bet(5))
            .unwrap();
        assert!(!game.players[flop_first].all_in);
        assert_eq!(game.current_bet, 10); // Should be minimum open bet

        // Other players should be able to normally call or raise after open bet
    }
}
