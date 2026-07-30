use eframe::Frame;
use egui::{vec2, FontId, RichText};
use crate::widgets::screen::{ScreenRegistry, ScreenWidget};
use crate::app::AppInterface;

#[derive(Default)]
pub struct MainMenu {
    screen_registry: ScreenRegistry,
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
                        let menu = self.screen_registry.menu_entries();
                        for (screen_id, meta) in menu {
                            let label = format!("{} {}", meta.icon, meta.display_name);

                            let button = egui::Button::new(
                                RichText::new(&label).font(FontId::proportional(16.0)),
                            );

                            if ui.add_sized(button_size, button).clicked() {
                                eprintln!("{} opened", meta.display_name);
                                app_interface.change_screen_id(screen_id);
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

crate::impl_screen_def!(MainMenu, "/", "Main", "🎮", "Main menu", false);
