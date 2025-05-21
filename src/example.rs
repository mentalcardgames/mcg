use egui;
use egui::frame;
use rand::Rng;
use std::fmt;
use std::ops::Add;

// TODO move this module into mcg_visual::card or make it a module inside mcg_visual::card
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct ConventionalCard {
    pub suit: Suit,
    pub rank: Rank,
    pub pos: egui::Pos2,
}
impl ConventionalCard {
    pub fn _iter() -> ConventionalCardIter {
        Default::default()
    }
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let rank: Rank = rng.gen_range(0..Rank::len()).into();
        let suit: Suit = rng.gen_range(0..Suit::len()).into();
        let x = rng.gen_range(0..1000) as f32;
        let y = rng.gen_range(0..1000) as f32;
        let pos = egui::Pos2::new(x, y);
        ConventionalCard { suit, rank, pos }
    }
    #[allow(dead_code)]
    fn img_path(&self) -> String {
        format!(
            "http://127.0.0.1:8080/media/img_cards/{}_{}.png",
            self.rank as usize + 1,
            self.suit.to_string().to_lowercase()
        )
    }
}
#[derive(Default)]
pub struct ConventionalCardIter {
    suit: SuitIter,
    rank: RankIter,
}
impl Iterator for ConventionalCardIter {
    type Item = ConventionalCard;

