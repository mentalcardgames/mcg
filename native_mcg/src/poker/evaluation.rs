use super::{
    cards::{CardRank, CardSuit},
    constants::*,
};
use mcg_shared::{Card, HandRank, HandRankCategory};

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
            rank_value_high(a.rank()).cmp(&rank_value_high(b.rank()))
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
                            None => {
                                best_rank = Some(rank);
                                best_combo = subset;
                            }
                            Some(r) => {
                                if rank > *r {
                                    best_rank = Some(rank);
                                    best_combo = subset;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    best_combo
}

// ===== Internal helpers =====

fn best_rank_from_seven(cards: &[Card]) -> HandRank {
    let flush_suit = analyze_suits_for_flush(cards);

    // Check for straight flush
    if let Some(sflush) = check_straight_flush(cards, flush_suit) {
        return sflush;
    }

    let (counts, all_values) = analyze_card_values(cards);

    // Check hands in descending rank order
    if let Some(four_kind) = check_four_of_a_kind(&counts, &all_values) {
        return four_kind;
    }

    if let Some(full_house) = check_full_house(&counts) {
        return full_house;
    }

    if let Some(flush) = check_flush(cards, flush_suit) {
        return flush;
    }

    if let Some(straight) = check_straight(cards) {
        return straight;
    }

    if let Some(three_kind) = check_three_of_a_kind(&counts, &all_values) {
        return three_kind;
    }

    if let Some(two_pair) = check_two_pair(&counts, &all_values) {
        return two_pair;
    }

    if let Some(pair) = check_one_pair(&counts, &all_values) {
        return pair;
    }

    check_high_card(&all_values)
}

fn analyze_suits_for_flush(cards: &[Card]) -> Option<u8> {
    // Group by suit
    let mut suit_cards: [Vec<Card>; NUM_SUITS] = [vec![], vec![], vec![], vec![]];
    for &c in cards {
        suit_cards[c.suit().as_usize()].push(c);
    }
    // Suit presence >=5 indicates possible flush
    (0..NUM_SUITS)
        .find(|&s| suit_cards[s].len() >= 5)
        .map(|s| s as u8)
}

fn analyze_card_values(cards: &[Card]) -> ([u8; RANK_COUNT_ARRAY_SIZE], Vec<u8>) {
    let mut counts = [0u8; RANK_COUNT_ARRAY_SIZE];
    let mut all_values = Vec::with_capacity(cards.len());
    for &c in cards {
        let v = rank_value_high(c.rank());
        counts[v as usize] += 1;
        all_values.push(v);
    }
    (counts, all_values)
}

fn check_straight_flush(cards: &[Card], flush_suit: Option<u8>) -> Option<HandRank> {
    if let Some(fs) = flush_suit {
        let mut suit_cards: [Vec<Card>; NUM_SUITS] = [vec![], vec![], vec![], vec![]];
        for &c in cards {
            suit_cards[c.suit().as_usize()].push(c);
        }

        let values = ranks_as_values_unique(&suit_cards[fs as usize]);
        if let Some(high) = straight_high(&values) {
            return Some(HandRank {
                category: HandRankCategory::StraightFlush,
                tiebreakers: vec![high],
            });
        }
    }
    None
}

fn check_four_of_a_kind(
    counts: &[u8; RANK_COUNT_ARRAY_SIZE],
    all_values: &[u8],
) -> Option<HandRank> {
    find_n_of_a_kind(counts, 4, all_values).map(|(quad, kicker)| HandRank {
        category: HandRankCategory::FourKind,
        tiebreakers: vec![quad, kicker],
    })
}

fn check_full_house(counts: &[u8; RANK_COUNT_ARRAY_SIZE]) -> Option<HandRank> {
    find_full_house(counts).map(|(trip, pair)| HandRank {
        category: HandRankCategory::FullHouse,
        tiebreakers: vec![trip, pair],
    })
}

fn check_flush(cards: &[Card], flush_suit: Option<u8>) -> Option<HandRank> {
    if let Some(fs) = flush_suit {
        let mut suit_cards: [Vec<Card>; NUM_SUITS] = [vec![], vec![], vec![], vec![]];
        for &c in cards {
            suit_cards[c.suit().as_usize()].push(c);
        }

        let mut vs = suit_cards[fs as usize]
            .iter()
            .map(|&c| rank_value_high(c.rank()))
            .collect::<Vec<u8>>();
        vs.sort_unstable_by(|a, b| b.cmp(a));
        vs.truncate(5);
        return Some(HandRank {
            category: HandRankCategory::Flush,
            tiebreakers: vs,
        });
    }
    None
}

fn check_straight(cards: &[Card]) -> Option<HandRank> {
    let values = ranks_as_values_unique(cards);
    straight_high(&values).map(|high| HandRank {
        category: HandRankCategory::Straight,
        tiebreakers: vec![high],
    })
}

fn check_three_of_a_kind(
    counts: &[u8; RANK_COUNT_ARRAY_SIZE],
    all_values: &[u8],
) -> Option<HandRank> {
    find_n_kind_with_kickers(counts, all_values, 3, 2).map(|(trip, kickers)| {
        let mut t = vec![trip];
        t.extend(kickers);
        HandRank {
            category: HandRankCategory::ThreeKind,
            tiebreakers: t,
        }
    })
}

fn check_two_pair(counts: &[u8; RANK_COUNT_ARRAY_SIZE], all_values: &[u8]) -> Option<HandRank> {
    find_two_pair(counts, all_values).map(|(p_high, p_low, kicker)| HandRank {
        category: HandRankCategory::TwoPair,
        tiebreakers: vec![p_high, p_low, kicker],
    })
}

fn check_one_pair(counts: &[u8; RANK_COUNT_ARRAY_SIZE], all_values: &[u8]) -> Option<HandRank> {
    find_n_kind_with_kickers(counts, all_values, 2, 3).map(|(pair, kickers)| {
        let mut t = vec![pair];
        t.extend(kickers);
        HandRank {
            category: HandRankCategory::Pair,
            tiebreakers: t,
        }
    })
}

fn check_high_card(all_values: &[u8]) -> HandRank {
    let mut highs = all_values.to_vec();
    highs.sort_unstable_by(|a, b| b.cmp(a));
    highs.dedup();
    highs.truncate(5);
    HandRank {
        category: HandRankCategory::HighCard,
        tiebreakers: highs,
    }
}

#[inline]
fn rank_value_high(rank: CardRank) -> u8 {
    // Map CardRank to high value (Ace=14, King=13, etc.)
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

fn ranks_as_values_unique(cards: &[Card]) -> Vec<u8> {
    let mut v = cards
        .iter()
        .map(|&c| rank_value_high(c.rank()))
        .collect::<Vec<u8>>();
    v.sort_unstable();
    v.dedup();
    v
}

fn straight_high(values_unique_sorted_asc: &Vec<u8>) -> Option<u8> {
    if values_unique_sorted_asc.is_empty() {
        return None;
    }
    // Build presence map for 2..14, also enable wheel (A as 1) if Ace present.
    let mut present = [false; RANK_COUNT_ARRAY_SIZE];
    for &v in values_unique_sorted_asc {
        if (2..=14).contains(&v) {
            present[v as usize] = true;
        }
    }
    // wheel A-2-3-4-5: treat Ace as 1
    if present[14] {
        present[1] = true;
    }

    // scan runs from high to low to get highest straight
    // We'll scan descending for convenience
    let mut best: Option<u8> = None;
    let mut run_len = 0usize;
    let mut last_v = 0usize;

    for v in (1..=14).rev() {
        if present[v] {
            if last_v == 0 || v + 1 == last_v {
                run_len += 1;
            } else {
                run_len = 1;
            }
            if run_len >= 5 {
                // v..v+4 is a straight; high card is last_v (or v+4), but since we're descending,
                // when we hit run_len==5, the high is v+4; continue to keep the highest found.
                let high = (v + 4) as u8;
                if best.is_none_or(|b| high > b) {
                    best = Some(high);
                }
            }
            last_v = v;
        } else {
            run_len = 0;
            last_v = 0;
        }
    }

    // normalize high=5 for wheel if detected (A-2-3-4-5)
    if best == Some(5) {
        return Some(5);
    }
    best
}

fn find_n_of_a_kind(
    counts: &[u8; RANK_COUNT_ARRAY_SIZE],
    n: u8,
    all_values: &[u8],
) -> Option<(u8, u8)> {
    // (rank, top kicker) with rank in 2..14
    let mut rank = None;
    for v in (2..=14).rev() {
        if counts[v] == n {
            rank = Some(v as u8);
            break;
        }
    }
    if let Some(rk) = rank {
        let mut kickers = all_values
            .iter()
            .cloned()
            .filter(|&v| v != rk)
            .collect::<Vec<u8>>();
        kickers.sort_unstable_by(|a, b| b.cmp(a));
        if let Some(&k) = kickers.first() {
            return Some((rk, k));
        }
    }
    None
}

fn find_full_house(counts: &[u8; RANK_COUNT_ARRAY_SIZE]) -> Option<(u8, u8)> {
    let mut trips = vec![];
    let mut pairs = vec![];
    for v in (2..=14).rev() {
        if counts[v] >= 3 {
            trips.push(v as u8);
        } else if counts[v] >= 2 {
            pairs.push(v as u8);
        }
    }
    if trips.is_empty() {
        return None;
    }
    let trip = trips[0];
    // Use second trip as pair if no pair exists
    let pair = pairs.first().cloned().or_else(|| trips.get(1).cloned());
    pair.map(|p| (trip, p))
}

fn find_n_kind_with_kickers(
    counts: &[u8; RANK_COUNT_ARRAY_SIZE],
    all_values: &[u8],
    n: u8,
    kicker_count: usize,
) -> Option<(u8, Vec<u8>)> {
    let mut kind_rank = None;
    for v in (2..=14).rev() {
        if counts[v] == n {
            kind_rank = Some(v as u8);
            break;
        }
    }
    if let Some(kr) = kind_rank {
        let mut kickers = all_values
            .iter()
            .cloned()
            .filter(|&v| v != kr)
            .collect::<Vec<u8>>();
        kickers.sort_unstable_by(|a, b| b.cmp(a));
        kickers.dedup();
        kickers.truncate(kicker_count);
        return Some((kr, kickers));
    }
    None
}

fn find_two_pair(counts: &[u8; RANK_COUNT_ARRAY_SIZE], all_values: &[u8]) -> Option<(u8, u8, u8)> {
    let mut pairs = vec![];
    for v in (2..=14).rev() {
        if counts[v] >= 2 {
            pairs.push(v as u8);
        }
    }
    if pairs.len() < 2 {
        return None;
    }
    let p_high = pairs[0];
    let p_low = pairs[1];

    let mut kickers = all_values
        .iter()
        .cloned()
        .filter(|&v| v != p_high && v != p_low)
        .collect::<Vec<u8>>();
    kickers.sort_unstable_by(|a, b| b.cmp(a));
    kickers.dedup();
    let kicker = kickers.first().copied().unwrap_or(2);
    Some((p_high, p_low, kicker))
}
