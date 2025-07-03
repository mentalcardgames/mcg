use std::cell::RefCell;
use std::rc::Rc;

use eframe::Frame;
use egui::Context;

use crate::sprintln;
use crate::game::field::FieldWidget;
use super::{ScreenWidget, ScreenType, GameConfig, DirectoryCardType, DNDSelector};
use crate::game::AppInterface;

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

    pub fn set_config(&mut self, config: GameConfig<DirectoryCardType>) {
        self.game_config = Some(config);
        self.drag = None;
        self.drop = None;
    }
}

impl Default for CardsTestDND {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for CardsTestDND {
    fn update(&mut self, app_interface: &mut AppInterface, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Exit").clicked() {
                app_interface.queue_event(crate::game::AppEvent::ChangeScreen(ScreenType::Main));
            }
            if self.game_config.is_none() {
                return;
            }
            if let Some(cfg) = self.game_config.as_mut() {
                ui.label("Stack");
                let stack = &cfg.stack;
                if let Some(_payload) = ui.add(stack.draw()).dnd_release_payload::<DNDSelector>() {
                    sprintln!("Received Payload in CardsTestDND over the Stack");
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
                if let Some(_payload) = ui.add(field_0.draw()).dnd_release_payload::<DNDSelector>() {
                    sprintln!("Received Payload in CardsTestDND over the first Field");
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
                if let Some(_payload) = ui.add(field_1.draw()).dnd_release_payload::<DNDSelector>() {
                    sprintln!("Received Payload in CardsTestDND over the second Field");
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
                    cfg.move_card(source, destination);
                    self.drag = None;
                    self.drop = None;
                } else {
                    sprintln!("Drag: {:?}\t Drop: {:?}", self.drag, self.drop);
                }
            }
        });
    }
}