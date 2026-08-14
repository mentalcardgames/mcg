use crate::game_data::GameData;

pub(super) fn format_game_data_low(data: &GameData) -> String {
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
