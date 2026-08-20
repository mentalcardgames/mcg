use crate::app::FrontendInterface;
use crate::sprintln;
use egui::{vec2, ColorImage, Context, Image, TextureHandle, TextureOptions};
use image::{ImageBuffer, Luma};
use js_sys::Date;
use mcg_qr_comm::data_structures::Package;
use mcg_qr_comm::network_coding::Epoch;
use mcg_qr_comm::MAX_PARTICIPANTS;
use mcg_shared::{Backend2FrontendMsg, Frontend2BackendMsg};
use qrcode::QrCode;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;
use crate::widgets::screen::{ScreenDef, ScreenMetadata, ScreenWidget};

#[derive(Default)]
pub struct QrTestTransmit {
    qr_queue: VecDeque<ImageBuffer<Luma<u8>, Vec<u8>>>,
    input: String,
    texture_handle: Option<TextureHandle>,
    epoch: Rc<RefCell<Epoch>>,
    file_list: Vec<String>,
    zoom: f32,
    last_code_shown: Option<f64>,
    initialized: bool,
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
        app_interface: &mut FrontendInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let ctx = ui.ctx().clone();

        // Lazy connect using central WebSocket
        if !self.initialized {
            let server = app_interface.state().server_address.clone();
            if !app_interface.is_connected() {
                app_interface.connect(&server);
            }

            self.initialized = true;
        }

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
                        let message = Frontend2BackendMsg::QrReq(file.clone());
                        app_interface.send_msg(message);
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
                    ctx.request_repaint_after(Duration::from_millis(100));
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

    fn on_message(&mut self, _app_interface: &mut FrontendInterface, message: Backend2FrontendMsg) {
        match message {
            Backend2FrontendMsg::QrRes(content) => {
                let text = String::from_utf8_lossy(&content);
                sprintln!("Got a response:\n\t- {:?}", text);
                if let Ok(mut epoch) = self.epoch.try_borrow_mut() {
                    let package = Package::new(&content);
                    epoch.write(package);
                    epoch.header.participant += 1;
                    epoch.header.participant %= MAX_PARTICIPANTS as u8;
                }
            }
            other => sprintln!("Got an unhandled message:\n\t- {:?}", other),
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
        Box::new(me)
    }
}
