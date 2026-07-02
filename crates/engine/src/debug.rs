use crate::game_data::GameData;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

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
        DebugLevel::Low => format_game_data_low(data),
        DebugLevel::Medium => format_game_data_medium(data),
        DebugLevel::High => format_game_data_high(data),
    }
}

fn format_game_data_low(data: &GameData) -> String {
    let mut output = String::new();
    output.push_str("=== GAME DATA (LOW) ===\n\n");

    let player_names: Vec<_> = data.players.iter().map(|p| p.name.clone()).collect();
    output.push_str(&format!("Players: {}\n", player_names.join(", ")));

    if let Some(current_idx) = data.current_player {
        if let Some(player_idx) = data.turn_order.get(current_idx) {
            if let Some(player) = data.players.get(*player_idx) {
                output.push_str(&format!("Current Player: {}\n", player.name));
            }
        }
    }

    if let Some(stage) = data.stage_stack.last() {
        output.push_str(&format!("Current Stage: {}\n", stage));
    }

    output.push_str(&format!("Turn Order Indices: {:?}\n", data.turn_order));

    output.push_str("\nCard Counts per Location:\n");
    for location in &data.locations {
        output.push_str(&format!(
            "  {}: {} cards\n",
            location.name,
            location.cards.len()
        ));
    }

    output
}

fn format_game_data_medium(data: &GameData) -> String {
    let mut output = String::new();
    output.push_str("=== GAME DATA (MEDIUM) ===\n\n");

    let player_names: Vec<_> = data.players.iter().map(|p| p.name.clone()).collect();
    output.push_str(&format!("Players: {}\n", player_names.join(", ")));

    if let Some(current_idx) = data.current_player {
        if let Some(player_idx) = data.turn_order.get(current_idx) {
            if let Some(player) = data.players.get(*player_idx) {
                output.push_str(&format!("Current Player: {}\n", player.name));
            }
        }
    }

    if let Some(stage) = data.stage_stack.last() {
        output.push_str(&format!("Current Stage: {}\n", stage));
    }

    output.push_str(&format!("Turn Order Indices: {:?}\n", data.turn_order));

    output.push_str("\nScores:\n");
    for player in &data.players {
        output.push_str(&format!("  {}: {}\n", player.name, player.score));
    }

    output.push_str("\nTeams:\n");
    for team in &data.teams {
        let member_names: Vec<_> = team
            .players
            .iter()
            .filter_map(|idx| data.players.get(*idx).map(|p| p.name.clone()))
            .collect();
        output.push_str(&format!("  {}: {}\n", team.name, member_names.join(", ")));
    }

    output.push_str("\nMemories:\n");
    for (key, value) in &data.memories {
        let value_str = match value {
            crate::game_data::MemoryValue::Int(i) => i.to_string(),
            crate::game_data::MemoryValue::String(s) => s.clone(),
            crate::game_data::MemoryValue::CardSet(ids) => format!("{:?}", ids),
            crate::game_data::MemoryValue::PlayerCollection(ids) => format!("{:?}", ids),
            crate::game_data::MemoryValue::Team(s) => s.clone(),
            crate::game_data::MemoryValue::IntCollection(ids) => format!("{:?}", ids),
            crate::game_data::MemoryValue::StringCollection(ids) => format!("{:?}", ids),
            crate::game_data::MemoryValue::LocationCollection(ids) => format!("{:?}", ids),
        };
        output.push_str(&format!("  {}: {}\n", key, value_str));
    }

    output.push_str("\nCard Counts per Location:\n");
    for location in &data.locations {
        let total = location.cards.len();
        let display = if total > 5 {
            let first_5: Vec<_> = location
                .cards
                .iter()
                .take(5)
                .filter_map(|id| data.cards.get(*id))
                .map(|c| {
                    c.get("name")
                        .cloned()
                        .unwrap_or_else(|| "Unnamed".to_string())
                })
                .collect::<Vec<_>>();
            format!("{} (first 5: {}, ...)", total, first_5.join(", "))
        } else {
            let card_names: Vec<_> = location
                .cards
                .iter()
                .filter_map(|id| data.cards.get(*id))
                .map(|c| {
                    c.get("name")
                        .cloned()
                        .unwrap_or_else(|| "Unnamed".to_string())
                })
                .collect::<Vec<_>>();
            format!("{} ({})", total, card_names.join(", "))
        };
        output.push_str(&format!("  {}: {}\n", location.name, display));
    }

    output
}

