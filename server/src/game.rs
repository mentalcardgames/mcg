//! Core poker game logic.

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

    pub fn apply_player_action(&mut self, actor: usize, action: PlayerAction) {
        if actor != self.to_act || self.players[actor].has_folded || self.players[actor].all_in {
            return;
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
                    let bet_to = x
                        .max(self.bb)
                        .min(self.players[actor].stack + self.round_bets[actor]);
                    let add = bet_to
                        .saturating_sub(self.round_bets[actor])
                        .min(self.players[actor].stack);
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
                    let need = self.current_bet.saturating_sub(self.round_bets[actor]);
                    let target_to = self.current_bet + x;
                    let required = target_to.saturating_sub(self.round_bets[actor]);
                    let add = required.min(self.players[actor].stack);

                    if add <= need {
                        // Not enough to raise; treat as call (possibly all-in)
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
                    } else {
                        // Valid raise
                        self.players[actor].stack -= add;
                        self.round_bets[actor] += add;
                        self.pot += add;
                        let new_to = self.round_bets[actor];
                        let by = new_to.saturating_sub(prev_current_bet);
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
            return;
        }

        // If betting round complete, advance stage
        if self.is_betting_round_complete() {
            self.advance_stage();
            if self.stage == Stage::Showdown {
                self.finish_showdown();
                return;
            }
            self.init_round_for_stage();
        } else if let Some(&nxt) = self.pending_to_act.first() {
            self.to_act = nxt;
        }
    }

    pub fn start_new_hand(&mut self) {
        // Shuffle fresh deck
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rand::rng());
        self.start_new_hand_from_deck(deck);
    }

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
        self.action_log.extend(dealt_logs.into_iter());
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

    fn is_betting_round_complete(&self) -> bool {
        // Complete if every non-folded, non-all-in player has matched current_bet
        for (i, p) in self.players.iter().enumerate() {
            if p.has_folded || p.all_in {
                continue;
            }
            if self.round_bets[i] != self.current_bet {
                return false;
            }
        }
        true
    }

    fn init_round_for_stage(&mut self) {
        self.round_bets.fill(0);
        // Don't reset current_bet and min_raise for preflop since blinds have been posted
        if self.stage != Stage::Preflop {
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

    fn random_bot_action(&self, bot_index: usize) -> PlayerAction {
        use rand::Rng;
        let mut rng = rand::rng();
        let need = self.current_bet.saturating_sub(self.round_bets[bot_index]);
        if need == 0 {
            let r: u8 = rng.random_range(0..100);
            if r < 70 {
                PlayerAction::CheckCall
            } else {
                PlayerAction::Bet(self.bb)
            }
        } else {
            let r: u8 = rng.random_range(0..100);
            if r < 60 {
                PlayerAction::CheckCall
            } else if r < 80 {
                PlayerAction::Fold
            } else {
                PlayerAction::Bet(self.bb)
            }
        }
    }

    pub fn play_out_bots(&mut self) {
        // Let bots act until it's human's turn or showdown
        while self.stage != Stage::Showdown && self.to_act != 0 {
            let actor = self.to_act;
            if actor == 0
                || actor >= self.players.len()
                || self.players[actor].has_folded
                || self.players[actor].all_in
            {
                break;
            }
            let action = self.random_bot_action(actor);
            println!("[BOT] {}: {:?}", self.players[actor].name, action);
            self.apply_player_action(actor, action);
        }
    }
}

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
        game.apply_player_action(0, PlayerAction::CheckCall);

        // Next to act should not be the player again immediately
        assert_ne!(game.to_act, 0);
    }

    #[test]
    fn test_valid_raises() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 123);
        // Heads-up preflop: player (SB/dealer) acts first, raises to 30 (BB=10, raise=20)
        game.apply_player_action(0, PlayerAction::Bet(20));
        assert_eq!(game.current_bet, 30);

        // Bot needs to call the raise (contribute 20 more to match 30 total)
        game.apply_player_action(1, PlayerAction::CheckCall);

        // Betting round should be complete, should advance to Flop
        assert_eq!(game.stage, Stage::Flop);
    }

    #[test]
    fn test_betting_round_completion() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 999);
        // Heads-up preflop: player (SB/dealer) acts first, calls to match BB
        game.apply_player_action(0, PlayerAction::CheckCall);
        // Bot (BB) checks/calls to match 30 total (here to match 10 total)
        game.apply_player_action(1, PlayerAction::CheckCall);
        // Betting round should be complete
        assert_eq!(game.stage, Stage::Flop);
    }

    #[test]
    fn test_player_folding() {
        let mut game = Game::new_with_seed("Player".to_string(), 1, 7);
        // Player folds immediately preflop
        game.apply_player_action(0, PlayerAction::Fold);
        assert_eq!(game.stage, Stage::Showdown);
    }
}
