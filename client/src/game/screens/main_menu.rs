use eframe::Frame;
use egui::{vec2, FontId, RichText};

use super::{AppInterface, ScreenRegistry, ScreenWidget};

pub struct MainMenu {
    screen_registry: ScreenRegistry,
}
impl MainMenu {
    pub fn new() -> Self {
        Self {
            screen_registry: ScreenRegistry::new(),
        }
    }
}
impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for MainMenu {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        // Center content both vertically and horizontally
        ui.centered_and_justified(|ui| {
            // Add margins around the main menu content
            ui.add_space(32.0);
            ui.horizontal_centered(|ui| {
                ui.add_space(32.0);
                ui.vertical(|ui| {
                    // Title
                    ui.label(
                        RichText::new("🎮 Main Menu")
                            .font(FontId::proportional(24.0))
                            .strong(),
                    );
                    ui.add_space(20.0);

                    // Grid layout for buttons
                    ui.horizontal_wrapped(|ui| {
                        // Set spacing between buttons
                        ui.spacing_mut().item_spacing = vec2(20.0, 20.0);

                        let button_size = vec2(180.0, 80.0);

                        // Generate buttons from screen registry
                        let menu_screens = self.screen_registry.get_menu_screens();
                        for screen_type in menu_screens {
                            let metadata = self.screen_registry.get_metadata(screen_type);
                            let label = format!("{} {}", metadata.icon, metadata.display_name);

                            let button = egui::Button::new(
                                RichText::new(&label).font(FontId::proportional(16.0)),
                            );

                            if ui.add_sized(button_size, button).clicked() {
                                eprintln!("{} opened", metadata.display_name);
                                app_interface
                                    .queue_event(crate::game::AppEvent::ChangeScreen(screen_type));
                            }
                        }
                    });
                    ui.add_space(32.0);
                });
                ui.add_space(32.0);
            });
        });
    }
}
