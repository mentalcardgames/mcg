use super::{
    cards::{card_rank, CardRank, CardSuit},
    constants::NUM_RANKS,
};
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
        let mut out = [Card::new(CardRank::Ace, CardSuit::Clubs); 5];
        let n = all.len().min(5);
        out[..n].copy_from_slice(&all[..n]);
        return out;
    }

    // Enumerate all 5-card combinations and select the one with the highest rank
    let n = all.len();
    let mut best_rank: Option<HandRank> = None;
    let mut best_combo: [Card; 5] = [Card::new(CardRank::Ace, CardSuit::Clubs); 5];

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
fn rank_value_high(rank: CardRank) -> u8 {
    match rank {
        CardRank::Ace => 14,
        CardRank::Two => 2,
        CardRank::Three => 3,
        CardRank::Four => 4,
        CardRank::Five => 5,
        CardRank::Six => 6,
        CardRank::Seven => 7,
        CardRank::Eight => 8,
        CardRank::Nine => 9,
        CardRank::Ten => 10,
        CardRank::Jack => 11,
        CardRank::Queen => 12,
        CardRank::King => 13,
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
        let rank = card_rank(card).as_usize();
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

    // Calculate proper tiebreakers based on hand category
    let tiebreakers = calculate_tiebreakers(&category, &rank_counts, cards);

    HandRank {
        category,
        tiebreakers,
    }
}

/// Calculate tiebreakers for hand comparison
fn calculate_tiebreakers(
    category: &mcg_shared::HandRankCategory,
    rank_counts: &[u8; NUM_RANKS],
    cards: &[Card],
) -> Vec<u8> {
    let mut tiebreakers = vec![0u8; 5];

    match category {
        mcg_shared::HandRankCategory::HighCard => {
            // Use all 5 highest cards in descending order
            let mut ranks: Vec<u8> = cards
                .iter()
                .map(|&card| rank_value_high(card_rank(card)))
                .collect();
            ranks.sort_unstable_by(|a, b| b.cmp(a));
            let copy_len = 5.min(ranks.len());
            tiebreakers[..copy_len].copy_from_slice(&ranks[..copy_len]);
        }
        mcg_shared::HandRankCategory::Pair => {
            // Find the pair rank and calculate kickers from actual cards
            let pair_rank = find_pair_rank(rank_counts);

            // Get all card ranks and remove the paired cards to find kickers
            let all_ranks: Vec<u8> = cards
                .iter()
                .map(|&card| rank_value_high(card_rank(card)))
                .collect();

            // Remove instances of the pair rank (remove 2 instances for the pair)
            let mut pair_count = 0;
            let mut kicker_ranks: Vec<u8> = all_ranks
                .into_iter()
                .filter(|&rank| {
                    if rank == pair_rank && pair_count < 2 {
                        pair_count += 1;
                        false
                    } else {
                        true
                    }
                })
                .collect();

            // Sort kickers in descending order
            kicker_ranks.sort_unstable_by(|a, b| b.cmp(a));

            tiebreakers[0] = pair_rank;
            let copy_len = 3.min(kicker_ranks.len());
            tiebreakers[1..(copy_len + 1)].copy_from_slice(&kicker_ranks[..copy_len]);
        }
        mcg_shared::HandRankCategory::TwoPair => {
            // Find both pair ranks, then kicker from actual cards
            let pair_ranks = find_two_pair_ranks(rank_counts);

            // Get all card ranks and remove the paired cards to find kicker
            let all_ranks: Vec<u8> = cards
                .iter()
                .map(|&card| rank_value_high(card_rank(card)))
                .collect();

            // Remove instances of the pair ranks (remove 2 instances for each pair)
            let mut pair1_count = 0;
            let mut pair2_count = 0;
            let kicker_ranks: Vec<u8> = all_ranks
                .into_iter()
                .filter(|&rank| {
                    if rank == pair_ranks[0] && pair1_count < 2 {
                        pair1_count += 1;
                        false
                    } else if rank == pair_ranks[1] && pair2_count < 2 {
                        pair2_count += 1;
                        false
                    } else {
                        true
                    }
                })
                .collect();

            tiebreakers[0] = pair_ranks[0].max(pair_ranks[1]); // Higher pair
            tiebreakers[1] = pair_ranks[0].min(pair_ranks[1]); // Lower pair
            if let Some(&kicker) = kicker_ranks.first() {
                tiebreakers[2] = kicker;
            }
        }
        mcg_shared::HandRankCategory::ThreeKind => {
            // Find the three of a kind rank, then kickers from actual cards
            let trips_rank = find_trips_rank(rank_counts);

            // Get all card ranks and remove the trip cards to find kickers
            let all_ranks: Vec<u8> = cards
                .iter()
                .map(|&card| rank_value_high(card_rank(card)))
                .collect();

            // Remove instances of the trips rank (remove 3 instances)
            let mut trips_count = 0;
            let mut kicker_ranks: Vec<u8> = all_ranks
                .into_iter()
                .filter(|&rank| {
                    if rank == trips_rank && trips_count < 3 {
                        trips_count += 1;
                        false
                    } else {
                        true
                    }
                })
                .collect();

            // Sort kickers in descending order
            kicker_ranks.sort_unstable_by(|a, b| b.cmp(a));

            tiebreakers[0] = trips_rank;
            let copy_len = 2.min(kicker_ranks.len());
            tiebreakers[1..(copy_len + 1)].copy_from_slice(&kicker_ranks[..copy_len]);
        }
        mcg_shared::HandRankCategory::Straight => {
            // For straights, the high card determines the winner
            let straight_high = find_straight_high(rank_counts);
            tiebreakers[0] = straight_high;
        }
        mcg_shared::HandRankCategory::Flush => {
            // For flushes, use high cards like high card
            let mut ranks: Vec<u8> = cards
                .iter()
                .map(|&card| rank_value_high(card_rank(card)))
                .collect();
            ranks.sort_unstable_by(|a, b| b.cmp(a));
            let copy_len = 5.min(ranks.len());
            tiebreakers[..copy_len].copy_from_slice(&ranks[..copy_len]);
        }
        mcg_shared::HandRankCategory::FullHouse => {
            // Three of a kind rank, then pair rank
            let trips_rank = find_trips_rank(rank_counts);
            let pair_rank = find_pair_rank(rank_counts);

            tiebreakers[0] = trips_rank;
            tiebreakers[1] = pair_rank;
        }
        mcg_shared::HandRankCategory::FourKind => {
            // Four of a kind rank, then kicker from actual cards
            let quads_rank = find_quads_rank(rank_counts);

            // Get all card ranks and remove the quad cards to find kicker
            let all_ranks: Vec<u8> = cards
                .iter()
                .map(|&card| rank_value_high(card_rank(card)))
                .collect();

            // Remove instances of the quads rank (remove 4 instances)
            let mut quads_count = 0;
            let kicker_ranks: Vec<u8> = all_ranks
                .into_iter()
                .filter(|&rank| {
                    if rank == quads_rank && quads_count < 4 {
                        quads_count += 1;
                        false
                    } else {
                        true
                    }
                })
                .collect();

            tiebreakers[0] = quads_rank;
            if let Some(&kicker) = kicker_ranks.first() {
                tiebreakers[1] = kicker;
            }
        }
        mcg_shared::HandRankCategory::StraightFlush => {
            // For straight flushes, the high card determines the winner
            let straight_high = find_straight_high(rank_counts);
            tiebreakers[0] = straight_high;
        }
    }

    tiebreakers
}

/// Find the rank of a single pair
fn find_pair_rank(rank_counts: &[u8; NUM_RANKS]) -> u8 {
    for (i, &count) in rank_counts.iter().enumerate() {
        if count == 2 {
            return rank_value_high(CardRank::from_u8(i as u8));
        }
    }
    0
}

/// Find the ranks of two pairs
fn find_two_pair_ranks(rank_counts: &[u8; NUM_RANKS]) -> Vec<u8> {
    let mut pairs = Vec::new();
    for (i, &count) in rank_counts.iter().enumerate() {
        if count == 2 {
            pairs.push(rank_value_high(CardRank::from_u8(i as u8)));
        }
    }
    pairs
}

/// Find the rank of three of a kind
fn find_trips_rank(rank_counts: &[u8; NUM_RANKS]) -> u8 {
    for (i, &count) in rank_counts.iter().enumerate() {
        if count == 3 {
            return rank_value_high(CardRank::from_u8(i as u8));
        }
    }
    0
}

/// Find the rank of four of a kind
fn find_quads_rank(rank_counts: &[u8; NUM_RANKS]) -> u8 {
    for (i, &count) in rank_counts.iter().enumerate() {
        if count == 4 {
            return rank_value_high(CardRank::from_u8(i as u8));
        }
    }
    0
}

/// Find the high card of a straight
fn find_straight_high(rank_counts: &[u8; NUM_RANKS]) -> u8 {
    // Check for ace-high straight (10-J-Q-K-A)
    if rank_counts[9] > 0   // Ten
        && rank_counts[10] > 0  // Jack
        && rank_counts[11] > 0  // Queen
        && rank_counts[12] > 0  // King
        && rank_counts[0] > 0
    // Ace
    {
        return 14; // Ace-high straight, Ace is high card
    }

    // Check for regular straights (not including ace-high)
    for i in 0..(NUM_RANKS - 4) {
        if rank_counts[i] > 0
            && rank_counts[i + 1] > 0
            && rank_counts[i + 2] > 0
            && rank_counts[i + 3] > 0
            && rank_counts[i + 4] > 0
        {
            return rank_value_high(CardRank::from_u8((i + 4) as u8));
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
        return 5; // Ace-low straight, high card is 5
    }

    0
}

/// Check if there's a straight in the rank counts
fn check_straight(rank_counts: &[u8; NUM_RANKS]) -> bool {
    // Check for ace-high straight (10-J-Q-K-A)
    if rank_counts[9] > 0   // Ten
        && rank_counts[10] > 0  // Jack
        && rank_counts[11] > 0  // Queen
        && rank_counts[12] > 0  // King
        && rank_counts[0] > 0
    // Ace
    {
        return true;
    }

    // Check for regular straights (not including ace-high)
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
