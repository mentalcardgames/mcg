use crate::game::card::{CardConfig, CardEncoding, DirectoryCardType, SimpleCard};
use crate::game::field::{FieldWidget, SimpleField, SimpleFieldKind::Stack};
use crate::sprintln;
use eframe::Frame;
use egui::{vec2, Color32, Context, Id};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub trait ScreenWidget {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, frame: &mut Frame);
}
impl ScreenWidget for MainMenu {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Start").clicked() {
                sprintln!("setup started");
                *next_screen.borrow_mut() = String::from("game_setup");
            };
            if ui.button("Drag & Drop Game").clicked() {
                sprintln!("game_dnd opened");
                *next_screen.borrow_mut() = String::from("game_dnd_setup");
            };
            if ui.button("Settings").clicked() {
                sprintln!("settings opened");
                *next_screen.borrow_mut() = String::from("settings");
            };
            if ui.button("Drag & Drop Test").clicked() {
                sprintln!("dnd_test opened");
                *next_screen.borrow_mut() = String::from("dnd_test");
            };
            if ui.button("Print Screen").clicked() {
                sprintln!("{}", next_screen.borrow());
            };
        });
    }
}

impl ScreenWidget for GameSetupScreen {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Card Pack:");
                match self.directory.borrow().as_ref() {
                    None => ui.label("Standard deck not loaded"),
                    Some(dir) => ui.label(format!("Using {}", &dir.path)),
                }
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
                        *next_screen.borrow_mut() = String::from("game");
                    }
                }
            }
            if ui.button("Back").clicked() {
                *next_screen.borrow_mut() = String::from("main");
            }
        });
    }
}
impl ScreenWidget for Game<DirectoryCardType> {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Exit").clicked() {
                sprintln!("back to main menu");
                *next_screen.borrow_mut() = String::from("main");
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
impl ScreenWidget for DNDTest {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Exit").clicked() {
                sprintln!("back to main menu");
                *next_screen.borrow_mut() = String::from("main");
            }
            ui.label("This is a simple example of drag-and-drop in egui.");
            ui.label("Drag items between columns.");

            // If there is a drop, store the location of the item being dragged, and the destination for the drop.
            let mut from = None;
            let mut to = None;

            ui.columns(self.columns.len(), |uis| {
                for (col_idx, column) in self.columns.clone().into_iter().enumerate() {
                    let ui = &mut uis[col_idx];

                    let frame = egui::Frame::default().inner_margin(4.0);

                    let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                        ui.set_min_size(vec2(64.0, 100.0));
                        for (row_idx, item) in column.iter().enumerate() {
                            let item_id = Id::new(("my_drag_and_drop_demo", col_idx, row_idx));
                            let item_location = Location {
                                col: col_idx,
                                row: row_idx,
                            };
                            let response = ui
                                .dnd_drag_source(item_id, item_location, |ui| {
                                    ui.label(item);
                                })
                                .response;

                            // Detect drops onto this item:
                            if let (Some(pointer), Some(hovered_payload)) = (
                                ui.input(|i| i.pointer.interact_pos()),
                                response.dnd_hover_payload::<Location>(),
                            ) {
                                let rect = response.rect;

                                // Preview insertion:
                                let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                                let insert_row_idx = if *hovered_payload == item_location {
                                    // We are dragged onto ourselves
                                    ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                    row_idx
                                } else if pointer.y < rect.center().y {
                                    // Above us
                                    ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                    row_idx
                                } else {
                                    // Below us
                                    ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                    row_idx + 1
                                };

                                if let Some(dragged_payload) = response.dnd_release_payload() {
                                    // The user dropped onto this item.
                                    from = Some(dragged_payload);
                                    to = Some(Location {
                                        col: col_idx,
                                        row: insert_row_idx,
                                    });
                                }
                            }
                        }
                    });

                    if let Some(dragged_payload) = dropped_payload {
                        // The user dropped onto the column, but not on any one item.
                        from = Some(dragged_payload);
                        to = Some(Location {
                            col: col_idx,
                            row: usize::MAX, // Inset last
                        });
                    }
                }
            });

            if let (Some(from), Some(mut to)) = (from, to) {
                if from.col == to.col {
                    // Dragging within the same column.
                    // Adjust row index if we are re-ordering:
                    to.row -= (from.row < to.row) as usize;
                }

                let item = self.columns[from.col].remove(from.row);

                let column = &mut self.columns[to.col];
                to.row = to.row.min(column.len());
                column.insert(to.row, item);
            }
        });
    }
}
impl ScreenWidget for GameSetupScreen<DirectoryCardType, CardsTestDND> {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Card Pack:");
                match self.directory.borrow().as_ref() {
                    None => ui.label("Alternative deck not loaded"),
                    Some(dir) => ui.label(format!("Using {}", &dir.path)),
                }
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
                        *next_screen.borrow_mut() = String::from("game_dnd");
                    }
                }
            }
            if ui.button("Back").clicked() {
                *next_screen.borrow_mut() = String::from("main");
            }
        });
    }
}
impl ScreenWidget for CardsTestDND {
    fn update(&mut self, next_screen: Rc<RefCell<String>>, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Exit").clicked() {
                *next_screen.borrow_mut() = String::from("main");
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
                    (Some(_idx), _) => if self.drag.is_none() { self.drag = Some(DNDSelector::Stack) },
                    (None, None) => {},
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
                    (Some(idx), _) => if self.drag.is_none() { self.drag = Some(DNDSelector::Player(0, idx)) },
                    (None, None) => {},
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
                    (Some(idx), _) => if self.drag.is_none() { self.drag = Some(DNDSelector::Player(1, idx)) },
                    (None, None) => {},
                }
                if let (Some(source), Some(destination)) = (self.drag, self.drop) {
                    sprintln!("Drag: {:?}\t Drop: {:?}", self.drag, self.drop);
                    cfg.move_card::<SimpleCard>(source, destination);
                    self.drag = None;
                    self.drop = None;
                } else {
                    sprintln!("Drag: {:?}\t Drop: {:?}", self.drag, self.drop);
                }
            }
        });
    }
}

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

