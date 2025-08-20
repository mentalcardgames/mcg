//! Showdown resolution and pot awarding.

use super::Game;
use crate::eval::{evaluate_best_hand, pick_best_five};
use mcg_shared::{ActionEvent, GameAction, HandResult};

/// Resolve showdown by evaluating all non-folded hands, splitting the pot on ties
/// and logging the results. Pot is distributed chip-by-chip for any remainder to
/// the earliest winners in table order. (Side-pots are intentionally not modeled here.)
pub(crate) fn finish_showdown(g: &mut Game) {
    // Evaluate all non-folded players
    let mut results: Vec<HandResult> = Vec::new();
    for (i, p) in g.players.iter().enumerate() {
        if p.has_folded {
            continue;
        }
        let rank = evaluate_best_hand(p.cards, &g.community);
        let best_five = pick_best_five(p.cards, &g.community);
        results.push(HandResult {
            player_id: mcg_shared::PlayerId(i),
            rank,
            best_five,
        });
    }

    // Determine winners (top rank; split on ties)
    results.sort_by(|a, b| a.rank.cmp(&b.rank));
    let winners: Vec<mcg_shared::PlayerId> = if let Some(best) = results.last().cloned() {
        results
            .iter()
            .rev()
            .take_while(|r| r.rank == best.rank)
            .map(|r| r.player_id)
            .collect()
    } else {
        vec![]
    };
    g.winner_ids = winners.clone().into_iter().map(|p| p.into()).collect();

    g.log(ActionEvent::game(GameAction::Showdown {
        hand_results: results.clone(),
    }));

    if !winners.is_empty() && g.pot > 0 {
        let share = g.pot / winners.len() as u32;
        let mut remainder = g.pot % winners.len() as u32;
        for &w in &winners {
            let mut win = share;
            if remainder > 0 {
                win += 1;
                remainder -= 1;
            }
            let w_idx: usize = w.into();
            g.players[w_idx].stack += win;
        }
        g.log(ActionEvent::game(GameAction::PotAwarded {
            winners: winners.clone(),
            amount: g.pot,
        }));
        println!("[SHOWDOWN] Pot {} awarded to {:?}", g.pot, winners);
        g.pot = 0;
    }
}
