use egui::{Context, Ui};
use mcg_shared::{PlayerConfig, PlayerId};

pub fn render_player_setup(ui: &mut Ui, _ctx: &Context) {
    ui.heading("Player Setup");
    ui.add_space(8.0);

    // This will be implemented by the screen using PlayerManager
    ui.add_space(16.0);

    render_start_game_button(ui);
    add_game_instructions(ui);
}

fn render_start_game_button(ui: &mut Ui) {
    if ui.button("Start Game").clicked() {
        // This will be handled by the screen
    }
}

fn add_game_instructions(_ui: &mut Ui) {
    // Instructions will be handled by the screen
}

pub struct PlayerManager {
    players: Vec<PlayerConfig>,
    next_player_id: usize,
    new_player_name: String,
    preferred_player: PlayerId,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            players: vec![
                PlayerConfig {
                    id: mcg_shared::PlayerId(0),
                    name: "You".to_string(),
                    is_bot: false,
                },
                PlayerConfig {
                    id: mcg_shared::PlayerId(1),
                    name: "Bot 1".to_string(),
                    is_bot: true,
                },
                PlayerConfig {
                    id: mcg_shared::PlayerId(2),
                    name: "Bot 2".to_string(),
                    is_bot: true,
                },
                PlayerConfig {
                    id: mcg_shared::PlayerId(3),
                    name: "Bot 3".to_string(),
                    is_bot: true,
                },
            ],
            next_player_id: 4,
            new_player_name: String::new(),
            preferred_player: PlayerId(0),
        }
    }

    pub fn get_players(&self) -> &Vec<PlayerConfig> {
        &self.players
    }

    pub fn get_players_mut(&mut self) -> &mut Vec<PlayerConfig> {
        &mut self.players
    }

    pub fn get_preferred_player(&self) -> PlayerId {
        self.preferred_player
    }

    pub fn get_preferred_player_mut(&mut self) -> &mut PlayerId {
        &mut self.preferred_player
    }

    pub fn get_new_player_name_mut(&mut self) -> &mut String {
        &mut self.new_player_name
    }

    pub fn add_new_player(&mut self) {
        let player_name = if self.new_player_name.is_empty() {
            self.generate_random_name()
        } else {
            self.new_player_name.clone()
        };

        self.players.push(PlayerConfig {
            id: mcg_shared::PlayerId(self.next_player_id),
            name: player_name,
            is_bot: true, // New players start as bots by default
        });
        self.next_player_id += 1;
        self.new_player_name.clear();
    }

    // Generate a random name that doesn't conflict with existing player names
    fn generate_random_name(&self) -> String {
        let random_names = Self::get_random_name_pool();
        let existing_names: std::collections::HashSet<&str> = self.get_existing_names();

        // Try to find a name that's not already used
        if let Some(name) = self.find_unused_name(&random_names, &existing_names) {
            return name;
        }

        // If all names are used, append a number
        if let Some(name) = self.find_available_name_with_number(&random_names, &existing_names) {
            return name;
        }

        // Fallback: use a timestamp-based name
        self.generate_timestamp_name()
    }

    fn get_random_name_pool() -> [&'static str; 48] {
        [
            "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
            "Kate", "Leo", "Mia", "Noah", "Olivia", "Peter", "Quinn", "Rose", "Sam", "Tina", "Uma",
            "Victor", "Wendy", "Xander", "Yara", "Zoe", "Alex", "Blake", "Casey", "Dylan", "Erin",
            "Finn", "Gabe", "Holly", "Ian", "Jade", "Kyle", "Luna", "Max", "Nora", "Owen", "Piper",
            "Ryan", "Sage", "Tyler", "Violet", "Wyatt", "Zara",
        ]
    }

    fn get_existing_names(&self) -> std::collections::HashSet<&str> {
        self.players.iter().map(|p| p.name.as_str()).collect()
    }

    fn find_unused_name(
        &self,
        random_names: &[&str],
        existing_names: &std::collections::HashSet<&str>,
    ) -> Option<String> {
        for &name in random_names {
            if !existing_names.contains(name) {
                return Some(name.to_string());
            }
        }
        None
    }

    fn find_available_name_with_number(
        &self,
        random_names: &[&str],
        existing_names: &std::collections::HashSet<&str>,
    ) -> Option<String> {
        for &base_name in random_names {
            for i in 2..100 {
                // Try numbers 2-99
                let candidate = format!("{} {}", base_name, i);
                if !existing_names.contains(candidate.as_str()) {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn generate_timestamp_name(&self) -> String {
        format!(
            "Player {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
    }
}
