use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::{vec2, Align, Button, Color32, Context, Grid, Layout, RichText, ScrollArea};

use super::{ScreenType, ScreenWidget};
use crate::sprintln;

/// Represents a player with pairing functionality
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

/// Pairing screen with a list of players and pairing functionality
pub struct PairingScreen {
    players: Vec<Player>,
}

impl PairingScreen {
    pub fn new() -> Self {
        // Create dummy data with several players
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

        Self { players }
    }
}

impl Default for PairingScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for PairingScreen {
    fn update(&mut self, next_screen: Rc<RefCell<ScreenType>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Player Pairing");

            // Back to Main Menu button
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                if ui.button("Back to Main Menu").clicked() {
                    *next_screen.borrow_mut() = ScreenType::Main;
                    sprintln!("Navigating back to Main Menu");
                }
            });

            // Scrollable area for player list
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Create a grid layout for our player list
                    Grid::new("player_grid")
                        .num_columns(3)
                        .spacing([20.0, 10.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Header row
                            ui.strong("Player");
                            ui.strong("Status");
                            ui.strong("Action");
                            ui.end_row();

                            // Player rows
                            for player in &mut self.players {
                                // Column 1: Player icon and name
                                ui.horizontal(|ui| {
                                    // Display user icon/emoji
                                    let icon = if player.paired { "👥" } else { "👤" };
                                    ui.label(RichText::new(icon).size(24.0));
                                    ui.label(&player.name);
                                });

                                // Column 2: Pairing status
                                let status_text = if player.paired {
                                    RichText::new("Paired").color(Color32::GREEN)
                                } else {
                                    RichText::new("Not Paired").color(Color32::RED)
                                };
                                ui.label(status_text);

                                // Column 3: Pairing button
                                let button_text = if player.paired { "Unpair" } else { "Pair" };
                                let button_color = if player.paired {
                                    Color32::from_rgb(255, 180, 180) // Lighter red
                                } else {
                                    Color32::from_rgb(180, 255, 180) // Lighter green
                                };

                                if ui
                                    .add(
                                        Button::new(
                                            RichText::new(button_text).color(Color32::BLACK),
                                        )
                                        .fill(button_color)
                                        .min_size(vec2(80.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    // Toggle the pairing status
                                    player.paired = !player.paired;
                                    sprintln!(
                                        "Player {} is now {}",
                                        player.name,
                                        if player.paired { "paired" } else { "unpaired" }
                                    );
                                }

                                ui.end_row();
                            }
                        });
                });
        });
    }
}
