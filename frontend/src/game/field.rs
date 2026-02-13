use crate::game::card::SimpleCard::Open;
use crate::game::card::{CardConfig, CardEncoding};
use crate::game::screens::DNDSelector;
use crate::sprintln;
use eframe::emath::{vec2, Rect};
use egui::{frame, Color32, Sense, Vec2};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::rc::Rc;

pub trait FieldWidget {
    fn draw(&self) -> impl egui::Widget;
}
impl<E: CardEncoding, C: CardConfig> FieldWidget for SimpleField<E, C> {
    fn draw(&self) -> impl egui::Widget {
        move |ui: &mut egui::Ui| -> egui::Response {
            frame::Frame::new()
                .inner_margin(egui::Margin::same(self.margin))
                .stroke(egui::Stroke::new(2.0, Color32::DEBUG_COLOR))
                .fill(Color32::DARK_GREEN)
                .corner_radius(egui::CornerRadius::same(self.margin.unsigned_abs()))
                .show(ui, |ui| match self.kind {
                    SimpleFieldKind::Stack => self.draw_stack(ui),
                    SimpleFieldKind::Horizontal => self.draw_horizontal(ui),
                })
                .response
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SimpleFieldKind {
    Stack,
    Horizontal,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct SimpleField<E: CardEncoding, C: CardConfig> {
    pub(crate) card_config: Rc<C>,
    pub(crate) cards: Vec<E>,
    pub kind: SimpleFieldKind,
    pub margin: i8,
    pub max_cards: usize,
    pub selectable: bool,
    pub draggable: bool,
    max_card_size: Option<Vec2>,
    pub(crate) drag_payload: RefCell<Option<usize>>,
    pub(crate) drop_payload: RefCell<Option<usize>>,
}
impl<E: CardEncoding, C: CardConfig> SimpleField<E, C> {
    pub fn new(card_config: Rc<C>) -> Self {
        Self {
            cards: vec![],
            card_config,
            kind: SimpleFieldKind::Horizontal,
            margin: 4,
            max_cards: 5,
            selectable: true,
            draggable: true,
            max_card_size: None,
            drag_payload: RefCell::new(None),
            drop_payload: RefCell::new(None),
        }
    }
    pub fn from_collection(card_config: Rc<C>, cards: impl IntoIterator<Item = E>) -> Self {
        SimpleField {
            cards: cards.into_iter().collect(),
            ..SimpleField::new(card_config)
        }
    }
    pub fn max_cards(self, max_cards: usize) -> Self {
        SimpleField { max_cards, ..self }
    }
    pub fn kind(self, kind: SimpleFieldKind) -> Self {
        SimpleField { kind, ..self }
    }
    pub fn margin(self, margin: i8) -> Self {
        SimpleField { margin, ..self }
    }
    pub fn selectable(self, selectable: bool) -> Self {
        SimpleField { selectable, ..self }
    }
    pub fn draggable(self, draggable: bool) -> Self {
        SimpleField { draggable, ..self }
    }
    pub fn max_card_size(self, max_card_size: Vec2) -> Self {
        let card_size = if let Some(card) = self.cards.first() {
            self.card_config
                .img(card)
                .calc_size(max_card_size, Some(self.card_config.natural_size()))
        } else {
            self.card_config
                .img(&Open(0))
                .calc_size(max_card_size, Some(self.card_config.natural_size()))
        };
        SimpleField {
            max_card_size: Some(card_size),
            ..self
        }
    }
}
impl<E: CardEncoding, C: CardConfig> SimpleField<E, C> {
    pub fn get_card_size(&self) -> Vec2 {
        self.max_card_size
            .unwrap_or_else(|| self.card_config.natural_size())
    }
    pub fn get_cards(&self) -> &Vec<E> {
        &self.cards
    }
    pub fn is_stack(&self) -> bool {
        matches!(self.kind, SimpleFieldKind::Stack)
    }
    pub fn is_horizontal(&self) -> bool {
        matches!(self.kind, SimpleFieldKind::Horizontal)
    }
    pub fn get_payload(&self) -> (Option<usize>, Option<usize>) {
        let drag = self.drag_payload.replace(None);
        let drop = self.drop_payload.replace(None);
        (drag, drop)
    }
}
impl<E: CardEncoding, C: CardConfig> SimpleField<E, C> {
    fn set_drag_payload(&self, response: &egui::Response, payload: usize) {
        // TODO Make the payload be a unique identifier
        response.dnd_set_drag_payload(DNDSelector::Index(payload));
        self.drag_payload.replace(Some(payload));
    }
    fn set_drop_payload(&self, response: &egui::Response, payload: usize) {
        // TODO Make the payload be a unique identifier
        if response.dnd_release_payload::<DNDSelector>().is_some() {
            self.drop_payload.replace(Some(payload));
        }
    }
    fn horizontal_card_selection(&self, ui: &egui::Ui) -> Option<usize> {
        let pointer_pos = ui.input(|state| state.pointer.latest_pos());
        let rect = ui.min_rect();
        if pointer_pos.is_some() && rect.contains(pointer_pos.unwrap()) {
            let max = if self.cards.len() > self.max_cards {
                rect.right() - rect.left()
            } else {
                self.cards.len() as f32 * (self.get_card_size().x + self.margin as f32)
                    - self.margin as f32
            };
            Some((self.cards.len() as f32 * (pointer_pos.unwrap().x - rect.left()) / max) as usize)
        } else {
            None
        }
    }
    fn horizontal_drag_size(&self) -> Vec2 {
        let mut size = self.get_card_size();
        size.x = self.card_pos(1).x - self.card_pos(0).x;
        size
    }
    fn content_size(&self) -> Vec2 {
        match self.kind {
            SimpleFieldKind::Stack => self
                .get_card_size()
                .add(vec2(self.max_cards as f32, self.max_cards as f32)),
            SimpleFieldKind::Horizontal => self.get_card_size().add(vec2(
                (self.max_cards as f32 - 1.0) * (self.get_card_size().x + self.margin as f32),
                self.margin as f32,
            )),
        }
    }
    fn card_pos(&self, idx: usize) -> Vec2 {
        match self.kind {
            SimpleFieldKind::Stack => {
                let x = if idx <= self.max_cards {
                    idx as f32
                } else {
                    self.max_cards as f32
                };
                Vec2::new(x, -x)
            }
            SimpleFieldKind::Horizontal => {
                let cards = self.cards.len();
                let x = if cards <= self.max_cards {
                    (self.get_card_size().x + self.margin as f32) * (idx as f32)
                } else {
                    (self.get_card_size().x + self.margin as f32)
                        * (idx as f32)
                        * ((self.max_cards - 1) as f32)
                        / ((cards - 1) as f32)
                };
                Vec2::new(x, 0.0)
            }
        }
    }
    // Utility methods used by game logic
    pub fn push(&mut self, card: E) {
        self.cards.push(card);
    }
    pub fn remove(&mut self, idx: usize) -> E {
        self.cards.remove(idx)
    }
    pub fn pop(&mut self) -> Option<E> {
        self.cards.pop()
    }
    pub fn insert(&mut self, idx: usize, card: E) {
        if idx >= self.cards.len() {
            self.cards.push(card);
        } else {
            self.cards.insert(idx, card);
        }
    }
    fn draw_stack(&self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_min_size(self.content_size());
        let origin = ui.cursor().left_top().add(vec2(0.0, self.max_cards as f32));
        for (idx, card) in self.cards.iter().enumerate() {
            self.card_config.img(card).paint_at(
                ui,
                Rect::from_min_size(origin.add(self.card_pos(idx)), self.get_card_size()),
            );
        }
        if self.draggable && !self.cards.is_empty() {
            ui.scope_builder(
                egui::UiBuilder::new()
                    .sense(Sense::click_and_drag())
                    .max_rect(Rect::from_min_size(
                        origin.add(self.card_pos(self.cards.len() - 1)),
                        self.get_card_size(),
                    )),
                |ui| {
                    ui.set_min_size(self.get_card_size());
                    if ui.response().drag_started() {
                        self.set_drag_payload(&ui.response(), self.cards.len() - 1);
                    }
                    self.set_drop_payload(&ui.response(), self.cards.len());
                    ui.response().context_menu(|ui| {
                        if ui.button("Show inner").clicked() {
                            sprintln!(
                                "Imagine the hided information of card {idx} here",
                                idx = self.cards.len() - 1
                            );
                            if ui.should_close() {
                                ui.close();
                            }
                        }
                    });
                },
            );
            self.set_drop_payload(&ui.response(), self.cards.len());
        }
        ui.response()
    }
    fn draw_horizontal(&self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_min_size(self.content_size());
        let origin = ui.cursor().left_top().add(vec2(0.0, self.margin as f32));
        // TODO show card on mouse hover if obstructed
        let _selection: Option<usize> = self.horizontal_card_selection(ui);
        for (idx, card) in self.cards.iter().enumerate() {
            self.card_config.img(card).paint_at(
                ui,
                Rect::from_min_size(origin.add(self.card_pos(idx)), self.get_card_size()),
            );
            if self.draggable {
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .sense(Sense::click_and_drag())
                        .max_rect(Rect::from_min_size(
                            origin.add(self.card_pos(idx)),
                            self.horizontal_drag_size(),
                        )),
                    |ui| {
                        ui.set_min_size(self.horizontal_drag_size());
                        if ui.response().drag_started() {
                            self.set_drag_payload(&ui.response(), idx);
                        }
                        self.set_drop_payload(&ui.response(), idx)
                    },
                );
            }
        }
        if self.draggable {
            let last_drag_rect_min = origin.add(self.card_pos(self.cards.len()));
            let mut last_drag_rect_size = self.get_card_size();
            last_drag_rect_size.x -= self.horizontal_drag_size().x;
            ui.scope_builder(
                egui::UiBuilder::new()
                    .sense(Sense::click_and_drag())
                    .max_rect(Rect::from_min_size(last_drag_rect_min, last_drag_rect_size)),
                |ui| {
                    ui.set_min_size(last_drag_rect_size);
                    if ui.response().drag_started() {
                        self.set_drag_payload(&ui.response(), self.cards.len() - 1);
                    }
                    self.set_drop_payload(&ui.response(), self.cards.len())
                },
            );
            self.set_drop_payload(&ui.response(), self.cards.len());
        }
        ui.response()
    }
}

impl<E: CardEncoding + Debug, C: CardConfig + Debug> Debug for SimpleField<E, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleField")
            .field("kind", &self.kind)
            .field("card_config", &self.card_config)
            .field("cards", &self.cards)
            .finish()
    }
}