pub struct GameSetupScreen<C: CardConfig = DirectoryCardType, G = Game<C>> {
    pub directory: Rc<RefCell<Option<C>>>,
    players: usize,
    pub(crate) game_widget: Weak<RefCell<G>>,
}
impl<C: CardConfig + Clone, G> GameSetupScreen<C, G> {
    pub fn new(game_widget: Weak<RefCell<G>>) -> Self {
        let directory = Rc::new(RefCell::new(None));
        let players = 2;
        Self {
            directory,
            players,
            game_widget,
        }
    }
    fn generate_config(&self) -> Option<GameConfig<C>> {
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
        let mut stack = SimpleField::new(Rc::clone(&directory)).kind(Stack).max_card_size(vec2(100.0, 150.0));
        for i in 0..directory.T() {
            let card = SimpleCard::Open(i);
            stack.push(card);
            players[i % self.players].1.push(SimpleCard::Open(i));
        }
        Some(GameConfig { players, stack })
    }
}

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

pub struct GameConfig<C: CardConfig> {
    players: Vec<(String, SimpleField<SimpleCard, C>)>,
    stack: SimpleField<SimpleCard, C>,
}
impl<C: CardConfig> GameConfig<C> {
    pub fn move_card<E: CardEncoding>(&mut self, src: DNDSelector, dst: DNDSelector) {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    col: usize,
    row: usize,
}
pub struct DNDTest {
    columns: Vec<Vec<String>>,
}
impl DNDTest {
    pub fn new() -> Self {
        let columns = vec![
            vec!["Item A", "Item B", "Item C", "Item D"],
            vec!["Item E", "Item F", "Item G"],
            vec!["Item H", "Item I", "Item J", "Item K"],
        ]
            .into_iter()
            .map(|v| v.into_iter().map(ToString::to_string).collect())
            .collect();
        DNDTest { columns }
    }
}
impl Default for DNDTest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DNDSelector {
    Player(usize, usize),
    Stack,
    Index(usize),
}
pub struct CardsTestDND {
    pub(crate) game_config: Option<GameConfig<DirectoryCardType>>,
    drag: Option<DNDSelector>,
    drop: Option<DNDSelector>,
}
impl CardsTestDND {
    pub fn new() -> Self {
        CardsTestDND { game_config: None, drag: None, drop: None }
    }
}
impl Default for CardsTestDND {
    fn default() -> Self {
        Self::new()
    }
}
