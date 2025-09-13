use mcg_shared::{HandRank, HandRankCategory};

/// Hand ranking utilities and comparisons
pub struct HandRanker;

impl HandRanker {
    /// Compare two hand ranks and return the winner
    pub fn compare_hands(rank1: &HandRank, rank2: &HandRank) -> std::cmp::Ordering {
        rank1.cmp(rank2)
    }

    /// Get a descriptive string for a hand rank category
    pub fn category_to_string(category: &HandRankCategory) -> &'static str {
        match category {
            HandRankCategory::HighCard => "High Card",
            HandRankCategory::Pair => "Pair",
            HandRankCategory::TwoPair => "Two Pair",
            HandRankCategory::ThreeKind => "Three of a Kind",
            HandRankCategory::Straight => "Straight",
            HandRankCategory::Flush => "Flush",
            HandRankCategory::FullHouse => "Full House",
            HandRankCategory::FourKind => "Four of a Kind",
            HandRankCategory::StraightFlush => "Straight Flush",
        }
    }

    /// Check if a hand beats another hand
    pub fn hand_beats(rank1: &HandRank, rank2: &HandRank) -> bool {
        Self::compare_hands(rank1, rank2) == std::cmp::Ordering::Greater
    }
}
