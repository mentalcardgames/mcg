use crate::game_data::GameData;

mod high;
mod low;
mod medium;
mod save;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DebugLevel {
    Low,
    Medium,
    High,
}

impl DebugLevel {
    pub fn from_marker(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "<!--LOW-->" => Some(DebugLevel::Low),
            "<!--MEDIUM-->" => Some(DebugLevel::Medium),
            "<!--HIGH-->" => Some(DebugLevel::High),
            _ => None,
        }
    }

    pub fn marker(&self) -> &'static str {
        match self {
            DebugLevel::Low => "<!--LOW-->",
            DebugLevel::Medium => "<!--MEDIUM-->",
            DebugLevel::High => "<!--HIGH-->",
        }
    }
}

pub fn format_game_data(data: &GameData, level: DebugLevel) -> String {
    match level {
        DebugLevel::Low => low::format_game_data_low(data),
        DebugLevel::Medium => medium::format_game_data_medium(data),
        DebugLevel::High => high::format_game_data_high(data),
    }
}

pub use save::save_game_data;
