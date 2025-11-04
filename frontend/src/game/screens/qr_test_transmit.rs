use super::{AppInterface, ScreenDef, ScreenMetadata, ScreenWidget};
use crate::game::websocket::WebSocketConnection;
use crate::sprintln;
use egui::{vec2, ColorImage, Context, Image, TextureHandle, TextureOptions};
use image::{ImageBuffer, Luma};
use js_sys::Date;
use mcg_qr_comm::data_structures::Package;
use mcg_qr_comm::network_coding::Epoch;
use mcg_qr_comm::MAX_PARTICIPANTS;
use mcg_shared::{ClientMsg, PlayerConfig, PlayerId, ServerMsg};
use qrcode::QrCode;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Default)]
pub struct QrTestTransmit {
    qr_queue: VecDeque<ImageBuffer<Luma<u8>, Vec<u8>>>,
    input: String,
    texture_handle: Option<TextureHandle>,
    web_socket_connection: WebSocketConnection,
    epoch: Rc<RefCell<Epoch>>,
    file_list: Vec<String>,
    zoom: f32,
    last_code_shown: Option<f64>,
}

impl QrTestTransmit {
    fn gen_new_code(&mut self) {
        if let Ok(epoch) = self.epoch.try_borrow_mut() {
            if let Some(frame) = epoch.pop_recent_frame() {
                let qr_res: Result<QrCode, _> = frame.try_into();
                if let Ok(qr) = qr_res {
                    let image = qr.render::<Luma<u8>>().build();
                    self.qr_queue.push_back(image);
                }
            }
        }
    }
    fn show_next_code(&mut self, ctx: &Context) {
        if let Some(img) = self.qr_queue.pop_front() {
            let size = [img.width() as usize, img.height() as usize];
            let data = img.iter().as_slice();
            let color_img = ColorImage::from_gray(size, data);
            let texture_handle = ctx.load_texture("qr_code", color_img, TextureOptions::default());
            self.texture_handle.replace(texture_handle);
        } else {
            self.texture_handle = None;
        }
    }
}

impl ScreenWidget for QrTestTransmit {
    fn ui(
        &mut self,
        _app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let ctx = ui.ctx().clone();
        ui.heading("QR Transmission Demo");
        ui.add_space(12.0);
        ui.label(format!("QR-Codes in Queue: {}", self.qr_queue.len()));
        let id = if let Ok(epoch) = self.epoch.try_borrow_mut() {
            Some(epoch.header.participant)
        } else {
            None
        };
        ui.label(format!("Current participant ID: {:?}", id));
        ui.horizontal(|ui| {
            ui.label("Text to transmit:");
            ui.text_edit_singleline(&mut self.input);
            if ui.button("Write custom AP").clicked() {
                if let Ok(mut epoch) = self.epoch.try_borrow_mut() {
                    let ap = Package::new(self.input.as_bytes());
                    epoch.write(ap);
                    epoch.header.participant += 1;
                    epoch.header.participant %= MAX_PARTICIPANTS as u8;
                }
            }
            if ui.button("Request next AP").clicked() {
                if let Ok(epoch) = self.epoch.try_borrow_mut() {
                    if let Some(file) = self.file_list.get(epoch.header.participant as usize) {
                        let message = ClientMsg::QrReq(file.clone());
                        self.web_socket_connection.send_msg(&message);
                    }
                }
            }
            if ui.button("Generate Frame").clicked() && self.last_code_shown.is_none() {
                self.last_code_shown.replace(Date::now());
                self.gen_new_code();
            }
            if let Some(last) = self.last_code_shown {
                let now = Date::now();
                if now - last >= 50.0 {
                    // 20 Hz
                    self.last_code_shown.replace(now);
                    while self.qr_queue.len() < 3 {
                        self.gen_new_code();
                    }
                    self.show_next_code(&ctx);
                }
            }
            if ui.button("Stop").clicked() {
                self.last_code_shown.take();
            }
        });
        ui.add_space(12.0);
        if ui.button("Next").clicked() && self.last_code_shown.is_none() {
            self.last_code_shown.replace(Date::now());
            self.show_next_code(&ctx);
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label("Zoom:");
            ui.add(
                egui::Slider::new(&mut self.zoom, 0.0..=1.0)
                    .text("Zoom")
                    .min_decimals(3),
            );
        });
        if let Some(handle) = &self.texture_handle {
            let width = ui.available_width();
            let height = ui.available_height();
            let mut size = if width <= height { width } else { height };
            size *= self.zoom;
            let image = Image::from_texture(handle).fit_to_exact_size(vec2(size, size));
            ui.add(image);
        }
    }
}

impl ScreenDef for QrTestTransmit {
    fn metadata() -> ScreenMetadata
    where
        Self: Sized,
    {
        ScreenMetadata {
            path: "/transmit",
            display_name: "Generate QR-Codes",
            icon: "🔍",
            description: "Send QR-Codes to peers",
            show_in_menu: true,
        }
    }

    fn create() -> Box<dyn ScreenWidget>
    where
        Self: Sized,
    {
        let mut me = Self::default();
        me.file_list.push(String::from("data_0.txt"));
        me.file_list.push(String::from("data_1.txt"));
        me.file_list.push(String::from("homepage.md"));
        me.file_list.push(String::from("dataset-card.png"));
        let epoch_copy = me.epoch.clone();
        let on_msg = move |x| match x {
            ServerMsg::State(s) => {
                sprintln!("Got a message state:\n\t- {:?}", s);
            }
            ServerMsg::Error(e) => {
                sprintln!("Got a message error:\n\t- {:?}", e);
            }
            ServerMsg::QrRes(content) => {
                let s = String::from_utf8_lossy(&content);
                sprintln!("Got a response:\n\t- {:?}", s);
                if let Ok(mut epoch) = epoch_copy.try_borrow_mut() {
                    let ap = Package::new(&content);
                    epoch.write(ap);
                    epoch.header.participant += 1;
                    epoch.header.participant %= MAX_PARTICIPANTS as u8;
                }
            }
        };
        let on_err = |e| {
            sprintln!("Got an error:\n\t- {:?}", e);
        };
        let on_cls = |c| {
            sprintln!("Got a close:\n\t- {:?}", c);
        };
        let mut players = Vec::new();
        let p = PlayerConfig {
            id: PlayerId::from(1337),
            name: "QR_COMM".to_string(),
            is_bot: false,
        };
        players.push(p);
        me.web_socket_connection
            .connect("127.0.0.1:3000", players, on_msg, on_err, on_cls);
        Box::new(me)
    }
}
