use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::{Context, RichText, vec2, Color32};

use crate::sprintln;
use super::{ScreenWidget, ScreenType};

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
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        // Configure panel with centered content
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create a centered layout with content taking up 70% of screen width
            ui.horizontal(|ui| {
                // Calculate widths
                let available_width = ui.available_width();
                let content_width = available_width * 0.7;
                let margin = (available_width - content_width) / 2.0;

                // Add left margin (15% of screen width)
                ui.add_space(margin);

                // Create content area
                ui.vertical(|ui| {
                    // Ensure content area has correct width
                    ui.set_max_width(content_width);

                    // Header with larger text (centered)
                    ui.vertical_centered(|ui| {
                        ui.heading(RichText::new("Player Pairing").size(32.0).strong());
                    });
                    ui.add_space(30.0);

                    // Back button at the top (centered)
                    ui.vertical_centered(|ui| {
                        let back_button_size = vec2(160.0, 36.0);
                        if ui.add_sized(back_button_size, egui::Button::new("Back to Main Menu")).clicked() {
                            *next_screen.borrow_mut() = ScreenType::Main.to_string();
                        }
                    });

                    ui.add_space(30.0);
                    ui.separator();
                    ui.add_space(20.0);

                    // Create a scrollable area for players
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Table-like header
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            ui.label(RichText::new("Player Name").size(18.0).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(120.0); // Space for button
                                ui.label(RichText::new("Status").size(18.0).strong());
                                ui.add_space(20.0);
                            });
                        });
                        
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        // Player list with pairing buttons
                        for player in &mut self.players {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                // Player name with icon
                                ui.label(RichText::new(format!("👤 {}", player.name)).size(18.0));
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Button styling
                                    let button_text = if player.paired {
                                        "Unpair"
                                    } else {
                                        "Pair"
                                    };
                                    
                                    let button_size = vec2(100.0, 30.0);
                                    let button = ui.add_sized(
                                        button_size, 
                                        egui::Button::new(button_text)
                                            .fill(if player.paired {
                                                Color32::from_rgb(200, 100, 100) // Red for unpair
                                            } else {
                                                Color32::from_rgb(100, 180, 100) // Green for pair
                                            })
                                    );
                                    
                                    if button.clicked() {
                                        player.paired = !player.paired;
                                        let status = if player.paired { "paired" } else { "unpaired" };
                                        sprintln!("Player {} {}", player.name, status);
                                    }
                                    
                                    // Display status with color
                                    let status_text = if player.paired {
                                        RichText::new("● Paired").color(Color32::from_rgb(100, 200, 100))
                                    } else {
                                        RichText::new("○ Unpaired").color(Color32::from_rgb(150, 150, 150))
                                    };
                                    ui.label(status_text);
                                    ui.add_space(20.0);
                                });
                            });
                            ui.add_space(8.0);
                            ui.separator();
                        }
                    });
                });
                
                // Right margin is automatically filled by the horizontal layout
            });
        });
    }
}