use eframe::Frame;
use egui::{vec2, FontId, RichText};

use super::{AppInterface, ScreenWidget};

/// Example screen to demonstrate the new generalized screen system
pub struct ExampleScreen {
    counter: i32,
    text_input: String,
}

impl ExampleScreen {
    pub fn new() -> Self {
        Self {
            counter: 0,
            text_input: String::new(),
        }
    }
}

impl Default for ExampleScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for ExampleScreen {
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                // Title
                ui.label(
                    RichText::new("🧪 Example Screen")
                        .font(FontId::proportional(24.0))
                        .strong(),
                );

                ui.add_space(20.0);

                // Description
                ui.label(
                    "This is an example screen to demonstrate how easy it is to add new screens!",
                );

                ui.add_space(20.0);

                // Counter demo
                ui.horizontal(|ui| {
                    ui.label("Counter:");
                    ui.label(format!("{}", self.counter));
                });

                ui.horizontal(|ui| {
                    if ui.button("➕ Increment").clicked() {
                        self.counter += 1;
                    }
                    if ui.button("➖ Decrement").clicked() {
                        self.counter -= 1;
                    }
                    if ui.button("🔄 Reset").clicked() {
                        self.counter = 0;
                    }
                });

                ui.add_space(20.0);

                // Text input demo
                ui.horizontal(|ui| {
                    ui.label("Text input:");
                    ui.text_edit_singleline(&mut self.text_input);
                });

                if !self.text_input.is_empty() {
                    ui.label(format!("You typed: {}", self.text_input));
                }

                ui.add_space(20.0);

                // Back to main menu button
                if ui
                    .add_sized(
                        vec2(200.0, 40.0),
                        egui::Button::new(
                            RichText::new("🏠 Back to Main Menu").font(FontId::proportional(16.0)),
                        ),
                    )
                    .clicked()
                {
                    app_interface
                        .queue_event(crate::game::AppEvent::ChangeScreen(super::ScreenType::Main));
                }

                ui.add_space(20.0);

                // Instructions
                ui.separator();
                ui.add_space(10.0);
                ui.label(
                    RichText::new("To add this screen to the system:")
                        .font(FontId::proportional(14.0))
                        .strong(),
                );
                ui.label("1. Add 'Example' to the ScreenType enum");
                ui.label("2. Add metadata in the ScreenType::metadata() method");
                ui.label("3. That's it! The router and main menu will automatically work.");
            });
        });
    }
}
