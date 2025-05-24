use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::{vec2, Color32, Context, Grid, RichText};

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
            ui.vertical_centered(|ui| {
                ui.add_space(50.0); // Add top spacing

                // Title
                ui.heading(RichText::new("Player Pairing").size(32.0).strong());
                ui.add_space(30.0);

                // Player list in a scrollable area
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let button_size = vec2(200.0, 40.0); // Consistent button size

                    // Player name and status
                    Grid::new("pairing").show(ui, |ui| {
                        for player in &mut self.players {
                            // Player name with icon
                            ui.label(RichText::new(format!("👤 {}", player.name)).size(18.0));

                            // Status indicator
                            let status_text = if player.paired {
                                RichText::new("● Paired").color(Color32::from_rgb(100, 200, 100))
                            } else {
                                RichText::new("○ Unpaired").color(Color32::from_rgb(150, 150, 150))
                            };
                            ui.label(status_text);

                            let button_text = if player.paired { "Unpair" } else { "Pair" };
                            let (button_fill, text_color) = if player.paired {
                                (Color32::from_rgb(220, 60, 60), Color32::WHITE)
                            // Red with white text
                            } else {
                                (Color32::from_rgb(60, 200, 60), Color32::BLACK)
                                // Green with black text
                            };

                            if ui
                                .add_sized(
                                    button_size,
                                    egui::Button::new(
                                        RichText::new(button_text)
                                            .color(text_color)
                                            .size(16.0)
                                            .strong(),
                                    )
                                    .fill(button_fill),
                                )
                                .clicked()
                            {
                                player.paired = !player.paired;
                                let status = if player.paired { "paired" } else { "unpaired" };
                                sprintln!("Player {} {}", player.name, status);
                            }
                            ui.end_row();
                        }
                    });
                });

                ui.add_space(30.0);

                // Back button
                let back_button_size = vec2(200.0, 40.0);
                if ui
                    .add_sized(back_button_size, egui::Button::new("Back to Main Menu"))
                    .clicked()
                {
                    *next_screen.borrow_mut() = ScreenType::Main;
                }

                ui.add_space(50.0); // Add bottom spacing
            });
        });
    }
}
