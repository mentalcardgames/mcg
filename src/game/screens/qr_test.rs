use egui::{vec2, ColorImage, TextureHandle};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, MediaStreamConstraints,
};

use super::{ScreenType, ScreenWidget};

use rqrr;

pub struct Camera {
    video_element: Option<HtmlVideoElement>,
    canvas_element: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
    stream: Option<web_sys::MediaStream>,
    is_active: bool,
    is_video_ready: bool,
    frame_texture: Option<TextureHandle>,
    frame_count: u32,
    last_qr_result: Option<String>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            video_element: None,
            canvas_element: None,
            context: None,
            stream: None,
            is_active: false,
            is_video_ready: false,
            frame_texture: None,
            frame_count: 0,
            last_qr_result: None,
        }
    }
    pub async fn start(&mut self) -> Result<HtmlVideoElement, JsValue> {
        if self.is_active {
            return Ok(self.video_element.clone().unwrap());
        }
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");

        // Create video element
        let video = document
            .create_element("video")?
            .dyn_into::<HtmlVideoElement>()?;
        video.set_autoplay(true);
        video.set_muted(true);
        video.set_attribute("playsinline", "true").unwrap();
        video.set_width(640);
        video.set_height(480);

        // Create canvas element for capturing frames
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
        // Use simple boolean constraint for now to avoid js-sys complexity
        constraints.set_video(&JsValue::from_bool(true));
        let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
        let stream = wasm_bindgen_futures::JsFuture::from(stream_promise).await?;
        let media_stream = stream.dyn_into::<web_sys::MediaStream>()?;
        video.set_src_object(Some(&media_stream));

        // Wait for metadata to load
        let video_clone = video.clone();

        // Try to play the video
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

    pub fn capture_frame(&mut self, ctx: &egui::Context) -> Result<(), JsValue> {
        if let (Some(video), Some(canvas), Some(context)) =
            (&self.video_element, &self.canvas_element, &self.context)
        {
            // Check if video is ready and has data
            let ready_state = video.ready_state();
            let video_width = video.video_width();
            let video_height = video.video_height();
            let paused = video.paused();
            let ended = video.ended();

            // Debug video state
            if ready_state < 2 || video_width == 0 || video_height == 0 || paused || ended {
                if !self.is_video_ready {
                    web_sys::console::log_1(
                        &format!(
                            "Video not ready: state={}, w={}, h={}, paused={}, ended={}",
                            ready_state, video_width, video_height, paused, ended
                        )
                        .into(),
                    );
                }
                return Ok(());
            }

            if !self.is_video_ready {
                self.is_video_ready = true;
                web_sys::console::log_1(
                    &format!("Video ready: {}x{}", video_width, video_height).into(),
                );
            }

            // Use actual video dimensions but limit canvas size
            let canvas_width = video_width.min(640);
            let canvas_height = video_height.min(480);

            // Ensure canvas is the right size
            if canvas.width() != canvas_width || canvas.height() != canvas_height {
                canvas.set_width(canvas_width);
                canvas.set_height(canvas_height);
            }

            // Clear canvas first
            context.clear_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);

            // Draw video frame to canvas
            context.draw_image_with_html_video_element_and_dw_and_dh(
                video,
                0.0,
                0.0,
                canvas_width as f64,
                canvas_height as f64,
            )?;

            // Get image data from canvas
            let image_data =
                context.get_image_data(0.0, 0.0, canvas_width as f64, canvas_height as f64)?;
            let data = image_data.data();

            if data.len() == 0 {
                return Ok(());
            }

            // Convert RGBA data to egui ColorImage
            let mut pixels = Vec::with_capacity((canvas_width * canvas_height) as usize);

            // ImageData is in RGBA format
            for i in (0..data.len()).step_by(4) {
                if i + 3 < data.len() {
                    let r = data[i];
                    let g = data[i + 1];
                    let b = data[i + 2];
                    let a = 255; // Force full opacity
                    pixels.push(egui::Color32::from_rgba_unmultiplied(r, g, b, a));
                }
            }

            if pixels.len() != (canvas_width * canvas_height) as usize {
                web_sys::console::log_1(
                    &format!(
                        "Pixel count mismatch: expected {}, got {}",
                        canvas_width * canvas_height,
                        pixels.len()
                    )
                    .into(),
                );
                return Ok(());
            }

            let color_image = ColorImage {
                size: [canvas_width as usize, canvas_height as usize],
                pixels: pixels.clone(),
            };

            // Increment frame count and process QR detection on every 5th frame
            self.frame_count += 1;
            if self.frame_count % 5 == 0 {
                self.process_qr_detection(&pixels, canvas_width as usize, canvas_height as usize);
            }

            // Update texture
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

    fn process_qr_detection(&mut self, pixels: &[egui::Color32], width: usize, height: usize) {
        // Convert egui::Color32 pixels to grayscale for QR detection
        let mut gray_data = Vec::with_capacity(width * height);

        for pixel in pixels {
            // Convert RGBA to grayscale using luminance formula
            let r = pixel.r() as f32;
            let g = pixel.g() as f32;
            let b = pixel.b() as f32;
            let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            gray_data.push(gray);
        }

        // Create rqrr PreparedImage using prepare_from_greyscale
        let mut prepared_image =
            rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| {
                gray_data[y * width + x]
            });

        // Detect QR codes
        let grids = prepared_image.detect_grids();

        for grid in grids {
            if let Ok((meta, content)) = grid.decode() {
                let qr_content = content;
                web_sys::console::log_1(&format!("QR Code detected: {}", qr_content).into());
                web_sys::console::log_1(&format!("QR Metadata: {:?}", meta).into());
                self.last_qr_result = Some(qr_content);
                break; // Only process the first QR code found
            }
        }
    }
}

pub struct QrScreen {
    camera: Rc<RefCell<Camera>>,
    camera_started: bool,
}

impl QrScreen {
    pub fn new() -> Self {
        Self {
            camera: Rc::new(RefCell::new(Camera::new())),
            camera_started: false,
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
        _frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("QR Scanner");
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(vec2(100.0, 30.0), egui::Button::new("Back"))
                    .clicked()
                {
                    *next_screen.borrow_mut() = ScreenType::Main;
                }

                if !self.camera_started {
                    if ui
                        .add_sized(vec2(150.0, 30.0), egui::Button::new("Start Camera"))
                        .clicked()
                    {
                        let camera_ref = self.camera.clone();
                        self.camera_started = true;
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Err(e) = camera_ref.borrow_mut().start().await {
                                web_sys::console::log_1(
                                    &format!("Camera start error: {:?}", e).into(),
                                );
                            }
                        });
                    }
                } else {
                    ui.label("Camera Active");
                }
            });

            ui.add_space(20.0);

            // Display camera preview
            if self.camera_started {
                // Capture frame from video
                if let Ok(mut camera) = self.camera.try_borrow_mut() {
                    if let Err(e) = camera.capture_frame(ctx) {
                        web_sys::console::log_1(&format!("Frame capture error: {:?}", e).into());
                    }

                    // Display the captured frame or status
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

                // Request continuous repainting for video updates
                ctx.request_repaint();
            } else {
                // Show placeholder when camera is not started
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

            // Display QR detection results
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

                    // Add copy to clipboard functionality
                    if ui.button("Copy to Clipboard").clicked() {
                        ui.ctx().copy_text(qr_result.clone());
                    }
                }
            }
        });
    }
}