fn format_game_data_high(data: &GameData) -> String {
    let mut output = String::new();
    output.push_str("=== GAME DATA (HIGH) ===\n\n");

    output.push_str("Players:\n");
    for (i, player) in data.players.iter().enumerate() {
        output.push_str(&format!(
            "  [{}] {}: score={}, in_game={}, in_stage={:?}\n",
            i, player.name, player.score, player.in_game, player.in_stage
        ));
    }

    output.push_str("\nTeams:\n");
    for (i, team) in data.teams.iter().enumerate() {
        output.push_str(&format!("  [{}] {}: {:?}\n", i, team.name, team.players));
    }

    output.push_str("\nTurn Order:\n");
    output.push_str(&format!("  Indices: {:?}\n", data.turn_order));
    output.push_str(&format!(
        "  Current Player Index: {:?}\n",
        data.current_player
    ));

    output.push_str("\nLocations:\n");
    for (i, location) in data.locations.iter().enumerate() {
        output.push_str(&format!(
            "  [{}] {}: {:?}\n",
            i, location.name, location.cards
        ));
    }

    output.push_str("\nCards:\n");
    for (i, card) in data.cards.iter().enumerate() {
        output.push_str(&format!("  [{}] {:?}\n", i, card));
    }

    output.push_str("\nStage Stack:\n");
    output.push_str(&format!("  {:?}\n", data.stage_stack));

    output.push_str("\nStage Counters:\n");
    for (stage, counter) in &data.stage_counters {
        output.push_str(&format!("  {}: {}\n", stage, counter));
    }

    output.push_str("\nMemories:\n");
    for (key, value) in &data.memories {
        let value_str = match value {
            crate::game_data::MemoryValue::Int(i) => format!("Int({})", i),
            crate::game_data::MemoryValue::String(s) => format!("String(\"{}\")", s),
            crate::game_data::MemoryValue::CardSet(ids) => format!("CardSet({:?})", ids),
            crate::game_data::MemoryValue::PlayerCollection(ids) => {
                format!("PlayerCollection({:?})", ids)
            }
            crate::game_data::MemoryValue::Team(s) => format!("Team(\"{}\")", s),
            crate::game_data::MemoryValue::IntCollection(ids) => {
                format!("IntCollection({:?})", ids)
            }
            crate::game_data::MemoryValue::StringCollection(ids) => {
                format!("StringCollection({:?})", ids)
            }
            crate::game_data::MemoryValue::LocationCollection(ids) => {
                format!("LocationCollection({:?})", ids)
            }
        };
        output.push_str(&format!("  {}: {}\n", key, value_str));
    }

    output.push_str("\nCombos:\n");
    for (i, combo) in data.combos.iter().enumerate() {
        output.push_str(&format!("  [{}] {}: {:?}\n", i, combo.name, combo.filter));
    }

    output.push_str("\nPrecedences:\n");
    for (i, precedence) in data.precedences.iter().enumerate() {
        output.push_str(&format!(
            "  [{}] {} (key={}): {:?}\n",
            i, precedence.name, precedence.key, precedence.values
        ));
    }

    output.push_str("\nPoint Maps:\n");
    for (i, point_map) in data.point_maps.iter().enumerate() {
        output.push_str(&format!(
            "  [{}] {}: {:?}\n",
            i, point_map.name, point_map.map
        ));
    }

    output
}

pub fn print_game_data(data: &GameData, level: DebugLevel) {
    println!("{}", format_game_data(data, level));
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_level_from_marker_valid() {
        assert_eq!(DebugLevel::from_marker("<!--LOW-->"), Some(DebugLevel::Low));
        assert_eq!(
            DebugLevel::from_marker("<!--MEDIUM-->"),
            Some(DebugLevel::Medium)
        );
        assert_eq!(
            DebugLevel::from_marker("<!--HIGH-->"),
            Some(DebugLevel::High)
        );
    }

    #[test]
    fn test_debug_level_from_marker_case_insensitive() {
        assert_eq!(DebugLevel::from_marker("<!--low-->"), Some(DebugLevel::Low));
        assert_eq!(
            DebugLevel::from_marker("<!--Medium-->"),
            Some(DebugLevel::Medium)
        );
        assert_eq!(
            DebugLevel::from_marker("<!--HIGH-->"),
            Some(DebugLevel::High)
        );
    }

    #[test]
    fn test_debug_level_from_marker_invalid() {
        assert_eq!(DebugLevel::from_marker("<!--INVALID-->"), None);
        assert_eq!(DebugLevel::from_marker("low"), None);
        assert_eq!(DebugLevel::from_marker(""), None);
    }

    #[test]
    fn test_debug_level_marker_roundtrip() {
        assert_eq!(DebugLevel::Low.marker(), "<!--LOW-->");
        assert_eq!(DebugLevel::Medium.marker(), "<!--MEDIUM-->");
        assert_eq!(DebugLevel::High.marker(), "<!--HIGH-->");
    }

    #[test]
    fn test_format_game_data_low() {
        let data = GameData::new();
        let output = format_game_data(&data, DebugLevel::Low);
        assert!(!output.is_empty());
        assert!(output.contains("GAME DATA (LOW)"));
        assert!(output.contains("Players:"));
        assert!(output.contains("Turn Order Indices:"));
        assert!(output.contains("Card Counts per Location:"));
    }

    #[test]
    fn test_format_game_data_medium() {
        let data = GameData::new();
        let output = format_game_data(&data, DebugLevel::Medium);
        assert!(!output.is_empty());
        assert!(output.contains("GAME DATA (MEDIUM)"));
        assert!(output.contains("Scores:"));
        assert!(output.contains("Teams:"));
        assert!(output.contains("Memories:"));
    }

    #[test]
    fn test_format_game_data_high() {
        let data = GameData::new();
        let output = format_game_data(&data, DebugLevel::High);
        assert!(!output.is_empty());
        assert!(output.contains("GAME DATA (HIGH)"));
        assert!(output.contains("Players:"));
        assert!(output.contains("Cards:"));
        assert!(output.contains("Combos:"));
        assert!(output.contains("Precedences:"));
        assert!(output.contains("Point Maps:"));
    }

    #[test]
    fn test_save_game_data_creates_file() {
        let data = GameData::new();
        let path = std::path::PathBuf::from("/tmp/test_mcg_debug.txt");
        let _ = fs::remove_file(&path);

        let result = save_game_data(&data, &path);
        assert!(result.is_ok());
        assert!(path.exists());

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_save_game_data_appends_to_file() {
        let data = GameData::new();
        let path = std::path::PathBuf::from("/tmp/test_mcg_debug_append.txt");
        let _ = fs::remove_file(&path);

        save_game_data(&data, &path).unwrap();
        save_game_data(&data, &path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let count = content.matches("=== GAME DATA").count();
        assert_eq!(count, 2);

        let _ = fs::remove_file(&path);
    }
}