    fn next(&mut self) -> Option<Self::Item> {
        let rank = self.rank.value;
        let suit = self.suit.value;
        if rank.is_none() {
            if suit.is_none() {
                return None;
            } else {
                self.rank = Default::default();
                self.suit.next();
            }
        }
        Some(ConventionalCard {
            rank: self.rank.next()?,
            suit: self.suit.value?,
            pos: Default::default(),
        })
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Suit {
    #[default]
    Heart,
    Diamond,
    Club,
    Spade,
}
impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Suit {
    #[allow(unused)]
    pub fn iter() -> SuitIter {
        Default::default()
    }
    pub const fn len() -> usize {
        4
    }
}
impl From<usize> for Suit {
    fn from(index: usize) -> Self {
        match index {
            0 => Suit::Heart,
            1 => Suit::Diamond,
            2 => Suit::Club,
            3 => Suit::Spade,
            _ => {
                panic!("Invalid index: {}", index)
            }
        }
    }
}
pub struct SuitIter {
    value: Option<Suit>,
}
impl Default for SuitIter {
    fn default() -> Self {
        Self {
            value: Some(Default::default()),
        }
    }
}
impl Iterator for SuitIter {
    type Item = Suit;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.value;
        match self.value? {
            Suit::Heart => self.value = Some(Suit::Diamond),
            Suit::Diamond => self.value = Some(Suit::Club),
            Suit::Club => self.value = Some(Suit::Spade),
            Suit::Spade => self.value = None,
        };
        current
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Rank {
    #[default]
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}
impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Rank {
    #[allow(unused)]
    pub fn iter() -> RankIter {
        Default::default()
    }

    pub const fn len() -> usize {
        13
    }
}
impl From<usize> for Rank {
    fn from(index: usize) -> Self {
        match index {
            0 => Rank::Ace,
            1 => Rank::Two,
            2 => Rank::Three,
            3 => Rank::Four,
            4 => Rank::Five,
            5 => Rank::Six,
            6 => Rank::Seven,
            7 => Rank::Eight,
            8 => Rank::Nine,
            9 => Rank::Ten,
            10 => Rank::Jack,
            11 => Rank::Queen,
            12 => Rank::King,
            _ => {
                panic!("Invalid index: {}", index)
            }
        }
    }
}
pub struct RankIter {
    value: Option<Rank>,
}
impl Default for RankIter {
    fn default() -> Self {
        Self {
            value: Some(Default::default()),
        }
    }
}
impl Iterator for RankIter {
    type Item = Rank;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.value;
        match self.value? {
            Rank::Ace => self.value = Some(Rank::Two),
            Rank::Two => self.value = Some(Rank::Three),
            Rank::Three => self.value = Some(Rank::Four),
            Rank::Four => self.value = Some(Rank::Five),
            Rank::Five => self.value = Some(Rank::Six),
            Rank::Six => self.value = Some(Rank::Seven),
            Rank::Seven => self.value = Some(Rank::Eight),
            Rank::Eight => self.value = Some(Rank::Nine),
            Rank::Nine => self.value = Some(Rank::Ten),
            Rank::Ten => self.value = Some(Rank::Jack),
            Rank::Jack => self.value = Some(Rank::Queen),
            Rank::Queen => self.value = Some(Rank::King),
            Rank::King => self.value = None,
        };
        current
    }
}
// TODO check if this version of Stack can be safely removed
pub struct Stack {
    pub cards: Vec<()>,
    pub pos: egui::Pos2,
    size: egui::Vec2,
    pub inner_margin: i8,
    max_cards: usize,
}
impl Default for Stack {
    fn default() -> Self {
        let mut x = Self {
            cards: vec![],
            pos: egui::pos2(314.15, 271.828),
            size: Default::default(),
            inner_margin: 5,
            max_cards: 0,
        };
        x.max_cards(5);
        x
    }
}
impl Stack {
    #[allow(dead_code)]
    fn card_pos(&self, idx: usize) -> egui::Vec2 {
        let x = if idx <= self.max_cards {
            idx as f32
        } else {
            self.max_cards as f32
        };
        egui::Vec2::new(x, -x + self.inner_margin as f32)
    }
    pub fn max_cards(&mut self, max_cards: usize) {
        let size = egui::Vec2::new(100.0 + max_cards as f32, 144.0 + max_cards as f32);
        self.size = size;
        self.max_cards = max_cards;
    }
    #[allow(dead_code)]
    fn ui(&self, ui: &mut egui::Ui) -> egui::Response {
        frame::Frame::new()
            .inner_margin(egui::Margin::same(self.inner_margin))
            .outer_margin(egui::Margin::same(5))
            .stroke(egui::Stroke::new(2.0, egui::Color32::DEBUG_COLOR))
            .fill(egui::Color32::DARK_GREEN)
            .corner_radius(egui::CornerRadius::same(5))
            .show(ui, |ui| {
                let next_pos = ui.next_widget_position();
                ui.allocate_new_ui(
                    egui::UiBuilder::new()
                        .max_rect(egui::Rect::from_min_size(next_pos, self.size))
                        .layer_id(egui::LayerId::background()),
                    |ui| {
                        ui.set_max_size(self.size);
                        ui.set_min_size(self.size);
                        for (idx, _card) in self.cards.iter().enumerate() {
                            let _card_pos = next_pos.add(self.card_pos(idx));
                            todo!("Paint card here");
                        }
                    },
                )
                .response
            })
            .inner
    }
}
// TODO check if this version of HandLayout can be safely removed
pub struct HandLayout {
    pub cards: Vec<()>,
    pub pos: egui::Pos2,
    size: egui::Vec2,
    pub inner_margin: i8,
    max_cards: usize,
}
impl Default for HandLayout {
    fn default() -> Self {
        let mut x = Self {
            cards: vec![],
            pos: egui::Pos2::new(69.0, 420.0),
            size: Default::default(),
            inner_margin: 5,
            max_cards: 0,
        };
        x.max_cards(5);
        x
    }
}
impl HandLayout {
    pub fn max_cards(&mut self, max_cards: usize) {
        let size = egui::Vec2::new(
            (100.0 + self.inner_margin as f32) * max_cards as f32 - self.inner_margin as f32,
            144.0,
        );
        self.size = size;
        self.max_cards = max_cards;
    }
    #[allow(dead_code)]
    fn ui(&self, ui: &mut egui::Ui) -> egui::Response {
        frame::Frame::new()
            .inner_margin(egui::Margin::same(self.inner_margin))
            .outer_margin(egui::Margin::same(5))
            .stroke(egui::Stroke::new(2.0, egui::Color32::DEBUG_COLOR))
            .fill(egui::Color32::DARK_GREEN)
            .corner_radius(egui::CornerRadius::same(5))
            .show(ui, |ui| {
                let next_pos = ui.next_widget_position();
                ui.allocate_new_ui(
                    egui::UiBuilder::new()
                        .max_rect(egui::Rect::from_min_size(next_pos, self.size))
                        .layer_id(egui::LayerId::background()),
                    |ui| {
                        ui.set_max_size(self.size);
                        ui.set_min_size(self.size);
                        let pointer = ui.input(|state| state.pointer.clone());
                        let mut selected = None;
                        if pointer.latest_pos().is_some()
                            && ui.max_rect().contains(pointer.latest_pos().unwrap())
                        {
                            let left = ui.max_rect().left();
                            let right = ui.max_rect().right();
                            let selector = self.cards.len() as f32
                                * (pointer
                                    .latest_pos()
                                    .unwrap_or_else(|| egui::pos2(left, 0.0))
                                    .x
                                    - left)
                                / (right - left);
                            selected = Some(selector as usize);
                        }
                        for (_idx, _card) in self.cards.iter().enumerate() {
                            #[allow(unreachable_code)]
                            #[allow(clippy::diverging_sub_expression)]
                            let _card_pos =
                                next_pos.add(todo!("Calculate card position relative to Field"));
                            #[allow(unreachable_code)]
                            if selected.is_some() && _idx == selected.unwrap() {
                                continue;
                            }
                            todo!("Paint card here");
                        }
                        if selected.is_some() {
                            if let Some(_card) = self.cards.get(selected.unwrap()) {
                                #[allow(unreachable_code)]
                                #[allow(clippy::diverging_sub_expression)]
                                let _card_pos = next_pos
                                    .add(todo!(
                                        "Calculate position of selected card relative to Field"
                                    ))
                                    .add(egui::vec2(0.0, -10.0));
                                egui::Area::new(ui.next_auto_id())
                                    .order(egui::Order::Foreground)
                                    .sense(egui::Sense::all())
                                    .current_pos(_card_pos)
                                    .show(ui.ctx(), |ui| {
                                        egui::Frame::new()
                                            .stroke(egui::Stroke::new(2.0, egui::Color32::RED))
                                            .corner_radius(egui::CornerRadius::same(2))
                                            .show(ui, |ui| {
                                                ui.allocate_new_ui(egui::UiBuilder::new(), |_ui| {
                                                    todo!("Paint card here");
                                                });
                                            });
                                    });
                            }
                        }
                    },
                )
                .response
            })
            .inner
    }
}