use eframe::Frame;
use egui::{Context, vec2};
use std::rc::Rc;

use crate::game::card::{CardConfig, SimpleCard};
use crate::game::field::{SimpleField, SimpleFieldKind::Stack};
use super::{ScreenWidget, ScreenType, GameConfig, DirectoryCardType, AppInterface, back_button};

pub struct GameSetupScreen { pub card_config: Option<DirectoryCardType>, pub players: usize, pub theme_index: usize, pub is_dnd_variant: bool }
impl GameSetupScreen {
    pub fn new() -> Self { let card_config = None; let players = 2; let theme_index = crate::hardcoded_cards::AVAILABLE_THEMES.iter().position(|&t| t == crate::hardcoded_cards::DEFAULT_THEME).unwrap_or(0); Self { card_config, players, theme_index, is_dnd_variant: false } }
    pub fn new_dnd() -> Self { let mut screen = Self::new(); screen.is_dnd_variant = true; screen }
    pub fn generate_config(&self) -> Option<GameConfig<DirectoryCardType>> {
        let card_config = self.card_config.as_ref()?.clone();
        let mut players: Vec<(String, SimpleField<SimpleCard, DirectoryCardType>)> = (0..self.players)
            .map(|i| (format!("Player {}", i + 1), SimpleField::new(Rc::new(card_config.clone())).max_cards(4).selectable(true).max_card_size(vec2(100.0, 150.0))))
            .collect();
        let mut stack = SimpleField::new(Rc::new(card_config.clone())).kind(Stack).max_card_size(vec2(100.0, 150.0));
        for i in 0..card_config.T() { let card = SimpleCard::Open(i); stack.push(card.clone()); players[i % self.players].1.push(SimpleCard::Open(i)); }
        Some(GameConfig { players, stack })
    }
}
impl Default for GameSetupScreen { fn default() -> Self { Self::new() } }
impl ScreenWidget for GameSetupScreen {
    fn update(&mut self, app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| { ui.label("Card Pack:"); match &self.card_config { None => ui.label("Deck not loaded"), Some(config) => ui.label(format!("Using {}", &config.path)), }; });
            ui.horizontal(|ui| {
                ui.label("Theme:");
                egui::ComboBox::new("theme_selector", "Theme")
                    .selected_text(crate::hardcoded_cards::AVAILABLE_THEMES[self.theme_index])
                    .show_ui(ui, |ui| {
                        for (i, theme) in crate::hardcoded_cards::AVAILABLE_THEMES.iter().enumerate() {
                            let theme_name = match *theme { "img_cards" => "Standard Cards", "alt_cards" => "Alternative Cards", _ => theme };
                            if ui.selectable_label(self.theme_index == i, theme_name).clicked() {
                                self.theme_index = i; let theme_str = crate::hardcoded_cards::AVAILABLE_THEMES[self.theme_index]; crate::hardcoded_cards::set_deck_by_theme(&mut self.card_config, theme_str);
                            }
                        }
                    });
            });
            ui.horizontal(|ui| { ui.label("# Players"); let drag = egui::DragValue::new(&mut self.players); ui.add(drag); let dec = egui::Button::new("-").min_size(vec2(30.0, 0.0)); if ui.add(dec).clicked() && self.players > 1 { self.players = self.players.saturating_sub(1); } let inc = egui::Button::new("+").min_size(vec2(30.0, 0.0)); if ui.add(inc).clicked() { self.players = self.players.saturating_add(1); } });
            if ui.button("Start Game").clicked() { if let Some(config) = self.generate_config() { if self.is_dnd_variant { app_interface.queue_event(crate::game::AppEvent::StartDndGame(config)); } else { app_interface.queue_event(crate::game::AppEvent::StartGame(config)); } } }
            back_button(ui, app_interface, ScreenType::Main, "Back");
        });
    }
}
