use egui::vec2;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlVideoElement, MediaStreamConstraints};

use super::{ScreenType, ScreenWidget};

pub struct Camera {
    video_element: Option<HtmlVideoElement>,
    stream: Option<web_sys::MediaStream>,
    is_active: bool,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            video_element: None,
            stream: None,
            is_active: false,
        }
    }
    pub async fn start(&mut self) -> Result<HtmlVideoElement, JsValue> {
        if self.is_active {
            return Ok(self.video_element.clone().unwrap());
        }
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");
        let video = document
            .create_element("video")?
            .dyn_into::<HtmlVideoElement>()?;
        video.set_autoplay(true);
        video.set_muted(true);
        video.set_attribute("playsinline", "true").unwrap();

        let navigator = window.navigator();
        let media_devices = navigator
            .media_devices()
            .map_err(|_| JsValue::from_str("MediaDevices not avaiable"))?;
        let constraints = MediaStreamConstraints::new();
        constraints.set_video(&JsValue::from_bool(true));
        let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream = wasm_bindgen_futures::JsFuture::from(stream_promise).await?;
        let media_stream = stream.dyn_into::<web_sys::MediaStream>()?;
        video.set_src_object(Some(&media_stream));
        self.video_element = Some(video.clone());
        self.stream = Some(media_stream);
        self.is_active = true;
        Ok(video)
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

pub struct QrScreen {
    camera: Rc<RefCell<Camera>>,
}

impl QrScreen {
    pub fn new() -> Self {
        Self {
            camera: Rc::new(RefCell::new(Camera::new())),
        }
    }
}

impl Default for QrScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for QrScreen {
    fn update(
        &mut self,
        next_screen: std::rc::Rc<std::cell::RefCell<super::ScreenType>>,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("QR Heading");
            ui.add_space(20.0);
            if ui
                .add_sized(vec2(100.0, 30.0), egui::Button::new("Back"))
                .clicked()
            {
                *next_screen.borrow_mut() = ScreenType::Main;
            }
            ui.add_space(20.0);
            if ui
                .add_sized(vec2(200.0, 50.0), egui::Button::new("QR start"))
                .clicked()
            {
                let camera_ref = self.camera.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = camera_ref.borrow_mut().start().await {
                        web_sys::console::log_1(&format!("Camera start error: {:?}", e).into());
                    }
                });
            }
        });
    }
}
