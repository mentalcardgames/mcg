use crate::widgets::camera::Camera;

pub enum QrDecodeTarget<'a> {
    String(&'a mut String),
    Binary(&'a mut Vec<u8>),
}

#[derive(Default)]
pub struct QrScanner {
    open: bool,
    camera: Camera,
    frame_count: u32,
}

impl QrScanner {
    pub fn button_and_popup(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        target: QrDecodeTarget<'_>,
    ) {
        let close_after_decode = matches!(&target, QrDecodeTarget::String(_));
        if ui
            .button("Scan QR")
            .on_hover_text("Open camera to scan a QR code")
            .clicked()
        {
            self.open = true;
            self.start_camera();
        }
        if self.open {
            let mut open = true;
            egui::Window::new("Scan QR")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if self.camera.is_active() {
                        // If camera failed to start previously, keep showing a friendly message
                        let frame = self.camera.capture_frame(ctx).ok().flatten();
                        if let Some(texture) = self.camera.get_texture() {
                            ui.add(
                                egui::Image::from_texture(texture)
                                    .max_size(egui::vec2(640.0, 480.0))
                                    .corner_radius(egui::CornerRadius::same(5)),
                            );
                        } else {
                            ui.label("Waiting for camera to initialize...");
                        }

                        if let Some(frame) = frame {
                            self.frame_count = self.frame_count.wrapping_add(1);
                            if self.frame_count.is_multiple_of(5)
                                && Self::decode(&frame, target)
                                && close_after_decode
                            {
                                self.stop_camera();
                            }
                        }
                    } else {
                        ui.label("Camera busy...");
                    }
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.stop_camera();
                    }
                    if ui
                        .button("Retry")
                        .on_hover_text("Retry initializing the camera")
                        .clicked()
                    {
                        self.stop_camera();
                        self.start_camera();
                    }
                    if ui
                        .button("Change Camera")
                        .on_hover_text("Change which camera is used")
                        .clicked()
                    {
                        self.stop_camera();
                        self.camera.flip_camera();
                        self.start_camera();
                    }
                });
            if !open {
                self.stop_camera();
            }
            ctx.request_repaint();
        }
    }

    fn start_camera(&mut self) {
        self.frame_count = 0;
        self.camera.start();
    }

    fn stop_camera(&mut self) {
        if self.camera.is_active() {
            self.camera.stop();
        }
        self.open = false;
    }

    pub fn decode(frame: &egui::ColorImage, target: QrDecodeTarget<'_>) -> bool {
        let [width, height] = frame.size;
        let mut gray_data = Vec::with_capacity(width * height);
        for pixel in &frame.pixels {
            let gray = (0.299 * pixel.r() as f32
                + 0.587 * pixel.g() as f32
                + 0.114 * pixel.b() as f32) as u8;
            gray_data.push(gray);
        }

        let mut prepared_image =
            rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| {
                gray_data[y * width + x]
            });

        let grids = prepared_image.detect_grids();
        match target {
            QrDecodeTarget::String(output) => {
                for grid in grids {
                    if let Ok((_meta, decoded)) = grid.decode() {
                        *output = decoded;
                        return true;
                    }
                }
            }
            QrDecodeTarget::Binary(output) => {
                for grid in grids {
                    let mut decoded = Vec::new();
                    if grid.decode_to(&mut decoded).is_ok() {
                        *output = decoded;
                        return true;
                    }
                }
            }
        }

        false
    }
}
