use crate::game_data::GameData;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::{format_game_data, DebugLevel};

pub fn save_game_data(data: &GameData, path: &Path) -> io::Result<()> {
    let level = if path.exists() {
        if let Ok(first_line) = fs::read_to_string(path) {
            let first_line = first_line.lines().next().unwrap_or("");
            DebugLevel::from_marker(first_line).unwrap_or(DebugLevel::Medium)
        } else {
            DebugLevel::Medium
        }
    } else {
        DebugLevel::Medium
    };

    let formatted = format_game_data(data, level);
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;
    writeln!(file, "\n{}", formatted)?;
    Ok(())
}
