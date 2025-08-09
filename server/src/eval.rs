use mcg_shared::{HandRank, HandRankCategory};

/// Returns the rank index (0..=12) where 0 is Ace, 1 is 2, ..., 12 is King.
#[inline]
pub fn card_rank(c: u8) -> u8 {
    c % 13
}

/// Returns the suit index (0..=3) where 0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades.
#[inline]
pub fn card_suit(c: u8) -> u8 {
    c / 13
}

/// Returns a string like "A♣", "T♦", etc.
pub fn card_str(c: u8) -> String {
    let rank_idx = card_rank(c) as usize;
    let suit_idx = card_suit(c) as usize;
    let ranks = [
        "A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K",
    ];
    let suits = ['♣', '♦', '♥', '♠'];
    format!("{}{}", ranks[rank_idx], suits[suit_idx])
}

/// Evaluate the best 5-card hand from 2 hole + up to 5 community cards.
/// Returns a HandRank with category and tiebreakers for comparison.
pub fn evaluate_best_hand(hole: [u8; 2], community: &[u8]) -> HandRank {
    let mut cards = Vec::with_capacity(7);
    cards.push(hole[0]);
    cards.push(hole[1]);
    for &c in community {
        cards.push(c);
    }
    best_rank_from_seven(&cards)
}

/// Naive best-five-card selection for presentation (not used for ranking).
/// Currently: returns the top 5 community cards by rank value as an approximation.
pub fn pick_best_five(_hole: [u8; 2], community: &[u8], _rank: &HandRank) -> [u8; 5] {
    let mut cc = community.to_vec();
    cc.sort_unstable_by(|a, b| rank_value_high(card_rank(*b)).cmp(&rank_value_high(card_rank(*a))));
    let mut out = [0u8; 5];
    for i in 0..5.min(cc.len()) {
        out[i] = cc[i];
    }
    out
}

// ===== Internal helpers =====

fn best_rank_from_seven(cards: &[u8]) -> HandRank {
    // Group by suit
    let mut suit_cards: [Vec<u8>; 4] = [vec![], vec![], vec![], vec![]];
    for &c in cards {
        suit_cards[card_suit(c) as usize].push(c);
    }
    // Suit presence >=5 indicates possible flush
    let flush_suit = (0..4).find(|&s| suit_cards[s].len() >= 5).map(|s| s as u8);

    // Straight flush
    if let Some(fs) = flush_suit {
        let mut values = ranks_as_values_unique(&suit_cards[fs as usize]);
        if let Some(high) = straight_high(&values) {
            return HandRank {
                category: HandRankCategory::StraightFlush,
                tiebreakers: vec![high],
            };
        }
    }

    // Counts by value (2..14)
    let mut counts = [0u8; 15]; // index is value (2..14). 0..1 unused
    let mut all_values = Vec::with_capacity(cards.len());
    for &c in cards {
        let v = rank_value_high(card_rank(c));
        counts[v as usize] += 1;
        all_values.push(v);
    }

    // Four of a kind
    if let Some((quad, kicker)) = find_n_of_a_kind(&counts, 4, &all_values) {
        return HandRank {
            category: HandRankCategory::FourKind,
            tiebreakers: vec![quad, kicker],
        };
    }

    // Full house
    if let Some((trip, pair)) = find_full_house(&counts) {
        return HandRank {
            category: HandRankCategory::FullHouse,
            tiebreakers: vec![trip, pair],
        };
    }

    // Flush
    if let Some(fs) = flush_suit {
        let mut vs = suit_cards[fs as usize]
            .iter()
            .map(|&c| rank_value_high(card_rank(c)))
            .collect::<Vec<u8>>();
        vs.sort_unstable_by(|a, b| b.cmp(a));
        vs.truncate(5);
        return HandRank {
            category: HandRankCategory::Flush,
            tiebreakers: vs,
        };
    }

    // Straight
    let mut values = ranks_as_values_unique(cards);
    if let Some(high) = straight_high(&values) {
        return HandRank {
            category: HandRankCategory::Straight,
            tiebreakers: vec![high],
        };
    }

    // Trips
    if let Some((trip, kickers)) = find_n_kind_with_kickers(&counts, &all_values, 3, 2) {
        let mut t = vec![trip];
        t.extend(kickers);
        return HandRank {
            category: HandRankCategory::ThreeKind,
            tiebreakers: t,
        };
    }

    // Two pair
    if let Some((p_high, p_low, kicker)) = find_two_pair(&counts, &all_values) {
        return HandRank {
            category: HandRankCategory::TwoPair,
            tiebreakers: vec![p_high, p_low, kicker],
        };
    }

    // One pair
    if let Some((pair, kickers)) = find_n_kind_with_kickers(&counts, &all_values, 2, 3) {
        let mut t = vec![pair];
        t.extend(kickers);
        return HandRank {
            category: HandRankCategory::Pair,
            tiebreakers: t,
        };
    }

    // High card
    let mut highs = all_values;
    highs.sort_unstable_by(|a, b| b.cmp(a));
    highs.dedup();
    highs.truncate(5);
    HandRank {
        category: HandRankCategory::HighCard,
        tiebreakers: highs,
    }
}

#[inline]
fn rank_value_high(r: u8) -> u8 {
    // Map 0(A) -> 14; 1..12 -> 2..13
    if r == 0 {
        14
    } else {
        r + 1
    }
}

fn ranks_as_values_unique(cards: &[u8]) -> Vec<u8> {
    let mut v = cards
        .iter()
        .map(|&c| rank_value_high(card_rank(c)))
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
    let mut present = [false; 15];
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
                if best.map_or(true, |b| high > b) {
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

fn find_n_of_a_kind(counts: &[u8; 15], n: u8, all_values: &Vec<u8>) -> Option<(u8, u8)> {
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

fn find_full_house(counts: &[u8; 15]) -> Option<(u8, u8)> {
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
    counts: &[u8; 15],
    all_values: &Vec<u8>,
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

fn find_two_pair(counts: &[u8; 15], all_values: &Vec<u8>) -> Option<(u8, u8, u8)> {
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
    let kicker = *kickers.first().unwrap_or(&2);
    Some((p_high, p_low, kicker))
}
