use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::Context;

use crate::sprintln;
use crate::game::card::{CardConfig, SimpleCard};
use crate::game::field::{SimpleField, FieldWidget};
use super::{ScreenWidget, ScreenType};

/// Main game screen for playing cards
pub struct Game<C: CardConfig> {
    pub(crate) game_config: Option<GameConfig<C>>,
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
}

impl<C: CardConfig> Default for Game<C> {
    fn default() -> Self {
        Self::new()
    }
}

/// Game configuration including players and card stack
pub struct GameConfig<C: CardConfig> {
    pub players: Vec<(String, SimpleField<SimpleCard, C>)>,
    pub stack: SimpleField<SimpleCard, C>,
}

impl<C: CardConfig> GameConfig<C> {
    pub fn move_card<E: crate::game::card::CardEncoding>(&mut self, src: DNDSelector, dst: DNDSelector) {
        if src == dst {
            return;
        }
        let card = match src {
            DNDSelector::Player(p_idx, c_idx) => self.players[p_idx].1.remove(c_idx),
            DNDSelector::Stack => self.stack.cards.pop().unwrap(),
            DNDSelector::Index(_) => return,
        };
        match dst {
            DNDSelector::Player(p_idx, c_idx) => self.players[p_idx].1.insert(c_idx, card),
            DNDSelector::Stack => self.stack.cards.push(card),
            DNDSelector::Index(_) => return,
        };
    }
}

/// Card type that uses images from a directory
pub type DirectoryCardType = crate::game::card::DirectoryCardType;

impl ScreenWidget for Game<DirectoryCardType> {
    fn update(&mut self, next_screen: Rc<RefCell<ScreenType>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Exit").clicked() {
                sprintln!("back to main menu");
                *next_screen.borrow_mut() = ScreenType::Main;
            }
            ui.horizontal(|ui| {
                ui.label("Image Directory:");
                ui.label(format!(
                    "{:?}",
                    &self.game_config.as_ref().unwrap().stack.card_config.path
                ));
            });
            let images = self
                .game_config
                .as_ref()
                .unwrap()
                .stack
                .card_config
                .img_names
                .clone();
            ui.horizontal(|ui| {
                ui.label("Images:");
                egui::ComboBox::from_id_salt("Image Name preview").show_index(
                    ui,
                    &mut self.image_idx,
                    self.game_config
                        .as_ref()
                        .unwrap()
                        .stack
                        .card_config
                        .img_names
                        .len(),
                    |i| &images[i],
                );
            });
            let player_names: Vec<String> = self
                .game_config
                .as_ref()
                .unwrap()
                .players
                .iter()
                .clone()
                .map(|e| e.0.clone())
                .collect();
            ui.horizontal(|ui| {
                ui.label("Player:");
                egui::ComboBox::from_id_salt("Display Player Fields").show_index(
                    ui,
                    &mut self.player_idx,
                    self.game_config.as_ref().unwrap().players.len(),
                    |i| &player_names[i],
                );
            });
            ui.horizontal(|ui| {
                ui.add(self.game_config.as_ref().unwrap().stack.draw());
                ui.add(
                    self.game_config.as_ref().unwrap().players[self.player_idx]
                        .1
                        .draw(),
                );
            });
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