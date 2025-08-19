use egui::{Image, Vec2};
#[allow(unused_imports)]
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
#[allow(unused_imports)]
use std::rc::Rc;
use std::slice::Iter;
#[cfg(target_arch = "wasm32")]
use web_sys;

pub trait CardEncoding {
    fn t(&self) -> Option<usize>;
    fn is_masked(&self) -> bool;
    fn is_open(&self) -> bool {
        !self.is_masked()
    }
    fn mask(self) -> Self;
    fn open(self) -> Self;
}
impl CardEncoding for SimpleCard {
    fn t(&self) -> Option<usize> {
        match self {
            SimpleCard::Open(t) => Some(*t),
            SimpleCard::Masked(_) => None,
        }
    }
    fn is_masked(&self) -> bool {
        matches!(self, SimpleCard::Masked(_))
    }
    fn mask(self) -> Self {
        match self {
            SimpleCard::Open(t) => Self::Masked(Some(t)),
            _ => self,
        }
    }
    fn open(self) -> Self {
        match self {
            SimpleCard::Masked(Some(t)) => Self::Open(t),
            _ => self,
        }
    }
}

#[derive(Hash, Debug, Clone)]
pub enum SimpleCard {
    Open(usize),
    Masked(Option<usize>),
}

fn get_origin() -> String {
    let window = web_sys::window().expect("should have a window in this context");
    let location = window.location();
    let origin = location
        .origin()
        .expect("should have an origin in this context");
    origin
}

#[allow(non_snake_case)]
pub trait CardConfig {
    fn img(&self, t: &impl CardEncoding) -> Image<'_>;
    fn T(&self) -> usize;
    fn w(&self) -> u32;
    fn natural_size(&self) -> Vec2;
    fn draw_at(
        &self,
        ui: &mut egui::Ui,
        t: &impl CardEncoding,
        pos: egui::Pos2,
    ) -> egui::InnerResponse<egui::Response> {
        let area = egui::Area::new(ui.next_auto_id()).current_pos(pos);
        area.show(ui.ctx(), |ui| ui.add(self.img(t)))
    }
}
impl CardConfig for DirectoryCardType {
    fn img(&self, t: &impl CardEncoding) -> Image<'_> {
        let origin = get_origin();
        let path = format!(
            "{origin}/media/{folder}/{card}",
            origin = origin,
            folder = self.path,
            card = self.img_names[t.t().unwrap_or(0)]
        );
        Image::new(path)
            .show_loading_spinner(true)
            .maintain_aspect_ratio(true)
    }
    fn T(&self) -> usize {
        self.T
    }
    fn w(&self) -> u32 {
        self.w
    }
    fn natural_size(&self) -> Vec2 {
        self.natural_size
    }
}

#[derive(Clone)]
#[allow(non_snake_case)]
pub struct DirectoryCardType {
    pub(crate) path: String,
    pub(crate) img_names: Vec<String>,
    pub(crate) T: usize,
    pub(crate) w: u32,
    pub(crate) natural_size: Vec2,
}
impl DirectoryCardType {
    #[allow(non_snake_case)]
    pub fn new(path: String, img_names: Vec<String>, natural_size: Vec2) -> Self {
        let T = img_names.len();
        let w = T.next_power_of_two().ilog2();
        Self {
            path,
            img_names,
            T,
            w,
            natural_size,
        }
    }
    pub fn all_images(&self) -> Iter<'_, String> {
        self.img_names.iter()
    }
}
impl Debug for DirectoryCardType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectoryCardType")
            .field("path", &self.path)
            .field("T", &self.T)
            .field("natural_size", &self.natural_size)
            .finish()
    }
}
