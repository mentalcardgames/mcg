/// An input submitted by a player.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    /// Name of the player who submitted this input (e.g. "P1").
    pub player_id: String,
    /// The kind of input selected.
    pub kind: InputKind,
}

/// The choice made by a player, without identity metadata.
#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
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
    /// A numeric answer (`InputType::Number`), e.g. from `bid any on <memory>
    /// of <owner>` (2026-08-10).
    Number {
        value: i32,
    },
}

impl Input {
    pub fn idx(&self) -> usize {
        self.kind.idx()
    }

    pub fn player_idx(&self) -> Option<usize> {
        self.kind.player_idx()
    }

    pub fn card_selection(&self) -> Option<&[usize]> {
        self.kind.card_selection()
    }
}

impl InputKind {
    pub fn idx(&self) -> usize {
        match self {
            InputKind::Choice { idx } => *idx,
            InputKind::OptionalAccept => 0,
            InputKind::OptionalDecline => 1,
            InputKind::ChoosePlayer { .. } => 0,
            InputKind::ChooseCards { .. } => 0,
            InputKind::Number { .. } => 0,
        }
    }

    /// If this is a `ChoosePlayer` input, the chosen 0-based candidate index.
    pub fn player_idx(&self) -> Option<usize> {
        match self {
            InputKind::ChoosePlayer { idx } => Some(*idx),
            _ => None,
        }
    }

    /// If this is a `ChooseCards` input, the chosen 0-based `display` indices.
    pub fn card_selection(&self) -> Option<&[usize]> {
        match self {
            InputKind::ChooseCards { selected } => Some(selected),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum StepResult {
    Ok,
    NeedsInput(InputType),
    GameOver,
    Error(crate::error::EngineError),
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
    /// Prompt the player to enter a number (`bid any on <memory> of
    /// <owner>`; 2026-08-10). `min`/`max` are advisory bounds from the
    /// quantity's `IntRange` when present (`None` = unbounded).
    Number {
        min: Option<i32>,
        max: Option<i32>,
        prompt: String,
    },
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
