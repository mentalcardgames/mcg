use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::Context;

use crate::sprintln;
use crate::game::field::FieldWidget;
use super::{ScreenWidget, ScreenType, GameConfig, DirectoryCardType, DNDSelector};

/// Drag and drop card test screen
pub struct CardsTestDND {
    pub(crate) game_config: Option<GameConfig<DirectoryCardType>>,
    drag: Option<DNDSelector>,
    drop: Option<DNDSelector>,
}

impl CardsTestDND {
    pub fn new() -> Self {
        CardsTestDND {
            game_config: None,
            drag: None,
            drop: None,
        }
    }
}

impl Default for CardsTestDND {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for CardsTestDND {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Exit").clicked() {
                *next_screen.borrow_mut() = ScreenType::Main.to_string();
            }
            if self.game_config.is_none() {
                return;
            }
            if let Some(cfg) = self.game_config.as_mut() {
                ui.label("Stack");
                let stack = &cfg.stack;
                if let Some(payload) = ui.add(stack.draw()).dnd_release_payload::<DNDSelector>() {
                    sprintln!("Received Payload in CardsTestDND over the Stack");
                    sprintln!("Payload: {payload:?}");
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
                let (name_0, field_0) = &cfg.players[0];
                ui.label(name_0);
                if let Some(payload) = ui.add(field_0.draw()).dnd_release_payload::<DNDSelector>() {
                    sprintln!("Received Payload in CardsTestDND over the first Field");
                    sprintln!("Payload: {payload:?}");
                    self.drop = Some(DNDSelector::Player(0, field_0.cards.len()))
                }
                match field_0.get_payload() {
                    (_, Some(idx)) => self.drop = Some(DNDSelector::Player(0, idx)),
                    (Some(idx), _) => {
                        if self.drag.is_none() {
                            self.drag = Some(DNDSelector::Player(0, idx))
                        }
                    }
                    (None, None) => {}
                }
                let (name_1, field_1) = &cfg.players[1];
                ui.label(name_1);
                if let Some(payload) = ui.add(field_1.draw()).dnd_release_payload::<DNDSelector>() {
                    sprintln!("Received Payload in CardsTestDND over the second Field");
                    sprintln!("Payload: {payload:?}");
                    self.drop = Some(DNDSelector::Player(1, field_1.cards.len()))
                }
                match field_1.get_payload() {
                    (_, Some(idx)) => self.drop = Some(DNDSelector::Player(1, idx)),
                    (Some(idx), _) => {
                        if self.drag.is_none() {
                            self.drag = Some(DNDSelector::Player(1, idx))
                        }
                    }
                    (None, None) => {}
                }
                if let (Some(source), Some(destination)) = (self.drag, self.drop) {
                    sprintln!("Drag: {:?}\t Drop: {:?}", self.drag, self.drop);
                    cfg.move_card::<crate::game::card::SimpleCard>(source, destination);
                    self.drag = None;
                    self.drop = None;
                } else {
                    sprintln!("Drag: {:?}\t Drop: {:?}", self.drag, self.drop);
                }
            }
        });
    }
}