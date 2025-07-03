use eframe::Frame;
use egui::Context;

use crate::sprintln;
use crate::game::card::{CardConfig, SimpleCard};
use crate::game::field::{SimpleField, FieldWidget};
use super::{ScreenWidget, ScreenType, AppInterface};

/// Main game screen for playing cards
pub struct Game<C: CardConfig> {
    game_config: Option<GameConfig<C>>,
    image_idx: usize,
    player_idx: usize,
}

impl<C: CardConfig> Game<C> {
    pub fn new() -> Self {
        Self {
            game_config: None,
            image_idx: 0,
            player_idx: 0,
        }
    }

    pub fn set_config(&mut self, config: GameConfig<C>) {
        self.game_config = Some(config);
    }
}

impl<C: CardConfig> Default for Game<C> {
    fn default() -> Self {
        Self::new()
    }
}

/// Game configuration including players and card stack
#[derive(Debug, Clone)]
pub struct GameConfig<C: CardConfig> {
    pub players: Vec<(String, SimpleField<SimpleCard, C>)>,
    pub stack: SimpleField<SimpleCard, C>,
}

impl<C: CardConfig> GameConfig<C> {
    pub fn move_card(&mut self, src: DNDSelector, dst: DNDSelector) {
        if src == dst {
            return;
        }
        let card = match src {
            DNDSelector::Player(p_idx, c_idx) => {
                if p_idx < self.players.len() {
                    self.players[p_idx].1.remove(c_idx)
                } else {
                    return;
                }
            }
            DNDSelector::Stack => {
                if let Some(card) = self.stack.pop() {
                    card
                } else {
                    return;
                }
            }
            DNDSelector::Index(_) => return,
        };
        
        match dst {
            DNDSelector::Player(p_idx, c_idx) => {
                if p_idx < self.players.len() {
                    self.players[p_idx].1.insert(c_idx, card);
                }
            }
            DNDSelector::Stack => {
                self.stack.push(card);
            }
            DNDSelector::Index(_) => {}
        };
    }
}

/// Card type that uses images from a directory
pub type DirectoryCardType = crate::game::card::DirectoryCardType;

impl ScreenWidget for Game<DirectoryCardType> {
    fn update(&mut self, app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Exit").clicked() {
                sprintln!("back to main menu");
                app_interface.queue_event(crate::game::AppEvent::ChangeScreen(ScreenType::Main));
            }

            if let Some(config) = &self.game_config {
                ui.horizontal(|ui| {
                    ui.label("Image Directory:");
                    ui.label(format!("{:?}", &config.stack.card_config.path));
                });

                // Extract the data we need before the mutable borrow
                let images = config.stack.card_config.img_names.clone();
                let images_len = config.stack.card_config.img_names.len();
                let player_names: Vec<String> = config
                    .players
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect();
                let players_len = config.players.len();
                
                // Now drop the config reference to avoid borrow conflicts
                let _ = config;
                
                ui.horizontal(|ui| {
                    ui.label("Images:");
                    egui::ComboBox::from_id_salt("Image Name preview").show_index(
                        ui,
                        &mut self.image_idx,
                        images_len,
                        |i| &images[i],
                    );
                });
                    
                ui.horizontal(|ui| {
                    ui.label("Player:");
                    egui::ComboBox::from_id_salt("Display Player Fields").show_index(
                        ui,
                        &mut self.player_idx,
                        players_len,
                        |i| &player_names[i],
                    );
                });

                // Get a fresh reference to config for the drawing
                if let Some(config) = &self.game_config {
                    ui.horizontal(|ui| {
                        ui.add(config.stack.draw());
                        if self.player_idx < config.players.len() {
                            ui.add(config.players[self.player_idx].1.draw());
                        }
                    });
                }
            } else {
                ui.label("No game configuration loaded");
            }
        });
    }
}

/// Selector for drag and drop operations
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DNDSelector {
    Player(usize, usize),
    Stack,
    Index(usize),
}