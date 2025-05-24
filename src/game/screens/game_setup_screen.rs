use std::cell::RefCell;
use std::rc::{Rc, Weak};

use eframe::Frame;
use egui::{Context, vec2};

use crate::game::card::{CardConfig, SimpleCard};
use crate::game::field::{SimpleField, SimpleFieldKind::Stack};
use super::{ScreenWidget, ScreenType, Game, GameConfig, DirectoryCardType};

/// Game setup screen for configuring players and cards
pub struct GameSetupScreen<C: CardConfig = DirectoryCardType, G = Game<C>> {
    pub directory: Rc<RefCell<Option<C>>>,
    pub players: usize,
    pub theme_index: usize,
    pub(crate) game_widget: Weak<RefCell<G>>,
}

impl<C: CardConfig + Clone, G> GameSetupScreen<C, G> {
    pub fn new(game_widget: Weak<RefCell<G>>) -> Self {
        let directory = Rc::new(RefCell::new(None));
        let players = 2;
        // Set the default theme index based on the DEFAULT_THEME
        let theme_index = crate::hardcoded_cards::AVAILABLE_THEMES
            .iter()
            .position(|&t| t == crate::hardcoded_cards::DEFAULT_THEME)
            .unwrap_or(0);
        Self {
            directory,
            players,
            theme_index,
            game_widget,
        }
    }

    pub fn generate_config(&self) -> Option<GameConfig<C>> {
        let directory = Rc::new(self.directory.borrow().clone()?);

        let mut players: Vec<(String, SimpleField<SimpleCard, C>)> = (0..self.players)
            .map(|i| {
                (
                    format!("{i}"),
                    SimpleField::new(Rc::clone(&directory))
                        .max_cards(4)
                        .selectable(true)
                        .max_card_size(vec2(100.0, 150.0)),
                )
            })
            .collect();
        let mut stack = SimpleField::new(Rc::clone(&directory))
            .kind(Stack)
            .max_card_size(vec2(100.0, 150.0));
        for i in 0..directory.T() {
            let card = SimpleCard::Open(i);
            stack.push(card);
            players[i % self.players].1.push(SimpleCard::Open(i));
        }
        Some(GameConfig { players, stack })
    }
}

impl ScreenWidget for GameSetupScreen<DirectoryCardType, Game<DirectoryCardType>> {
    fn update(&mut self, next_screen: Rc<RefCell<ScreenType>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Card Pack:");
                match self.directory.borrow().as_ref() {
                    None => ui.label("Deck not loaded"),
                    Some(dir) => ui.label(format!("Using {}", &dir.path)),
                }
            });

            ui.horizontal(|ui| {
                ui.label("Theme:");
                egui::ComboBox::new("theme_selector", "Theme")
                    .selected_text(crate::hardcoded_cards::AVAILABLE_THEMES[self.theme_index])
                    .show_ui(ui, |ui| {
                        for (i, theme) in
                            crate::hardcoded_cards::AVAILABLE_THEMES.iter().enumerate()
                        {
                            // Display a user-friendly name for the theme
                            let theme_name = match *theme {
                                "img_cards" => "Standard Cards",
                                "alt_cards" => "Alternative Cards",
                                _ => theme,
                            };

                            if ui
                                .selectable_label(self.theme_index == i, theme_name)
                                .clicked()
                            {
                                self.theme_index = i;
                                let theme_str =
                                    crate::hardcoded_cards::AVAILABLE_THEMES[self.theme_index];
                                crate::hardcoded_cards::set_deck_by_theme(
                                    &self.directory,
                                    theme_str,
                                );
                            }
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("# Players");
                let drag = egui::DragValue::new(&mut self.players);
                ui.add(drag);
                let dec = egui::Button::new("-").min_size(vec2(30.0, 0.0));
                if ui.add(dec).clicked() && self.players > 1 {
                    self.players = self.players.saturating_sub(1);
                }
                let inc = egui::Button::new("+").min_size(vec2(30.0, 0.0));
                if ui.add(inc).clicked() {
                    self.players = self.players.saturating_add(1);
                }
            });
            if ui.button("Start Game").clicked() {
                if let Some(game) = self.game_widget.upgrade() {
                    let config = self.generate_config();
                    if config.is_some() {
                        game.borrow_mut().game_config = config;
                        *next_screen.borrow_mut() = ScreenType::Game;
                    }
                }
            }
            if ui.button("Back").clicked() {
                *next_screen.borrow_mut() = ScreenType::Main;
            }
        });
    }
}

// Implementation for drag-and-drop version
use super::cards_test_dnd::CardsTestDND;

impl ScreenWidget for GameSetupScreen<DirectoryCardType, CardsTestDND> {
    fn update(&mut self, next_screen: Rc<RefCell<ScreenType>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Card Pack:");
                match self.directory.borrow().as_ref() {
                    None => ui.label("Deck not loaded"),
                    Some(dir) => ui.label(format!("Using {}", &dir.path)),
                }
            });

            ui.horizontal(|ui| {
                ui.label("Theme:");
                egui::ComboBox::new("theme_selector_dnd", "Theme")
                    .selected_text(crate::hardcoded_cards::AVAILABLE_THEMES[self.theme_index])
                    .show_ui(ui, |ui| {
                        for (i, theme) in
                            crate::hardcoded_cards::AVAILABLE_THEMES.iter().enumerate()
                        {
                            // Display a user-friendly name for the theme
                            let theme_name = match *theme {
                                "img_cards" => "Standard Cards",
                                "alt_cards" => "Alternative Cards",
                                _ => theme,
                            };

                            if ui
                                .selectable_label(self.theme_index == i, theme_name)
                                .clicked()
                            {
                                self.theme_index = i;
                                let theme_str =
                                    crate::hardcoded_cards::AVAILABLE_THEMES[self.theme_index];
                                crate::hardcoded_cards::set_deck_by_theme(
                                    &self.directory,
                                    theme_str,
                                );
                            }
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("# Players");
                let drag = egui::DragValue::new(&mut self.players);
                ui.add(drag);
                let dec = egui::Button::new("-").min_size(vec2(30.0, 0.0));
                if ui.add(dec).clicked() && self.players > 1 {
                    self.players = self.players.saturating_sub(1);
                }
                let inc = egui::Button::new("+").min_size(vec2(30.0, 0.0));
                if ui.add(inc).clicked() {
                    self.players = self.players.saturating_add(1);
                }
            });
            if ui.button("Start Game").clicked() {
                if let Some(game) = self.game_widget.upgrade() {
                    let config = self.generate_config();
                    if config.is_some() {
                        game.borrow_mut().game_config = config;
                        *next_screen.borrow_mut() = ScreenType::GameDnd;
                    }
                }
            }
            if ui.button("Back").clicked() {
                *next_screen.borrow_mut() = ScreenType::Main;
            }
        });
    }
}