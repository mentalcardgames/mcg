use crate::game_data::GameData;

pub(super) fn format_game_data_high(data: &GameData) -> String {
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
