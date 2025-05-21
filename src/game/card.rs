#[cfg(target_arch = "wasm32")]
use crate::openDirectoryPicker;
use egui::{Image, Vec2};
#[allow(unused_imports)]
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
#[allow(unused_imports)]
use std::rc::Rc;
use std::slice::Iter;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::js_sys::Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::{spawn_local, JsFuture};

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

#[derive(Hash, Debug)]
pub enum SimpleCard {
    /// You are supposed to ensure your card isn't outside its type bounds!
    /// e.g. assert!(t < T)
    Open(usize),
    /// You don't have to specify which type this card *really* is
    Masked(Option<usize>),
}

// TODO make CardEncoding be usable as Idx for Trait Index
#[allow(non_snake_case)]
pub trait CardConfig {
    fn img(&self, t: &impl CardEncoding) -> Image;
    fn T(&self) -> usize;
    fn w(&self) -> u32;
    fn natural_size(&self) -> Vec2;
    // Is draw_at(...) needed when egui::Image::paint_at(...) exists?
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
    fn img(&self, t: &impl CardEncoding) -> Image {
        let path = format!(
            "media/{folder}/{card}",
            folder = self.path,
            card = self.img_names[t.t().unwrap_or(0)]
        );
        Image::new(path)
            .show_loading_spinner(true)
            .maintain_aspect_ratio(true)
    }
    #[allow(non_snake_case)]
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
    /// It's assumed the image URL is inside servers /media directory and the
    /// type order corresponds to the lexicographical.
    ///
    /// For real file upload you need to extend the simple python http server to accept uploads.
    /// Does pythons simple https server already accept POST requests?
    #[cfg(target_arch = "wasm32")]
    pub fn new_from_selection(holder: Rc<RefCell<Option<DirectoryCardType>>>) {
        let type_rc = Rc::clone(&holder);
        spawn_local(async move {
            let response = JsFuture::from(openDirectoryPicker()).await;
            if let Ok(file_info_array) = response {
                let file_info_array: Array = file_info_array.into();
                let mut path = String::new();
                let mut img_names = Vec::new();
                let img_size = Into::<Array>::into(file_info_array.pop())
                    .to_vec()
                    .iter()
                    .map(|x| x.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<f32>>();
                let natural_size = egui::vec2(img_size[0], img_size[1]);
                let file_info_array: Array = file_info_array.pop().into();
                for file_info in file_info_array {
                    let file_info: Array = Array::from(&file_info);
                    let file_info: Vec<String> = file_info
                        .iter()
                        .map(|x| x.as_string().unwrap().clone())
                        .collect();
                    let file_name = file_info.first().expect("Every file has a name!").clone();
                    if path.is_empty() {
                        path = file_info
                            .get(1)
                            .expect("Every file has a path!")
                            .clone()
                            .strip_suffix(format!("/{file_name}").as_str())
                            .unwrap()
                            .to_string();
                    }
                    let file_type = file_info.get(2);
                    if let Some(file_type) = file_type {
                        if file_type.starts_with("image") {
                            img_names.push(file_name);
                        }
                    }
                }
                img_names.sort();
                let card_type = Self::new(path, img_names, natural_size);
                type_rc.borrow_mut().replace(card_type);
            }
        });
    }
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
    pub fn all_images(&self) -> Iter<String> {
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