#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    Choice {
        idx: usize,
    },
    OptionalAccept,
    OptionalDecline,
    /// Chosen player index (0-based into `InputType::ChoosePlayer::candidates`).
    ChoosePlayer {
        idx: usize,
    },
    /// Chosen card indices (0-based into `InputType::ChooseCards::display`).
    ChooseCards {
        selected: Vec<usize>,
    },
}

impl Input {
    pub fn idx(&self) -> usize {
        match self {
            Input::Choice { idx } => *idx,
            Input::OptionalAccept => 0,
            Input::OptionalDecline => 1,
            // The new variants are not consumed by the Choice/Optional arms;
            // `idx()` is meaningless for them — callers use the dedicated
            // accessors below. Returning 0 keeps the match exhaustive.
            Input::ChoosePlayer { .. } => 0,
            Input::ChooseCards { .. } => 0,
        }
    }

    /// If this is a `ChoosePlayer` input, the chosen 0-based candidate index.
    pub fn player_idx(&self) -> Option<usize> {
        match self {
            Input::ChoosePlayer { idx } => Some(*idx),
            _ => None,
        }
    }

    /// If this is a `ChooseCards` input, the chosen 0-based `display` indices.
    pub fn card_selection(&self) -> Option<&[usize]> {
        match self {
            Input::ChooseCards { selected } => Some(selected),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum StepResult {
    Ok,
    NeedsInput(InputType),
    GameOver,
    Error(String),
}

#[derive(Clone, Debug)]
pub enum InputType {
    Choice {
        options: Vec<String>,
        max_index: usize,
    },
    Optional(String),
    /// Prompt the player to pick one player by index from `candidates`.
    ChoosePlayer {
        candidates: Vec<String>,
        prompt: String,
    },
    /// Prompt the player to pick a subset of `display` cards. `min`/`max`
    /// bound the number of selections (for `Quantifier::Any` this is
    /// `1..display.len()`; for an `IntRange` it reflects the range).
    ChooseCards {
        display: Vec<crate::game_data::Card>,
        min: usize,
        max: usize,
        prompt: String,
    },
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
