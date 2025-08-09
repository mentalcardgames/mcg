#[cfg(target_arch = "wasm32")]
use egui::ColorImage;
use egui::{vec2, TextureHandle};
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, MediaStreamConstraints,
};

use super::{AppInterface, ScreenWidget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraFacing {
    User,
    Environment,
}
impl CameraFacing {
    pub fn as_str(&self) -> &'static str {
        match self {
            CameraFacing::User => "user",
            CameraFacing::Environment => "environment",
        }
    }
}
impl Default for CameraFacing {
    fn default() -> Self {
        CameraFacing::Environment
    }
}
impl std::fmt::Display for CameraFacing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[allow(dead_code)]
pub struct Camera {
    #[cfg(target_arch = "wasm32")]
    video_element: Option<HtmlVideoElement>,
    #[cfg(target_arch = "wasm32")]
    canvas_element: Option<HtmlCanvasElement>,
    #[cfg(target_arch = "wasm32")]
    context: Option<CanvasRenderingContext2d>,
    #[cfg(target_arch = "wasm32")]
    stream: Option<web_sys::MediaStream>,
    is_active: bool,
    is_video_ready: bool,
    frame_texture: Option<TextureHandle>,
    #[allow(dead_code)]
    frame_count: u32,
    last_qr_result: Option<String>,
    facing_mode: CameraFacing,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            video_element: None,
            #[cfg(target_arch = "wasm32")]
            canvas_element: None,
            #[cfg(target_arch = "wasm32")]
            context: None,
            #[cfg(target_arch = "wasm32")]
            stream: None,
            is_active: false,
            is_video_ready: false,
            frame_texture: None,
            frame_count: 0,
            last_qr_result: None,
            facing_mode: CameraFacing::Environment,
        }
    }
    #[cfg(target_arch = "wasm32")]
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
        video.set_width(640);
        video.set_height(480);
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        canvas.set_width(640);
        canvas.set_height(480);
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        let navigator = window.navigator();
        let media_devices = navigator
            .media_devices()
            .map_err(|_| JsValue::from_str("MediaDevices not available"))?;
        let constraints = MediaStreamConstraints::new();
        let video_constraints = js_sys::Object::new();
        js_sys::Reflect::set(
            &video_constraints,
            &JsValue::from_str("facingMode"),
            &JsValue::from_str(self.facing_mode.as_str()),
        )?;
        constraints.set_video(&video_constraints.into());
        let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream = wasm_bindgen_futures::JsFuture::from(stream_promise).await?;
        let media_stream = stream.dyn_into::<web_sys::MediaStream>()?;
        video.set_src_object(Some(&media_stream));
        let video_clone = video.clone();
        if let Ok(play_promise) = video_clone.play() {
            let play_result = wasm_bindgen_futures::JsFuture::from(play_promise).await;
            if let Err(e) = play_result {
                web_sys::console::log_1(&format!("Video play failed: {:?}", e).into());
            } else {
                web_sys::console::log_1(&"Video playing successfully".into());
            }
        }
        self.video_element = Some(video_clone);
        self.canvas_element = Some(canvas);
        self.context = Some(context);
        self.stream = Some(media_stream);
        self.is_active = true;
        Ok(video)
    }
    #[cfg(target_arch = "wasm32")]
    pub fn capture_frame(&mut self, ctx: &egui::Context) -> Result<(), JsValue> {
        if let (Some(video), Some(canvas), Some(context)) =
            (&self.video_element, &self.canvas_element, &self.context)
        {
            let ready_state = video.ready_state();
            let video_width = video.video_width();
            let video_height = video.video_height();
            let paused = video.paused();
            let ended = video.ended();
            if ready_state < 2 || video_width == 0 || video_height == 0 || paused || ended {
                return Ok(());
            }
            let canvas_width = video_width.min(640);
            let canvas_height = video_height.min(480);
            if canvas.width() != canvas_width || canvas.height() != canvas_height {
                canvas.set_width(canvas_width);
                canvas.set_height(canvas_height);
            }
            context.clear_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);
            context.draw_image_with_html_video_element_and_dw_and_dh(
                video,
                0.0,
                0.0,
                canvas_width as f64,
                canvas_height as f64,
            )?;
            let image_data =
                context.get_image_data(0.0, 0.0, canvas_width as f64, canvas_height as f64)?;
            let data = image_data.data();
            if data.len() == 0 {
                return Ok(());
            }
            let mut pixels = Vec::with_capacity((canvas_width * canvas_height) as usize);
            for i in (0..data.len()).step_by(4) {
                if i + 3 < data.len() {
                    let r = data[i];
                    let g = data[i + 1];
                    let b = data[i + 2];
                    let a = 255;
                    pixels.push(egui::Color32::from_rgba_unmultiplied(r, g, b, a));
                }
            }
            if pixels.len() != (canvas_width * canvas_height) as usize {
                return Ok(());
            }
            let color_image = ColorImage {
                size: [canvas_width as usize, canvas_height as usize],
                pixels: pixels.clone(),
            };
            self.frame_count += 1;
            if self.frame_count % 5 == 0 {
                self.process_qr_detection(&pixels, canvas_width as usize, canvas_height as usize);
            }
            if let Some(texture) = &mut self.frame_texture {
                texture.set(color_image, egui::TextureOptions::LINEAR);
            } else {
                self.frame_texture = Some(ctx.load_texture(
                    "camera_frame",
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        Ok(())
    }
    pub fn get_texture(&self) -> Option<&TextureHandle> {
        self.frame_texture.as_ref()
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn is_video_ready(&self) -> bool {
        self.is_video_ready
    }
    pub fn get_last_qr_result(&self) -> Option<&String> {
        self.last_qr_result.as_ref()
    }
    #[cfg(target_arch = "wasm32")]
    pub fn stop(&mut self) {
        if let Some(stream) = &self.stream {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        if let Some(video) = &self.video_element {
            video.set_src_object(None);
        }
        self.video_element = None;
        self.canvas_element = None;
        self.context = None;
        self.stream = None;
        self.is_active = false;
        self.is_video_ready = false;
        self.frame_texture = None;
        self.frame_count = 0;
        self.last_qr_result = None;
    }
    pub fn flip_camera(&mut self) {
        self.facing_mode = match self.facing_mode {
            CameraFacing::User => CameraFacing::Environment,
            CameraFacing::Environment => CameraFacing::User,
        };
    }
    pub fn get_facing_mode(&self) -> CameraFacing {
        self.facing_mode
    }
    #[cfg(target_arch = "wasm32")]
    fn process_qr_detection(&mut self, pixels: &[egui::Color32], width: usize, height: usize) {
        let mut gray_data = Vec::with_capacity(width * height);
        for pixel in pixels {
            let r = pixel.r() as f32;
            let g = pixel.g() as f32;
            let b = pixel.b() as f32;
            let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            gray_data.push(gray);
        }
        let mut prepared_image =
            rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| {
                gray_data[y * width + x]
            });
        let grids = prepared_image.detect_grids();
        for grid in grids {
            if let Ok((_meta, content)) = grid.decode() {
                let qr_content = content;
                self.last_qr_result = Some(qr_content);
                break;
            }
        }
    }
}

#[allow(dead_code)]
pub struct QrScreen {
    camera: Rc<RefCell<Camera>>,
    camera_started: bool,
    is_mobile: bool,
}
impl QrScreen {
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        let is_mobile = if let Some(window) = web_sys::window() {
            let has_touch = window.navigator().max_touch_points() > 0;
            let user_agent = window
                .navigator()
                .user_agent()
                .unwrap_or_default()
                .to_lowercase();
            let is_mobile_ua = user_agent.contains("mobile")
                || user_agent.contains("android")
                || user_agent.contains("iphone")
                || user_agent.contains("ipad")
                || user_agent.contains("ipod");
            has_touch || is_mobile_ua
        } else {
            false
        };
        #[cfg(not(target_arch = "wasm32"))]
        let is_mobile = false;
        Self {
            camera: Rc::new(RefCell::new(Camera::new())),
            camera_started: false,
            is_mobile,
        }
    }
}
impl Default for QrScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for QrScreen {
    fn ui(
        &mut self,
        _app_interface: &mut AppInterface,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        let ctx = ui.ctx().clone();
        ui.heading("QR Scanner");
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            if !self.camera_started {
                if ui
                    .add_sized(vec2(150.0, 30.0), egui::Button::new("Start Camera"))
                    .clicked()
                {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let camera_ref = self.camera.clone();
                        self.camera_started = true;
                        wasm_bindgen_futures::spawn_local(async move {
                            let _ = camera_ref.borrow_mut().start().await;
                        });
                    }
                }
            } else {
                if ui
                    .add_sized(vec2(150.0, 30.0), egui::Button::new("Stop Camera"))
                    .clicked()
                {
                    #[cfg(target_arch = "wasm32")]
                    if let Ok(mut camera) = self.camera.try_borrow_mut() {
                        camera.stop();
                    }
                    self.camera_started = false;
                }
                if self.is_mobile {
                    let facing_mode = {
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Ok(camera) = self.camera.try_borrow() {
                                camera.get_facing_mode()
                            } else {
                                CameraFacing::Environment
                            }
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            CameraFacing::Environment
                        }
                    };
                    let (button_text, current_camera) = if facing_mode == CameraFacing::User {
                        ("🔄 Switch to Rear", "📷 Front Camera")
                    } else {
                        ("🔄 Switch to Front", "📷 Rear Camera")
                    };
                    ui.horizontal(|ui| {
                        ui.label("Current:");
                        ui.colored_label(egui::Color32::from_rgb(0, 150, 0), current_camera);
                    });
                    if ui
                        .add_sized(vec2(150.0, 30.0), egui::Button::new(button_text))
                        .on_hover_text("Switch between front and rear camera")
                        .clicked()
                    {
                        #[cfg(target_arch = "wasm32")]
                        if let Ok(mut camera) = self.camera.try_borrow_mut() {
                            camera.stop();
                            camera.flip_camera();
                            let camera_ref = self.camera.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = camera_ref.borrow_mut().start().await;
                            });
                        }
                    }
                }
            }
        });
        ui.add_space(20.0);
        if self.camera_started {
            #[cfg(target_arch = "wasm32")]
            {
                if let Ok(mut camera) = self.camera.try_borrow_mut() {
                    let _ = camera.capture_frame(&ctx);
                    if let Some(texture) = camera.get_texture() {
                        ui.add(
                            egui::Image::from_texture(texture)
                                .max_size(vec2(640.0, 480.0))
                                .corner_radius(egui::CornerRadius::same(5)),
                        );
                    } else if camera.is_video_ready() {
                        ui.label("Processing video frames...");
                    } else {
                        ui.label("Waiting for camera to initialize...");
                    }
                } else {
                    ui.label("Camera busy...");
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.label("Camera not available on native build");
            }
            ctx.request_repaint();
        } else {
            ui.allocate_ui_with_layout(
                vec2(640.0, 480.0),
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.label("Camera preview will appear here");
                    ui.label("Click 'Start Camera' to begin");
                },
            );
        }
        ui.add_space(20.0);
        ui.label("Point your camera at a QR code to scan it");
        #[cfg(target_arch = "wasm32")]
        if let Ok(camera) = self.camera.try_borrow() {
            if let Some(qr_result) = camera.get_last_qr_result() {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                ui.heading("QR Code Detected:");
                ui.horizontal(|ui| {
                    ui.label("Content:");
                    ui.monospace(qr_result);
                });
                if ui.button("Copy to Clipboard").clicked() {
                    ui.ctx().copy_text(qr_result.clone());
                }
            }
        }
    }
}
