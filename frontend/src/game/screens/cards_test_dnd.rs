use eframe::Frame;

use super::{DNDSelector, DirectoryCardType, GameConfig, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::game::field::{FieldWidget, SimpleField, SimpleFieldKind::Stack};
use crate::game::AppInterface;
use crate::game::card::SimpleCard;
use crate::sprintln;
use egui::vec2;
use std::rc::Rc;

pub struct CardsTestDND {
    pub(crate) game_config: Option<GameConfig<DirectoryCardType>>,
    drag: Option<DNDSelector>,
    drop: Option<DNDSelector>,
}
impl CardsTestDND {
    fn demo_config() -> GameConfig<DirectoryCardType> {
        // Create a demo deck and populate a stack and two player fields
        let card_config = crate::hardcoded_cards::create_deck(crate::hardcoded_cards::DEFAULT_THEME);
        let mut players: Vec<(String, SimpleField<SimpleCard, DirectoryCardType>)> = (0..2)
            .map(|i| {
                (
                    format!("Player {}", i + 1),
                    SimpleField::new(Rc::new(card_config.clone()))
                        .max_cards(8)
                        .selectable(true)
                        .max_card_size(vec2(100.0, 150.0)),
                )
            })
            .collect();
        let mut stack = SimpleField::new(Rc::new(card_config.clone()))
            .kind(Stack)
            .max_card_size(vec2(100.0, 150.0));
        for i in 0..card_config.T {
            let card = SimpleCard::Open(i);
            stack.push(card.clone());
            players[i % 2].1.push(SimpleCard::Open(i));
        }
        GameConfig { players, stack }
    }

    pub fn new() -> Self {
        let mut s = CardsTestDND {
            game_config: None,
            drag: None,
            drop: None,
        };
        // populate with demo data so the screen is not empty when opened from the menu
        s.set_config(Self::demo_config());
        s
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
    fn ui(&mut self, _app_interface: &mut AppInterface, ui: &mut egui::Ui, _frame: &mut Frame) {
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
    }
}

impl ScreenDef for CardsTestDND {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/game-dnd",
            display_name: "Game (DND)",
            icon: "🃏",
            description: "Drag-and-drop game screen",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        Box::new(Self::new())
    }
}
