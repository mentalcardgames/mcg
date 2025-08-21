//! Dealing and hand initialization helpers.

use anyhow::{Context, Result};
use rand::seq::SliceRandom;
use std::collections::VecDeque;

use mcg_shared::{ActionEvent, ActionKind, BlindKind, Card, GameAction};

use super::Game;

/// Public method on Game to start a new hand with a fresh shuffled deck.
impl Game {
    pub fn start_new_hand(&mut self) -> Result<()> {
        // Shuffle fresh deck
        let mut deck: Vec<Card> = (0..52).map(Card).collect();
        deck.shuffle(&mut rand::rng());
        start_new_hand_from_deck(self, deck).context("Failed to start new hand from shuffled deck")
    }
}

/// Initialize a new hand using the provided deck order.
/// This resets round state, deals hole cards, posts blinds and
/// establishes the first player to act according to heads-up vs 3+ rules.
pub(crate) fn start_new_hand_from_deck(g: &mut Game, deck: Vec<Card>) -> Result<()> {
    g.deck = VecDeque::from(deck);

    // Deal hole cards
    let mut dealt_events = Vec::with_capacity(g.players.len());
    for p in &mut g.players {
        p.has_folded = false;
        p.all_in = false;
        let c1 = g.deck.pop_front().ok_or_else(|| {
            anyhow::anyhow!(
                "Deck underflow while dealing hole card 1 to player {}",
                p.id
            )
        })?;
        let c2 = g.deck.pop_front().ok_or_else(|| {
            anyhow::anyhow!(
                "Deck underflow while dealing hole card 2 to player {}",
                p.id
            )
        })?;
        p.cards = [c1, c2];
        // collect typed events to avoid mutable-borrow conflicts while iterating players
        dealt_events.push(ActionEvent::game(GameAction::DealtHole {
            player_id: mcg_shared::PlayerId(p.id),
        }));
        tracing::info!(player = %p.name, card0 = %super::super::eval::card_str(p.cards[0]), card1 = %super::super::eval::card_str(p.cards[1]), "dealt hole cards");
    }

    // Reset table state
    g.community.clear();
    g.pot = 0;
    g.stage = mcg_shared::Stage::Preflop;
    g.current_bet = 0;
    g.min_raise = g.bb;
    g.round_bets = vec![0; g.players.len()];
    g.recent_actions.clear();
    g.winner_ids.clear();

    // Emit dealing events now that borrowing finished (derive legacy LogEntry on public serialization)
    g.recent_actions.extend(dealt_events);
    super::utils::cap_logs(g);

    // Post blinds
    let n = g.players.len();
    if n > 1 {
        // In heads-up, dealer posts SB and acts first preflop; otherwise SB=dealer+1, BB=dealer+2
        let (sb_idx, bb_idx) = if n == 2 {
            (g.dealer_idx, (g.dealer_idx + 1) % n)
        } else {
            ((g.dealer_idx + 1) % n, (g.dealer_idx + 2) % n)
        };
        post_blind(g, sb_idx, BlindKind::SmallBlind, g.sb);
        post_blind(g, bb_idx, BlindKind::BigBlind, g.bb);
        g.current_bet = g.bb;
        g.min_raise = g.bb;
        // Preflop first to act is left of BB
        g.to_act = (bb_idx + 1) % n;
    } else {
        g.to_act = g.dealer_idx;
    }

    g.init_round_for_stage();
    g.log(ActionEvent::game(GameAction::StageChanged(g.stage)));
    Ok(())
}

/// Post a small/big blind, capping to available stack and marking all-in when necessary.
fn post_blind(g: &mut Game, idx: usize, kind: BlindKind, amount: u32) {
    let a = amount.min(g.players[idx].stack);
    g.players[idx].stack -= a;
    g.round_bets[idx] += a;
    g.pot += a;
    if a < amount {
        g.players[idx].all_in = true;
    }
    g.log(ActionEvent::player(
        mcg_shared::PlayerId(idx),
        ActionKind::PostBlind { kind, amount: a },
    ));
    tracing::info!(player = %g.players[idx].name, kind = ?kind, amount = a, stack = g.players[idx].stack, "posted blind");
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn shuffled_deck_with_seed(seed: u64) -> Vec<Card> {
    // Simple LCG for deterministic shuffling in tests
    fn lcg(next: &mut u64) -> u32 {
        // Constants from Numerical Recipes
        *next = next.wrapping_mul(1664525).wrapping_add(1013904223);
        (*next >> 16) as u32
    }
    let mut deck: Vec<Card> = (0..52).map(Card).collect();
    let mut s = seed;
    // Fisher-Yates
    for i in (1..deck.len()).rev() {
        let r = lcg(&mut s) as usize % (i + 1);
        deck.swap(i, r);
    }
    deck
}
