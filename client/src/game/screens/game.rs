use eframe::Frame;

use super::{AppInterface, ScreenWidget};
use crate::game::card::{CardConfig, SimpleCard};
use crate::game::field::{FieldWidget, SimpleField};
// use crate::sprintln;

pub struct Game<C: CardConfig> {
    pub game_config: Option<GameConfig<C>>,
    pub image_idx: usize,
    pub player_idx: usize,
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

pub type DirectoryCardType = crate::game::card::DirectoryCardType;

impl ScreenWidget for Game<DirectoryCardType> {
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
        if let Some(config) = &self.game_config {
            ui.horizontal(|ui| {
                ui.label("Image Directory:");
                ui.label(format!("{:?}", &config.stack.card_config.path));
            });
            let images = config.stack.card_config.img_names.clone();
            let images_len = config.stack.card_config.img_names.len();
            let player_names: Vec<String> = config
                .players
                .iter()
                .map(|(name, _)| name.clone())
                .collect();
            let players_len = config.players.len();
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
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DNDSelector {
    Player(usize, usize),
    Stack,
    Index(usize),
}
