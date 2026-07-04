use crate::game_data::GameData;

pub(super) fn format_game_data_medium(data: &GameData) -> String {
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
