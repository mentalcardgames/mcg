#![allow(unused_imports)]
//! Misc helper methods for Game (active players, round state transitions, logging cap).

use anyhow::{Result, Context};
use mcg_shared::{LogEntry, LogEvent, Stage};
use crate::eval::card_str;
use crate::game::engine::{MAX_RECENT_ACTIONS, MAX_LOG_ENTRIES};
use super::Game;

/// Cap in-memory logs to bounded sizes to avoid unbounded growth when
/// the server runs for long sessions. This protects both serialization
/// payload size and UI performance when rendering histories.
pub(crate) fn cap_logs(g: &mut Game) {
    if g.recent_actions.len() > MAX_RECENT_ACTIONS {
        let start = g.recent_actions.len() - MAX_RECENT_ACTIONS;
        g.recent_actions.drain(0..start);
    }
    if g.action_log.len() > MAX_LOG_ENTRIES {
        let start = g.action_log.len() - MAX_LOG_ENTRIES;
        g.action_log.drain(0..start);
    }
}

impl Game {
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
                let c = self
                    .deck
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("Deck underflow while dealing turn card"))?;
                self.community.push(c);
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
                let c = self
                    .deck
                    .pop_front()
                    .ok_or_else(|| anyhow::anyhow!("Deck underflow while dealing river card"))?;
                self.community.push(c);
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
        Ok(())
    }
}
