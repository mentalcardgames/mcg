use eframe::Frame;
use egui::{Align, Layout, Rect, UiBuilder};

use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::game::card::{CardConfig, SimpleCard};
use crate::game::field::{FieldWidget, SimpleField};

#[derive(Default)]
pub struct Game<C: CardConfig> {
    pub game_state: Option<GameState<C>>,
    player0_idx: usize,
    player1_idx: usize,
    drag: Option<DNDSelector>,
    drop: Option<DNDSelector>,
}
impl<C: CardConfig> Game<C> {
    pub fn set_state(&mut self, config: GameState<C>) {
        self.game_state = Some(config);
    }
}

#[derive(Debug, Clone)]
pub struct GameState<C: CardConfig> {
    pub players: Vec<(String, SimpleField<SimpleCard, C>)>,
    pub stack: SimpleField<SimpleCard, C>,
}
impl<C: CardConfig> GameState<C> {
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
        let mut rect = ui.max_rect();
        let width = rect.width() / 3.0;
        rect.set_left(width);
        rect.set_right(2.0 * width);
        ui.scope_builder(
            UiBuilder::new()
                .layout(Layout::top_down_justified(Align::Min))
                .max_rect(rect),
            |ui| {
                ui.add_space(20.0);
                if self.game_state.is_none() {
                    return;
                }
                ui.add_space(5.0);
                ui.vertical_centered_justified(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("1. Player:");
                        egui::ComboBox::from_id_salt("Display Player 0").show_index(
                            ui,
                            &mut self.player0_idx,
                            self.game_state.as_mut().unwrap().players.len(),
                            |i| i.to_string(),
                        );
                        ui.add_space(
                            3.0 * width - 2.0 * ui.cursor().left() + ui.spacing().item_spacing.x,
                        );
                        ui.label("2. Player:");
                        egui::ComboBox::from_id_salt("Display Player 1").show_index(
                            ui,
                            &mut self.player1_idx,
                            self.game_state.as_mut().unwrap().players.len(),
                            |i| i.to_string(),
                        );
                    });
                });
                ui.add_space(5.0);
                let cfg = self.game_state.as_mut().unwrap();
                ui.add_space(5.0);
                ui.label("Stack");
                let stack = &cfg.stack;
                if let Some(_payload) = ui.add(stack.draw()).dnd_release_payload::<DNDSelector>() {
                    self.drop = Some(DNDSelector::Stack)
                }
                match stack.get_payload() {
                    (_, Some(_idx)) => self.drop = Some(DNDSelector::Stack),
                    (Some(_idx), _) => {
                        if self.drag.is_none() {
                            self.drag = Some(DNDSelector::Stack)
                        }
                    }
                    (None, None) => {}
                }
                let (name_0, field_0) = &cfg.players[self.player0_idx];
                ui.add_space(5.0);
                ui.label(name_0);
                if let Some(_payload) = ui.add(field_0.draw()).dnd_release_payload::<DNDSelector>()
                {
                    self.drop = Some(DNDSelector::Player(self.player0_idx, field_0.cards.len()))
                }
                match field_0.get_payload() {
                    (_, Some(idx)) => self.drop = Some(DNDSelector::Player(self.player0_idx, idx)),
                    (Some(idx), _) => {
                        if self.drag.is_none() {
                            self.drag = Some(DNDSelector::Player(self.player0_idx, idx))
                        }
                    }
                    (None, None) => {}
                }
                let (name_1, field_1) = &cfg.players[self.player1_idx];
                ui.add_space(5.0);
                ui.label(name_1);
                if let Some(_payload) = ui.add(field_1.draw()).dnd_release_payload::<DNDSelector>()
                {
                    self.drop = Some(DNDSelector::Player(self.player1_idx, field_1.cards.len()))
                }
                match field_1.get_payload() {
                    (_, Some(idx)) => self.drop = Some(DNDSelector::Player(self.player1_idx, idx)),
                    (Some(idx), _) => {
                        if self.drag.is_none() {
                            self.drag = Some(DNDSelector::Player(self.player1_idx, idx))
                        }
                    }
                    (None, None) => {}
                }
                if let (Some(source), Some(destination)) = (self.drag, self.drop) {
                    cfg.move_card(source, destination);
                    self.drag = None;
                    self.drop = None;
                }
                if ui.input(|i| i.pointer.primary_down()) {
                    if let Some(drag) = self.drag {
                        let (img, size) = match drag {
                            DNDSelector::Player(field_idx, card_idx) => {
                                let card = &cfg.players[field_idx].1.cards[card_idx];
                                (
                                    cfg.players[field_idx].1.card_config.img(card),
                                    cfg.players[field_idx].1.get_card_size(),
                                )
                            }
                            DNDSelector::Stack => {
                                let card = cfg.stack.cards.last().unwrap();
                                (cfg.stack.card_config.img(card), cfg.stack.get_card_size())
                            }
                            _ => {
                                panic!("This should not happen")
                            }
                        };
                        if let Some(pointer_pos) = ui.input(|i| i.pointer.latest_pos()) {
                            img.paint_at(ui, Rect::from_min_size(pointer_pos, size));
                        }
                    }
                } else if self.drag.is_some() {
                    self.drag = None;
                }
            },
        );
    }
}

crate::impl_screen_def!(
    Game<DirectoryCardType>,
    "/game",
    "Game",
    "🃏",
    "Active game screen",
    false
);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DNDSelector {
    Player(usize, usize),
    Stack,
    Index(usize),
}
