use eframe::Frame;
use egui::{vec2, FontId, RichText};

use super::{AppInterface, ScreenType, ScreenWidget};

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
    fn ui(&mut self, app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        // Center content both vertically and horizontally
        ui.centered_and_justified(|ui| {
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
                    let buttons = [
                        ("🎮 Start", "setup started", Some(ScreenType::GameSetup)),
                        (
                            "🎯 Drag & Drop",
                            "game_dnd opened",
                            Some(ScreenType::GameDndSetup),
                        ),
                        ("📱 Pairing", "pairing opened", Some(ScreenType::Pairing)),
                        ("⚙️ Settings", "settings opened", Some(ScreenType::Settings)),
                        ("🖱️ DND Test", "dnd_test opened", Some(ScreenType::DndTest)),
                        ("📚 Articles", "articles opened", Some(ScreenType::Articles)),
                        ("🔍 QR Test", "qr_test opened", Some(ScreenType::QRScreen)),
                        (
                            "🃏 Poker Online",
                            "poker online opened",
                            Some(ScreenType::PokerOnline),
                        ),
                        ("🖨️ Print Screen", "Print Screen clicked", None),
                    ];

                    for (label, message, screen_type) in buttons.iter() {
                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(
                                    RichText::new(*label).font(FontId::proportional(16.0)),
                                ),
                            )
                            .clicked()
                        {
                            if *label == "🖨️ Print Screen" {
                                eprintln!("Print Screen clicked");
                            } else {
                                eprintln!("{}", message);
                                if let Some(screen) = screen_type {
                                    app_interface
                                        .queue_event(crate::game::AppEvent::ChangeScreen(*screen));
                                }
                            }
                        }
                    }
                });
            });
        });
    }
}
