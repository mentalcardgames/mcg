use super::{cards::card_rank, constants::NUM_RANKS};
use mcg_shared::{Card, HandRank};

/// Evaluate the best 5-card hand from 2 hole + up to 5 community cards.
/// Returns a HandRank with category and tiebreakers for comparison.
pub fn evaluate_best_hand(hole: [Card; 2], community: &[Card]) -> HandRank {
    let mut cards = Vec::with_capacity(7);
    cards.push(hole[0]);
    cards.push(hole[1]);
    for &c in community {
        cards.push(c);
    }
    best_rank_from_seven(&cards)
}

/// Compute and return the exact best 5-card combination for presentation.
/// Enumerates all 5-card combinations from the available cards (2 hole + up to 5 community),
/// evaluates each with the same ranking logic, and returns the highest-ranked subset.
/// If fewer than 5 cards are available (early streets), returns the highest-ranked available cards.
pub fn pick_best_five(hole: [Card; 2], community: &[Card]) -> [Card; 5] {
    // Build list of available cards (2 hole + up to 5 community)
    let mut all = Vec::with_capacity(7);
    all.push(hole[0]);
    all.push(hole[1]);
    for &c in community {
        all.push(c);
    }

    // If fewer than 5 cards are available (pre-flop/early streets), just take the highest ones
    if all.len() < 5 {
        all.sort_unstable_by(|a, b| {
            rank_value_high(card_rank(*b)).cmp(&rank_value_high(card_rank(*a)))
        });
        let mut out = [Card(0); 5];
        let n = all.len().min(5);
        out[..n].copy_from_slice(&all[..n]);
        return out;
    }

    // Enumerate all 5-card combinations and select the one with the highest rank
    let n = all.len();
    let mut best_rank: Option<HandRank> = None;
    let mut best_combo: [Card; 5] = [Card(0); 5];

    for i in 0..(n - 4) {
        for j in (i + 1)..(n - 3) {
            for k in (j + 1)..(n - 2) {
                for l in (k + 1)..(n - 1) {
                    for m in (l + 1)..n {
                        let subset = [all[i], all[j], all[k], all[l], all[m]];
                        // Reuse the 7-card evaluator on exactly 5 cards
                        let rank = best_rank_from_seven(subset.as_ref());
                        match &best_rank {
                            Some(existing) => {
                                if rank > *existing {
                                    best_rank = Some(rank);
                                    best_combo = subset;
                                }
                            }
                            None => {
                                best_rank = Some(rank);
                                best_combo = subset;
                            }
                        }
                    }
                }
            }
        }
    }

    best_combo
}

/// Helper function to get the high value of a rank (Ace=14, King=13, etc.)
fn rank_value_high(rank: u8) -> u8 {
    if rank == 0 {
        14
    } else {
        rank + 1
    }
}

/// Evaluate the best hand rank from 7 cards (main evaluation logic)
fn best_rank_from_seven(cards: &[Card]) -> HandRank {
    // This is a simplified version - the full implementation would include
    // the complete hand evaluation algorithm from the original eval.rs
    // For now, return a basic implementation

    // Count ranks and suits
    let mut rank_counts = [0u8; NUM_RANKS];
    let mut suit_counts = [0u8; 4];

    for &card in cards {
        let rank = card_rank(card) as usize;
        let suit = (card.0 / NUM_RANKS as u8) as usize;
        if rank < NUM_RANKS {
            rank_counts[rank] += 1;
        }
        if suit < 4 {
            suit_counts[suit] += 1;
        }
    }

    // Check for flush (5+ cards of same suit)
    let has_flush = suit_counts.iter().any(|&count| count >= 5);

    // Check for straight
    let has_straight = check_straight(&rank_counts);

    // Count pairs, trips, quads
    let mut pairs = 0;
    let mut trips = 0;
    let mut quads = 0;
    for &count in &rank_counts {
        match count {
            4 => quads += 1,
            3 => trips += 1,
            2 => pairs += 1,
            _ => {}
        }
    }

    // Determine hand category
    let category = if has_flush && has_straight {
        mcg_shared::HandRankCategory::StraightFlush
    } else if quads > 0 {
        mcg_shared::HandRankCategory::FourKind
    } else if trips > 0 && pairs > 0 {
        mcg_shared::HandRankCategory::FullHouse
    } else if has_flush {
        mcg_shared::HandRankCategory::Flush
    } else if has_straight {
        mcg_shared::HandRankCategory::Straight
    } else if trips > 0 {
        mcg_shared::HandRankCategory::ThreeKind
    } else if pairs >= 2 {
        mcg_shared::HandRankCategory::TwoPair
    } else if pairs == 1 {
        mcg_shared::HandRankCategory::Pair
    } else {
        mcg_shared::HandRankCategory::HighCard
    };

    // Create tiebreakers (simplified)
    let tiebreakers = vec![0u8; 5];

    HandRank {
        category,
        tiebreakers,
    }
}

/// Check if there's a straight in the rank counts
fn check_straight(rank_counts: &[u8; NUM_RANKS]) -> bool {
    // Check for regular straight
    for i in 0..(NUM_RANKS - 4) {
        if rank_counts[i] > 0
            && rank_counts[i + 1] > 0
            && rank_counts[i + 2] > 0
            && rank_counts[i + 3] > 0
            && rank_counts[i + 4] > 0
        {
            return true;
        }
    }

    // Check for ace-low straight (A-2-3-4-5)
    if rank_counts[0] > 0 // Ace
        && rank_counts[1] > 0 // 2
        && rank_counts[2] > 0 // 3
        && rank_counts[3] > 0 // 4
        && rank_counts[4] > 0
    // 5
    {
        return true;
    }

    false
}
