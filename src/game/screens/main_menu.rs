use eframe::Frame;
use egui::{vec2, Context};

use super::{ScreenType, ScreenWidget, AppInterface};
use crate::sprintln;

/// Main menu screen
pub struct MainMenu {}

impl MainMenu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for MainMenu {
    fn update(&mut self, app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0); // Add top spacing

                let button_size = vec2(200.0, 40.0); // Consistent button size

                let buttons = [
                    ("Start", "setup started", Some(ScreenType::GameSetup)),
                    (
                        "Drag & Drop Game",
                        "game_dnd opened",
                        Some(ScreenType::GameDndSetup),
                    ),
                    ("Pairing", "pairing opened", Some(ScreenType::Pairing)),
                    ("Settings", "settings opened", Some(ScreenType::Settings)),
                    (
                        "Drag & Drop Test",
                        "dnd_test opened",
                        Some(ScreenType::DndTest),
                    ),
                    ("Articles", "articles opened", Some(ScreenType::Articles)),
                    ("QR Test", "qr_test opened", Some(ScreenType::QRScreen)),
                    ("Print Screen", "", None),
                ];

                for (i, (label, message, screen_type)) in buttons.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(20.0); // Add spacing between buttons
                    }

                    if ui
                        .add_sized(button_size, egui::Button::new(*label))
                        .clicked()
                    {
                        if *label == "Print Screen" {
                            sprintln!("Print Screen clicked");
                        } else {
                            sprintln!("{}", message);
                            if let Some(screen) = screen_type {
                                app_interface.queue_event(crate::game::AppEvent::ChangeScreen(*screen));
                            }
                        }
                    }
                }

                ui.add_space(50.0); // Add bottom spacing
            });
        });
    }
}
