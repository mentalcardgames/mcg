use mcg_shared::HandRank;

/// Hand ranking utilities and comparisons
pub struct HandRanker;

impl HandRanker {
    /// Compare two hand ranks and return the winner
    pub fn compare_hands(rank1: &HandRank, rank2: &HandRank) -> std::cmp::Ordering {
        rank1.cmp(rank2)
    }

    /// Check if a hand beats another hand
    pub fn hand_beats(rank1: &HandRank, rank2: &HandRank) -> bool {
        Self::compare_hands(rank1, rank2) == std::cmp::Ordering::Greater
    }
}
