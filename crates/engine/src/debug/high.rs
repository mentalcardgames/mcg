use crate::game_data::{Card, GameData};
use std::collections::HashMap;

fn format_card(card: &Card) -> String {
    let mut items: Vec<String> = card
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect();
    items.sort();
    format!("{{{}}}", items.join(", "))
}

fn owner_names(data: &GameData) -> HashMap<usize, String> {
    let mut map = HashMap::new();
    for &loc_id in &data.table.locations {
        map.insert(loc_id, "Table".to_string());
    }
    for player in &data.players {
        for &loc_id in &player.owner.locations {
            map.insert(loc_id, player.name.clone());
        }
    }
    map
}

fn location_label(i: usize, location: &crate::game_data::Location, owners: &HashMap<usize, String>) -> String {
    let owner = owners.get(&i).map(|s| s.as_str()).unwrap_or("?");
    format!("{}:{}", owner, location.name)
}

pub(super) fn format_game_data_high(data: &GameData) -> String {
    let owners = owner_names(data);
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
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&format!(
            "  [{}] {} ({} cards):\n",
            i,
            location_label(i, location, &owners),
            location.cards.len()
        ));
        for &card_id in &location.cards {
            if let Some(card) = data.get_card(card_id) {
                output.push_str(&format!("     [{}] {}\n", card_id, format_card(card)));
            }
        }
    }

    output.push_str(&format!("\nTotal cards: {}\n", data.cards.len()));
    let in_any_location: std::collections::HashSet<usize> = data
        .locations
        .iter()
        .flat_map(|l| l.cards.iter().copied())
        .collect();
    let orphaned: Vec<usize> = (0..data.cards.len())
        .filter(|i| !in_any_location.contains(i))
        .collect();
    if !orphaned.is_empty() {
        output.push_str("\nOrphaned cards (not in any location):\n");
        for &id in &orphaned {
            if let Some(card) = data.get_card(id) {
                output.push_str(&format!("  [{}] {}\n", id, format_card(card)));
            }
        }
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
