use eframe::Frame;
use egui::{vec2, Align, Button, Color32, Grid, Layout, RichText, ScrollArea};

use super::{AppInterface, ScreenWidget};
use crate::sprintln;
use crate::utils::emoji_hash;

pub struct Player {
    pub name: String,
    pub paired: bool,
}
impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            paired: false,
        }
    }
}

pub struct PairingScreen {
    players: Vec<Player>,
    confirm_action_player: Option<String>,
    confirm_action: Option<bool>,
}
impl PairingScreen {
    pub fn new() -> Self {
        let players = vec![
            Player::new("Alice"),
            Player::new("Bob"),
            Player::new("Charlie"),
            Player::new("David"),
            Player::new("Eve"),
            Player::new("Frank"),
            Player::new("Grace"),
            Player::new("Heidi"),
            Player::new("Ivan"),
            Player::new("Julia"),
            Player::new("Kevin"),
            Player::new("Laura"),
            Player::new("Michael"),
            Player::new("Natalie"),
            Player::new("Oscar"),
            Player::new("Patricia"),
        ];
        Self {
            players,
            confirm_action_player: None,
            confirm_action: None,
        }
    }
}
impl Default for PairingScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for PairingScreen {
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.heading("Player Pairing");
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            // Global Back button is provided by the layout
            ui.add_space(0.0);
        });
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Grid::new("player_grid")
                    .num_columns(3)
                    .spacing([20.0, 10.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Player");
                        ui.strong("Status");
                        ui.strong("Action");
                        ui.end_row();
                        for player in &mut self.players {
                            ui.horizontal(|ui| {
                                let icon = if player.paired { "👥" } else { "👤" };
                                ui.label(RichText::new(icon).size(24.0));
                                ui.label(&player.name);
                            });
                            let status_text = if player.paired {
                                RichText::new("Paired").color(Color32::GREEN)
                            } else {
                                RichText::new("Not Paired").color(Color32::RED)
                            };
                            ui.label(status_text);
                            let button_text = if player.paired { "Unpair" } else { "Pair" };
                            let button_color = if player.paired {
                                Color32::from_rgb(255, 180, 180)
                            } else {
                                Color32::from_rgb(180, 255, 180)
                            };
                            if ui
                                .add(
                                    Button::new(RichText::new(button_text).color(Color32::BLACK))
                                        .fill(button_color)
                                        .min_size(vec2(80.0, 24.0)),
                                )
                                .clicked()
                            {
                                self.confirm_action_player = Some(player.name.clone());
                                self.confirm_action = Some(!player.paired);
                            }
                            ui.end_row();
                        }
                    });
            });
        if let (Some(player_name), Some(pair_action)) =
            (&self.confirm_action_player, self.confirm_action)
        {
            let action_text = if pair_action { "pair" } else { "unpair" };
            let player_name_clone = player_name.clone();
            let player_hash = emoji_hash(player_name_clone.as_bytes(), ui.ctx());
            egui::Window::new(format!("Confirm {}", action_text))
                .collapsible(false)
                .resizable(false)
                .min_width(300.0)
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&player_hash).size(48.0).strong());
                        ui.add_space(8.0);
                    });
                    ui.separator();
                    ui.label(format!(
                        "Are you sure you want to {} {}?",
                        action_text, player_name_clone
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.confirm_action_player = None;
                            self.confirm_action = None;
                        }
                        let mut perform_action = false;
                        if ui.button("Confirm").clicked() {
                            perform_action = true;
                        }
                        if perform_action {
                            for player in &mut self.players {
                                if player.name == player_name_clone {
                                    player.paired = pair_action;
                                    sprintln!(
                                        "Player {} ({}) is now {}",
                                        player.name,
                                        emoji_hash(player.name.as_bytes(), ui.ctx()),
                                        if player.paired { "paired" } else { "unpaired" }
                                    );
                                    break;
                                }
                            }
                            self.confirm_action_player = None;
                            self.confirm_action = None;
                        }
                    });
                });
        }
    }
}
